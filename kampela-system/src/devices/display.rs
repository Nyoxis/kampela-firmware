//! display control functions
use crate::draw::LinesUpdated;
use crate::in_free;
use crate::parallel::{AsyncOperation, Threads, Timer};
use crate::devices::display_transmission::{SharpDataWriting, DATA_UPDATE_MODE};
use crate::peripherals::usart::{select_display, deselect_display};
use kampela_ui::display_def::*;

pub type Bounds = (usize, usize);

/// Draw sequence
///
/// Iterate through this to perform drawing and send display to proper sleep mode
pub struct Request {
    threads: Threads<RequestState, 1>,
    gateline: usize,
    end_gateline: usize,
}

enum RequestState {
    Init(Option<Timer>),
    DataTransferWithModeSelection(Option<(SharpDataWriting<2>, [u8; 2])>),
    DataTransfer(Option<(SharpDataWriting<2>, [u8; 2])>),
    DataWriting(Option<SharpDataWriting<SCREEN_BUFFER_SIZE>>),
    End,
    Error,
}

impl Default for RequestState {
    fn default() -> Self { RequestState::Error }
}

impl Request {
    fn inc_gateline(&mut self, lines_updated: &mut LinesUpdated) -> bool {
        while {
            self.gateline += 1;
            if self.gateline > self.end_gateline {
                return false
            }
            let mut line_updated = lines_updated.get_mut(self.gateline).expect("checked end gateline");
            let r = line_updated.replace(false);
            !r
        } {}
        true
    }

    fn data_transfer(&self) -> u16 {
        ((0x00FF & (self.gateline + 1) as u16) << 8).reverse_bits()
    }
}

impl AsyncOperation for Request {
    type Init = Option<Bounds>;
    type Input<'a> = (&'a [u8; SCREEN_BUFFER_SIZE], &'a mut LinesUpdated);
    type Output = Option<bool>;

    fn new(part_options: Self::Init) -> Self {
        let (gateline, end_gateline) = match part_options {
            None => {(
                0,
                SCREEN_SIZE_Y as usize - 1,
            )},
            Some(part) => {
                part
            }
        };

        Self {
            threads: Threads::new(RequestState::Init(None)),
            gateline,
            end_gateline
        }
    }

    fn advance(&mut self, (data, lines_updated): Self::Input<'_>) -> Self::Output {
        match self.threads.turn() {
            RequestState::Init(state) => {
                match state {
                    None => {
                        in_free(|peripherals| 
                            select_display(&mut peripherals.gpio_s)
                        );
                        self.threads.change(RequestState::Init(Some(Timer::new(2))));
                    },
                    Some(t) => {
                        if t.tick() { return None };
                        self.threads.change(RequestState::DataTransferWithModeSelection(None));
                    }
                }
                Some(false)
            },
            RequestState::DataTransferWithModeSelection(state) => {
                match state {
                    None => {
                        let data_transfer = DATA_UPDATE_MODE | self.data_transfer();
                        let data_transfer_bytes = data_transfer.to_be_bytes();
                        self.threads.change(RequestState::DataTransferWithModeSelection(Some((SharpDataWriting::new(None), data_transfer_bytes))));
                    },
                    Some((a, bytes)) => {
                        match a.advance(bytes) {
                            Some(true) => {
                                self.threads.change(RequestState::DataWriting(None));
                            },
                            r => return r
                        };
                    }
                }
                Some(false)
            },
            RequestState::DataTransfer(state) => {
                match state {
                    None => {
                        let data_transfer = self.data_transfer();
                        let data_transfer_bytes = data_transfer.to_be_bytes();
                        self.threads.change(RequestState::DataTransfer(Some((SharpDataWriting::new(None), data_transfer_bytes))));
                    },
                    Some((a, bytes)) => {
                        match a.advance(bytes) {
                            Some(true) => {
                                self.threads.change(RequestState::DataWriting(None));
                            },
                            r => return r
                        };
                    }
                }
                Some(false)
            },
            RequestState::DataWriting(state) => {
                match state {
                    None => {
                        self.threads.change(RequestState::DataWriting(Some(SharpDataWriting::new(Some(self.gateline)))));
                    },
                    Some(a) => {
                        match a.advance(data) {
                            Some(true) => {
                                if self.inc_gateline(lines_updated) {
                                    self.threads.change(RequestState::DataTransfer(None));
                                    return None
                                }
                                self.threads.change(RequestState::End);
                            },
                            r => return r
                        };
                    }
                }
                Some(false)
            },
            RequestState::End => {
                in_free(|peripherals| 
                    deselect_display(&mut peripherals.gpio_s)
                );
                Some(true)
            },
            RequestState::Error => {
                panic!("Unknown RequestState while display")
            }
        }
    }
}
