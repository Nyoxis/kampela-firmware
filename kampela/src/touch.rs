use alloc::{collections::vec_deque::VecDeque, borrow::ToOwned};
use nalgebra::{Affine2, OMatrix, Point2, RowVector3};
use lazy_static::lazy_static;
use embedded_graphics::prelude::Point;

use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};

use kampela_system::{devices::touch::LEN_NUM_TOUCHES, in_free, if_in_free};
use kampela_ui::display_def::*;
use efm32pg23_fix::interrupt;

pub const MAX_TOUCH_QUEUE: usize = 2;

lazy_static! {
    // MAGIC calibration numbers obtained through KOLIBRI tool
    static ref AFFINE_MATRIX: Affine2<f32> = Affine2::from_matrix_unchecked(
        OMatrix::from_rows(&[
            RowVector3::<f32>::new(1.0022, -0.0216, -4.2725),
            RowVector3::<f32>::new(0.0061, 1.1433, -13.7305),
            RowVector3::<f32>::new(0.0, 0.0, 1.0),
        ])
    );
    // touches len needed in interrupt function, hence global
    pub static ref TOUCHES: Mutex<RefCell<VecDeque<Point>>> = Mutex::new(RefCell::new(VecDeque::with_capacity(MAX_TOUCH_QUEUE)));
    // set by interrupt function, hence global
    pub static ref TOUCH_STATUS: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
}

fn reset_touch_status() {
    free(|cs| {
        let mut touch_status = TOUCH_STATUS.borrow(cs).borrow_mut();
        *touch_status = false;
    })
}

pub fn get_touch_status() -> bool {
    free(|cs| TOUCH_STATUS.borrow(cs).borrow().to_owned())
}

#[interrupt]  // IRQ required to wake up after vfi
fn GPIO_EVEN() {  // Swtich status to start read touch
    if if_in_free(|peripherals|
        peripherals.gpio_s.if_().read().extif0().bit_is_clear()
    ).unwrap_or(true) {
        return
    }
    free(|cs| {
        let touches = TOUCHES.borrow(cs).borrow();
        if touches.len() < MAX_TOUCH_QUEUE {
            let mut status = TOUCH_STATUS.borrow(cs).borrow_mut();
            *status = true
        }
    });
    in_free(|peripherals|
        peripherals.gpio_s.if_clr().write(|w_reg| w_reg.extif0().set_bit())
    );
}

pub fn try_push_touch_data(touch_data: [u8; LEN_NUM_TOUCHES]) {
    reset_touch_status();
    if let Some(point) = convert(touch_data) {
        free(|cs| {
            let mut touches = TOUCHES.borrow(cs).borrow_mut();
            if touches.len() < MAX_TOUCH_QUEUE {
                touches.push_back(point);
            }
        })
    }
}

pub fn get_touch_point() -> Option<Point> {
    free(|cs| {
        let mut touches = TOUCHES.borrow(cs).borrow_mut();
        touches.pop_front()
    })
}

pub fn convert(touch_data: [u8; LEN_NUM_TOUCHES]) -> Option<Point> {
    if touch_data[0] == 1 {
        let detected_y = (((touch_data[1] as u16 & 0b00001111) << 8) | touch_data[2] as u16) as i32;
        let detected_x = (((touch_data[3] as u16 & 0b00001111) << 8) | touch_data[4] as u16) as i32;
        let touch = Point::new(SCREEN_SIZE_X as i32 - detected_x, detected_y);

        let touch_as_point2 = Point2::new(touch.x as f32, touch.y as f32);
        let display_as_point2 = AFFINE_MATRIX.transform_point(&touch_as_point2);

        Some(
            Point {
                x: display_as_point2.coords[0] as i32,
                y: display_as_point2.coords[1] as i32,
            }
        )
    } else { None }
}

/*
pub struct TouchState {
    state: touch::Read,
}

impl TouchState {
    pub fn new(&mut self) -> Self {
        Self {
            state: touch::Read::new();
        }
    }

    pub fn query_touch(&mut self, peripherals: &mut Peripherals) -> Option<Point> {
        
        if let Some(touch_data) = self.advance() {
            peripherals
                .gpio_s
                .if_
                .write(|w_reg| w_reg.extif0().clear_bit());
            if touch_data[0] == 1 {
                    let detected_y = (((touch_data[1] as u16 & 0b00001111) << 8) | touch_data[2] as u16) as i32;
                    let detected_x = (((touch_data[3] as u16 & 0b00001111) << 8) | touch_data[4] as u16) as i32;
                    let touch = Point::new(SCREEN_SIZE_X as i32 - detected_x, detected_y);

                    let touch_as_point2 = Point2::new(touch.x as f32, touch.y as f32);
                    let display_as_point2 = affine_matrix.transform_point(&touch_as_point2);
            
                    Some( Point {
                        x: display_as_point2.coords[0] as i32,
                        y: display_as_point2.coords[1] as i32,
                    })
            } else { None }
        } else { None }
    }

    fn advance(&mut self) -> Option<[u8; LEN_NUM_TOUCHES]> {
free(|cs| {
            if let Some(ref mut peripherals) = PERIPHERALS.borrow(cs).borrow_mut().deref_mut() {
                if peripherals.gpio_s.if_.read().extif0().bit_is_set() {
                    self.status = UIStatus::TouchState(ft6336_read_at::<LEN_NUM_TOUCHES>(peripherals, FT6X36_REG_NUM_TOUCHES).unwrap());
                }
            }
        });

        None
    }
}
*/