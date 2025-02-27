
use alloc::borrow::ToOwned;
use bitvec::prelude::{BitArr, Msb0, bitarr};
use efm32pg23_fix::Peripherals;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, Point},
    pixelcolor::BinaryColor,
    primitives::rectangle::Rectangle,
    prelude::Drawable,
    Pixel,
};

use kampela_display_common::display_def::*;
use qrcodegen_no_heap::{QrCode, QrCodeEcc, Version};

use crate::{
    devices::{display::{
        Bounds, Request, UpdateFast, UpdateFull, UpdateUltraFast
    }, touch::{disable_touch_int, enable_touch_int}},
    parallel::{AsyncOperation, Threads}
};
use kampela_ui::uistate::UpdateRequest;
use crate::debug_display::epaper_draw_stuff_differently;

// x and y of framebuffer and display RAM address are inversed
fn refreshable_area_address(refreshable_area: Rectangle) -> Bounds {
    let x_start_address: u8 = if refreshable_area.top_left.y < 0 {
        0
    } else if refreshable_area.top_left.y > (SCREEN_SIZE_Y - 1) as i32 {
        (SCREEN_SIZE_Y / 8 - 1) as u8
    } else {
        (refreshable_area.top_left.y / 8) as u8
    };

    let y_start_address: u16 = if refreshable_area.top_left.x < 0 {
        (SCREEN_SIZE_X) as u16
    } else if refreshable_area.top_left.x > (SCREEN_SIZE_X - 1) as i32{
        0
    } else {
        ((SCREEN_SIZE_X) as i32 - refreshable_area.top_left.x) as u16
    };

    let bottom_right = refreshable_area.top_left + refreshable_area.size - Point{x: 1, y: 1};
    
    let x_end_address: u8 = if bottom_right.y > (SCREEN_SIZE_Y - 1) as i32 {
        (SCREEN_SIZE_Y / 8 - 1) as u8
    } else if bottom_right.y < 0 {
        0
    } else {
        (bottom_right.y / 8) as u8
    };

    let y_end_address: u16 = if bottom_right.x > (SCREEN_SIZE_X - 1) as i32 {
        0
    } else if bottom_right.x < 0 {
        (SCREEN_SIZE_X - 1) as u16
    } else {
        ((SCREEN_SIZE_X - 1) as i32 - bottom_right.x) as u16
    };

    (x_start_address, x_end_address, y_start_address, y_end_address)
}

#[derive(Debug)]
pub enum DisplayError {}

/// These are voltage thresholds to allow screen updates;
/// for wired debug, set both well below 5000
///
//TODO tune these values for prod; something like 12k and 8k
const FAST_REFRESH_POWER: i32 = 5000;
const FULL_REFRESH_POWER: i32 = 5000;
const PART_REFRESH_POWER: i32 = 5000;

const SEQUENCIAL_SELECTIVE_LIMIT: usize = 5; // more sequencial selective refreshes cause to leave traces, less cause artefacts
/// Virtual display data storage
type PixelData = BitArr!(for SCREEN_RESOLUTION as usize, in u8, Msb0);


/// A virtual display that could be written to EPD simultaneously
pub struct FrameBuffer {
    data: PixelData,
}

pub struct DisplayOperationThreads{
    threads: Threads<DisplayState, 1>,
    next: Option<UpdateRequest>,
    last_black: bool,
    selective_counter: usize,
}

impl core::ops::Deref for DisplayOperationThreads {
    type Target = Threads<DisplayState, 1>;

    fn deref(&self) -> &Self::Target {
        &self.threads
    }
}

impl core::ops::DerefMut for DisplayOperationThreads {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.threads
    }
}

impl DisplayOperationThreads {
    pub fn new() -> Self {
        Self{
            threads: Threads::new(DisplayState::IdleOrPending),
            next: None,
            last_black: false,
            selective_counter: 0,
        }
    }

    pub fn try_add_next(&mut self, next: UpdateRequest) -> bool {
        if self.next.is_none() {
            self.next = Some(next);
            true
        } else {
            false
        }
    }

    pub fn is_pending(&self) -> bool {
        if self.next.is_none() {
            false
        } else {
            true
        }
    }

    /// Start part display update sequence with white draw
    pub fn request(&mut self, voltage: i32) -> Option<bool> {
        if let Some(u) = &self.next {
            match u {
                UpdateRequest::Slow => {
                    if voltage > FULL_REFRESH_POWER {
                        self.change(DisplayState::FullOperating(None));
                        disable_touch_int();
                        return Some(false)
                    }
                },
                UpdateRequest::Fast => {
                    if voltage > FAST_REFRESH_POWER {
                        self.change(DisplayState::FastOperating(None));
                        return Some(false)
                    }
                },
                UpdateRequest::UltraFast => {
                    if voltage > PART_REFRESH_POWER {
                        self.change(DisplayState::UltraFastOperating((None, None, false)));
                        return Some(false)
                    }
                },
                UpdateRequest::UltraFastSelective => {
                    if voltage > PART_REFRESH_POWER {
                        self.change(DisplayState::UltraFastOperating((None, None, true)));
                        return Some(false)
                    }
                },
                UpdateRequest::Part(r) => {
                    if voltage > PART_REFRESH_POWER {
                        let part_options = Some(refreshable_area_address(*r));
                        self.change(DisplayState::UltraFastOperating((None, part_options, true)));
                        return Some(false)
                    }
                },
                _ => {}
            }
            return None
        }
        Some(true)
    }
}

impl FrameBuffer {
    /// Create new virtual display and fill it with ON pixels
    pub fn new_white() -> Self {
        Self {
            data: bitarr!(u8, Msb0; 1; SCREEN_RESOLUTION as usize),
        }
    }

    /// Send display data to real EPD; invokes full screen refresh
    ///
    /// this is for cs environment; do not use otherwise
    pub fn apply(&self, peripherals: &mut Peripherals) {
        epaper_draw_stuff_differently(peripherals, self.data.into_inner());
    }
}

/// Display's updating progress
///
/// This is intentionally done without typestates, as typesafety it offers is outweighted by
/// reallocations made in new item creation.
pub enum DisplayState {
    /// Initial state, where we can change framebuffer. If this was typestate, this would be Zero.
    IdleOrPending,
    /// Slow update was requested; waiting for power
    FullOperating(Option<Request<UpdateFull>>),
    /// Fast update was requested; waiting for power
    FastOperating(Option<Request<UpdateFast>>),
    /// Part update was requested; waiting for power
    UltraFastOperating((Option<Request<UpdateUltraFast>>, Option<Bounds>, bool)),
    /// Display not available due to update cycle
    End,
}

impl Default for DisplayState {
    fn default() -> Self { DisplayState::IdleOrPending }
}

impl AsyncOperation for FrameBuffer {
    type Init = ();
    type Input<'a> = (i32, &'a mut DisplayOperationThreads);
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self::new_white()
    }

    /// Move through display update progress
    fn advance<'a>(&mut self, (voltage, threads): Self::Input<'a>) -> Self::Output {
        match threads.turn() {
            DisplayState::IdleOrPending => {
                let r = threads.request(voltage);
                if r == Some(false) {
                    threads.next = None;
                }
                return r
            },
            DisplayState::FullOperating(state) => {
                match state {
                    None => {
                        threads.change(DisplayState::FullOperating(Some(Request::<UpdateFull>::new((None, None)))));
                        threads.last_black =true;
                        threads.selective_counter = 0;
                        Some(false)
                    },
                    Some(a) => {
                        let r = a.advance(&self.data.data);
                        if r == Some(true) {
                            threads.change(DisplayState::End);
                            return Some(false)
                        }
                        r
                    }
                }
            },
            DisplayState::FastOperating(state) => {
                match state {
                    None => {
                        threads.change(DisplayState::FastOperating(Some(Request::<UpdateFast>::new((None, None)))));
                        threads.last_black =true;
                        threads.selective_counter = 0;
                        Some(false)
                    },
                    Some(a) => {
                        let r = a.advance(&self.data.data);
                        if r == Some(true) {
                            threads.change(DisplayState::End);
                            return Some(false)
                        }
                        r
                    }
                }
            },
            DisplayState::UltraFastOperating((state, part_options, selective_refresh)) => {
                match state {
                    None => {
                        let p = part_options.take();
                        let selective = selective_refresh.to_owned();
                        let r = if selective && threads.selective_counter < SEQUENCIAL_SELECTIVE_LIMIT {
                            threads.selective_counter += 1;
                            threads.last_black = !threads.last_black;
                            Some(!threads.last_black)
                        } else {
                            threads.selective_counter = 0;
                            threads.last_black = true;
                            None
                        };
                        threads.change(DisplayState::UltraFastOperating((Some(Request::<UpdateUltraFast>::new((p, r))), None, false)));
                        Some(false)
                    },
                    Some(a) => {
                        let r = a.advance(&self.data.data);
                        if r == Some(true) {
                            threads.change(DisplayState::End);
                            return Some(false)
                        }
                        r
                    }
                }
            },
            DisplayState::End => {
                enable_touch_int();
                threads.change(DisplayState::IdleOrPending);
                Some(false)
            },
        }
    }
}

impl Dimensions for FrameBuffer {
    fn bounding_box(&self) -> Rectangle {
            Rectangle {
                top_left: SCREEN_ZERO,
                size: SCREEN_SIZE,
            }
    }
}

// this was an experiment to find Y offset value in memory
//const SHIFT_COEFFICIENT: usize = (SCREEN_SIZE_Y * 7) as usize;

impl DrawTarget for FrameBuffer {
    type Color = BinaryColor;
    type Error = DisplayError;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            if (pixel.0.x<0)|(pixel.0.x>=SCREEN_SIZE_X as i32) {continue}
            if (pixel.0.y<0)|(pixel.0.y>=SCREEN_SIZE_Y as i32) {continue}
            //transposing pizels correctly here
            let n = (pixel.0.y + pixel.0.x*SCREEN_SIZE_Y as i32) /*(pixel.0.y*176 + (175 - pixel.0.x))*/ as usize;
            //let n = if n<SHIFT_COEFFICIENT { n + SCREEN_SIZE_VALUE - SHIFT_COEFFICIENT } else { n - SHIFT_COEFFICIENT };
            let mut pixel_update = self.data.get_mut(n).expect("checked the bounds");
            match pixel.1 {
                BinaryColor::Off => {
                    *pixel_update = true; //white
                },
                BinaryColor::On => {
                    *pixel_update = false; //black
                }
            }
        }
        Ok(())
    }
}

pub fn draw_qr(peripherals: &mut Peripherals, data_to_qr: &[u8]) {

    let len = data_to_qr.len();

    let mut outbuffer = [0u8; Version::new(18).buffer_len()].to_vec();
    let mut dataandtemp = [0u8; Version::new(18).buffer_len()].to_vec();
    
    dataandtemp[..len].copy_from_slice(data_to_qr);
    
    let qr_code = QrCode::encode_binary(&mut dataandtemp, len, &mut outbuffer, QrCodeEcc::Low, Version::MIN, Version::new(18), None, true).unwrap();

    let scaling = {
        if qr_code.version() == Version::new(18) {2}
        else {SCREEN_SIZE_Y as i32/qr_code.size()}
    };

    let mut buffer = FrameBuffer::new_white();

    let size = qr_code.size() * scaling;
    for y in 0..size {
        for x in 0..size {
            let color = {
                if qr_code.get_module(x / scaling, y / scaling) {BinaryColor::On}
                else {BinaryColor::Off}
            };
            let x_point = SCREEN_SIZE_X as i32/2 - size/2 + x;
            let y_point = SCREEN_SIZE_Y as i32/2 - size/2 + y;
            let point = Point::new(x_point, y_point);
            let pixel = Pixel::<BinaryColor>(point, color);
            pixel.draw(&mut buffer).unwrap();
        }
    }
    buffer.apply(peripherals);
}
