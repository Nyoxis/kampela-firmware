use alloc::collections::vec_deque::VecDeque;
use nalgebra::{Affine2, OMatrix, Point2, RowVector3};
use lazy_static::lazy_static;
use embedded_graphics::prelude::Point;

use kampela_system::devices::touch::{clear_touch_if, LEN_NUM_TOUCHES};
use kampela_ui::display_def::*;

pub const MAX_TOUCH_QUEUE: usize = 2;

lazy_static! {
    // MAGIC calibration numbers obtained through KOLIBRI tool
    static ref AFFINE_MATRIX: Affine2<f32> = Affine2::from_matrix_unchecked(
        OMatrix::from_rows(&[
            RowVector3::<f32>::new(1.4588, -0.0164, -195.5833),
            RowVector3::<f32>::new(-0.1546, 1.5155, 30.7841),
            RowVector3::<f32>::new(0.0, 0.0, 1.0),
        ])
    );
}

pub struct Touches(VecDeque<Point>);

impl Touches {
    pub fn new() -> Self {
        Self(VecDeque::with_capacity(MAX_TOUCH_QUEUE))
    }

    pub fn try_push_touch_data(&mut self, touch_data: [u8; LEN_NUM_TOUCHES]) -> bool {
        clear_touch_if();
        if self.0.len() < MAX_TOUCH_QUEUE {
            if let Some(point) = convert(touch_data) {
                self.0.push_back(point);
                return true
            }
        }
        false
    }
    
    pub fn take_touch_point(&mut self) -> Option<Point> {
        self.0.pop_front()
    }
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
fn blocking_touch_read() -> Point {
    let mut state: Option<Read<LEN_NUM_TOUCHES, FT6X36_REG_NUM_TOUCHES>> = None;
    clear_touch_if();
    let touch_data = loop {
        match &mut state {
            None => {
                if is_touch_int() {
                    state = Some(Read::new(()));
                }
            },
            Some(reader) => {
                match reader.advance(()) {
                    Ok(Some(Some(touch))) => {
                        break touch;
                    },
                    Ok(Some(None)) => {},
                    Ok(None) => {}
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
    };

    let detected_y = (((touch_data[1] as u16 & 0b00001111) << 8) | touch_data[2] as u16) as i32;
    let detected_x = (((touch_data[3] as u16 & 0b00001111) << 8) | touch_data[4] as u16) as i32;
    Point::new(SCREEN_SIZE_X as i32 - detected_x, detected_y)
}

pub fn kolibri_test() {
    // Prepare
    let mut display = FrameBuffer::new_white();

    let mut rng = se_rng::SeRng{};

    let mut state = UIState::init(&mut rng);

    let mut do_update = true;
    loop {
        if do_update {
            state.render(&mut display).unwrap();
            do_update = false;
        }
        in_free(|peripherals| {
            display.apply(peripherals);
        });

        let point = blocking_touch_read();
        do_update = state.process_touch(point, &mut rng).unwrap();
    }
}
*/