use crate::{if_in_free, in_free};
use crate::parallel::{AsyncOperation, Threads};

use kampela_display_common::display_def::*;
pub const X_ADDRESS_WIDTH: usize = (SCREEN_SIZE_X / 8) as usize;
pub const DATA_UPDATE_MODE: u16 = 0b100 << 13;
pub const HOLD_MODE: u16 = 0b000 << 13;
pub const ALL_CLEAR_MODE: u16 = 0b001 << 13;

/// Send data array
pub enum SharpDataWritingState {
    /// Send byte
    Send,
    /// Receive something to keep protocol running and close connection
    WaitSend,

    End,
}
impl Default for SharpDataWritingState {
    fn default() -> Self { SharpDataWritingState::End }
}

pub struct SharpDataWriting<const LEN: usize>{
    threads: Threads<SharpDataWritingState, 1>,
    position: usize,
    end_position: usize,
}

impl <const LEN: usize> AsyncOperation for SharpDataWriting<LEN> {
    type Init = Option<usize>; // Borders of 2D array
    type Input<'a> = &'a [u8; LEN];
    type Output = Option<bool>;

    fn new(gateline_option: Self::Init) -> Self {
        let (position, end_position) = match gateline_option {
            None => {(
                0,
                LEN - 1
            )},
            Some(gateline) => {(
                gateline * X_ADDRESS_WIDTH,
                (gateline + 1) * X_ADDRESS_WIDTH - 1
            )}
        };
        Self {
            threads: Threads::new(SharpDataWritingState::Send),
            position,
            end_position,
        }
    }

    fn advance(&mut self, data: Self::Input<'_>) -> Self::Output {
        match self.threads.turn() {
            SharpDataWritingState::Send => {
                if if_in_free(|peripherals|
                    peripherals.usart0_s.status().read().txbl().bit_is_set()
                ) != Ok(true) {
                    return None
                }
                in_free(|peripherals|
                    peripherals
                        .usart0_s
                        .txdata()
                        .write(|w_reg| unsafe { w_reg.txdata().bits(data[self.position]) })
                );
                self.threads.change(SharpDataWritingState::WaitSend);
                Some(false)
            },
            SharpDataWritingState::WaitSend => {
                if if_in_free(|peripherals|
                    peripherals
                        .usart0_s
                        .status()
                        .read()
                        .txc()
                        .bit_is_set()
                ) != Ok(true) {
                    return None
                }
                in_free(|peripherals| {
                    peripherals
                        .usart0_s
                        .rxdata()
                        .read()
                        .rxdata()
                        .bits();
                });
                if self.position < self.end_position {
                    self.position += 1;
                    self.threads.change(SharpDataWritingState::Send);
                } else {
                    self.threads.change(SharpDataWritingState::End);
                }
                Some(false)
            },
            SharpDataWritingState::End => {
                Some(true)
            },
        }
    }
}