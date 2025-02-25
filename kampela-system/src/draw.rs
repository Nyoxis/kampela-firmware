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
    devices::display::{
        Bounds, Request
    }, parallel::{AsyncOperation, Threads}
};
use kampela_ui::uistate::UpdateRequest;
use crate::debug_display::debug_draw;

// x and y of framebuffer and display RAM address are inversed
fn refreshable_area_address(refreshable_area: &Rectangle) -> Bounds {
    let y_start_address = refreshable_area.top_left.y as usize;

    let bottom_right = refreshable_area.top_left + refreshable_area.size - Point{x: 1, y: 1};

    let y_end_address = bottom_right.y as usize;

    (y_start_address, y_end_address)
}

#[derive(Debug)]
pub enum DisplayError {}

/// These are voltage thresholds to allow screen updates;
/// for wired debug, set both well below 5000
///
//TODO tune these values for prod; something like 12k and 8k
const PART_REFRESH_POWER: i32 = 1000;

/// Virtual display data storage
type PixelData = BitArr!(for SCREEN_RESOLUTION as usize, in u8, Msb0);
pub type LinesUpdated = BitArr!(for SCREEN_SIZE_Y as usize, in u8, Msb0);

/// A virtual display that could be written to EPD simultaneously
pub struct FrameBuffer {
    data: PixelData,
    lines_updated: LinesUpdated
}

pub struct DisplayOperationThreads{
    threads: Threads<DisplayState, 1>,
    next: Option<UpdateRequest>,
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
            threads: Threads::from([]),
            next: None,
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
                UpdateRequest::Part(r) => {
                    if voltage > PART_REFRESH_POWER {
                        let part_options = Some(refreshable_area_address(r));
                        self.change(DisplayState::Operating((None, part_options)));
                        return Some(false)
                    }
                },
                _ => {
                    if voltage > PART_REFRESH_POWER {
                    self.change(DisplayState::Operating((None, None)));
                    return Some(false)
                }}
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
            lines_updated: bitarr!(u8, Msb0; 1; SCREEN_SIZE_Y as usize),
        }
    }

    /// Send display data to real EPD; invokes full screen refresh
    ///
    /// this is for cs environment; do not use otherwise
    pub fn apply(&self, peripherals: &mut Peripherals) {
        debug_draw(peripherals, self.data.into_inner());
    }
}

/// Display's updating progress
///
/// This is intentionally done without typestates, as typesafety it offers is outweighted by
/// reallocations made in new item creation.
pub enum DisplayState {
    /// Initial state, where we can change framebuffer. If this was typestate, this would be Zero.
    IdleOrPending,
    /// Part update was requested; waiting for power
    Operating((Option<Request>, Option<Bounds>)),
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
            DisplayState::Operating((state, part_options)) => {
                match state {
                    None => {
                        let p = part_options.take();
                        threads.change(DisplayState::Operating((Some(Request::new(p)), None)));
                        Some(false)
                    },
                    Some(a) => {
                        let r = a.advance((&self.data.data, &mut self.lines_updated));
                        if r == Some(true) {
                            threads.change(DisplayState::End);
                            return Some(false)
                        }
                        r
                    }
                }
            },
            DisplayState::End => {
                threads.change(DisplayState::IdleOrPending);
                Some(false)
            }
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
            let mut line_update = self.lines_updated.get_mut(pixel.0.y as usize).expect("checked the bounds");
            *line_update = true;
            //transposing pizels correctly here
            let n = (pixel.0.x + pixel.0.y*SCREEN_SIZE_X as i32) /*(pixel.0.y*176 + (175 - pixel.0.x))*/ as usize;
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
