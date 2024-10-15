//! display control functions

use crate::peripherals::usart::*;
use crate::peripherals::gpio_pins::{display_res_clear, display_res_set};
use crate::in_free;
use crate::parallel::{AsyncOperation, Threads, Timer, WithDelay, DELAY};
use crate::devices::display_transmission::{display_is_busy, EPDCommand, EPDDataB, EPDData, EPDDataBuffer, BUFSIZE};

const X_START_BOUNDS: u8 = 0x00;
const Y_START_BOUNDS: u16 = 0x00;

const LUT: [u8; 0x99] = [ //not working
    0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUT 0
    0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUT 1
    0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUT 2
    0xAA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUT 3
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // LUT 4
//  TPA   TPB   SRAB  TPC   TPD   SRCD  RP
    0x10, 0x10, 0x00, 0x10, 0x10, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    
    0x10, 0x00, 0x00, 0x00, 0x00, 0x00, // FR
    0x00, 0x00, 0x00,                   // XON
];


/// Draw sequence
///
/// Iterate through this to perform drawing and send display to proper sleep mode
pub struct Request<R> where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    > 
{
    threads: Threads<RequestState<R>, 1>,
    bounds: <PrepareDraw<R> as AsyncOperation>::Init,
}

pub enum RequestState<R> where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    > 
{
    Init(Option<EPDInit>),
    Draw(Option<PrepareDraw<R>>),
    Error,
}

impl<R> Default for RequestState<R> where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    >
{
    fn default() -> Self { RequestState::<R>::Error }
}

impl<R> AsyncOperation for Request<R> where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    >
{
    type Init = <PrepareDraw<R> as AsyncOperation>::Init;
    type Input<'a> = <PrepareDraw<R> as AsyncOperation>::Input<'a>;
    type Output = Option<bool>;

    fn new(bounds: Self::Init) -> Self {
        Self {
            threads: Threads::new(RequestState::Init(None)),
            bounds,
        }
    }

    fn advance(&mut self, data: Self::Input<'_>) -> Self::Output {
        match self.threads.advance_state() {
            RequestState::Init(state) => {
                match state {
                    None => {
                        self.threads.change(RequestState::Init(Some(EPDInit::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(RequestState::Draw(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            RequestState::Draw(state) => {
                match state {
                    None => {
                        self.threads.change(RequestState::Draw(Some(PrepareDraw::<R>::new(self.bounds.take()))));
                        Some(false)
                    },
                    Some(a) => {
                        a.advance(data)
                    }
                }
            },
            RequestState::Error => {
                panic!("Unknown RequestState while display")
            }
        }
    }
}




/// EPD init to wake up display
pub struct EPDInit {
    threads: Threads<EPDInitState, 1>,
}

pub enum EPDInitState {
    Reset(Option<Reset>),
    WakeUp(Option<WithDelay<EPDCommand<0x12>>>),
    End,
    Error,
}

impl Default for EPDInitState {
    fn default() -> Self { EPDInitState::Error }
}

impl AsyncOperation for EPDInit {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(EPDInitState::Reset(None)),
        }
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.advance_state() {
            EPDInitState::Reset(state) => {
                match state {
                    None => {
                        self.threads.change(EPDInitState::Reset(Some(Reset::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(EPDInitState::WakeUp(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            EPDInitState::WakeUp(state) => {
                match state {
                    None => {
                        self.threads.change(EPDInitState::WakeUp(Some(WithDelay::Do(EPDCommand::new(())))));
                    },
                    Some(w) => {
                        match w {
                            WithDelay::Do(a) => {
                                match a.advance(()) {
                                    Some(true) => {
                                        self.threads.change(EPDInitState::WakeUp(Some(WithDelay::Wait(Timer::new(DELAY)))));
                                    },
                                    r => return r
                                };
                            },
                            WithDelay::Wait(t) => {
                                if t.tick() {
                                    return None
                                }
                                self.threads.change(EPDInitState::End);
                                return Some(true)
                            },
                        }
                    }
                }
                Some(false)
            },
            EPDInitState::End => {
                Some(true)
            }
            EPDInitState::Error => {
                panic!("Unknown EPDInitState while display")
            },
        }
    }
}

/// Reset display
///
/// notably used for waking up
pub struct Reset {
    threads: Threads<ResetState, 1>,
}

pub enum ResetState {
    R0(Option<Timer>),
    R1(Option<Timer>),
    R2,
    Error,
}

impl Default for ResetState {
    fn default() -> Self { ResetState::Error }
}

impl AsyncOperation for Reset {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(ResetState::R0(None)),
        }
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.advance_state() {
            ResetState::R0(state) => {
                match state {
                    None => {
                        in_free(|peripherals| {
                            deselect_display(&mut peripherals.gpio_s);
                            display_res_set(&mut peripherals.gpio_s);
                        });
                        self.threads.change(ResetState::R0(Some(Timer::new(DELAY))));
                    },
                    Some(t) => {
                        if t.tick() {
                            return None
                        }
                        self.threads.change(ResetState::R1(None));
                    }
                }
                Some(false)
            },
            ResetState::R1(state) => {
                match state {
                    None => {
                        in_free(|peripherals| {
                            display_res_clear(&mut peripherals.gpio_s);
                        });
                        self.threads.change(ResetState::R1(Some(Timer::new(DELAY))));
                    },
                    Some(t) => {
                        if t.tick() {
                            return None
                        }
                        self.threads.change(ResetState::R2);
                    }
                }
                Some(false)
            },
            ResetState::R2 => {
                Some(true)
            },
            ResetState::Error => {
                panic!("Unknown ResetState while display")
            },
        }
    }
}


pub trait RequestType: AsyncOperation {}
impl RequestType for UpdateFast {}
impl RequestType for UpdateFull {}
impl RequestType for UpdateUltraFast {}

pub struct PrepareDraw<R> where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    >
{
    threads: Threads<PrepareDrawState<R>, 1>,
    bounds: <PrepareDraw<R> as AsyncOperation>::Init,
}

pub enum PrepareDrawState<R> {
    //Set RAM X address start/end postition (which is Y due to orientation)
    PrepareC2(Option<EPDCommand<0x44>>),
    PrepareD21(Option<EPDData<1>>),
    PrepareD22(Option<EPDData<1>>),
    //Set RAM Y address start/end postition (which is X due to orientation)
    PrepareC3(Option<EPDCommand<0x45>>),
    PrepareD31(Option<EPDData<2>>),
    PrepareD32(Option<EPDData<2>>),
    //Set RAM X&Y address write starting position
    PrepareC4(Option<EPDCommand<0x4E>>),
    PrepareD4(Option<(EPDData<1>, [u8; 1])>),
    PrepareC5(Option<EPDCommand<0x4F>>),
    PrepareD5(Option<(EPDData<2>, [u8; 2])>),

    SendC1(Option<EPDCommand<0x24>>),
    SendD1(Option<EPDDataBuffer<BUFSIZE>>),

    Update(Option<R>),

    Error,
}

impl<R> Default for PrepareDrawState<R> {
    fn default() -> Self { PrepareDrawState::<R>::Error }
}

impl<R> AsyncOperation for PrepareDraw<R>  where
    R: for <'a> RequestType<
        Init = (),
        Input<'a> = (),
        Output = Option<bool>,
    >
{
    type Init = <EPDDataBuffer::<BUFSIZE> as AsyncOperation>::Init;
    type Input<'a> = &'a [u8];
    type Output = Option<bool>;

    fn new(bounds: Self::Init) -> Self {
        let init_state = match bounds {
            //skip bounds addresses transmission
            None => {
                PrepareDrawState::PrepareC4(None)
            },
            Some(_) => {
                PrepareDrawState::PrepareC2(None)
            }
        };
        Self {
            threads: Threads::new(init_state),
            bounds,
        }
    }

    fn advance(&mut self, data: Self::Input<'_>) -> Self::Output {
        match self.threads.advance_state() {
            PrepareDrawState::PrepareC2(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareC2(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD21(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD21(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareD21(Some(EPDData::new(()))));
                    },
                    Some(a) => {
                        match a.advance(&self.bounds.unwrap().0.to_le_bytes()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD22(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD22(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareD22(Some(EPDData::new(()))));
                    },
                    Some(a) => {
                        match a.advance(&self.bounds.unwrap().1.to_le_bytes()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareC3(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareC3(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareC3(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD31(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD31(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareD31(Some(EPDData::new(()))));
                    },
                    Some(a) => {
                        match a.advance(&self.bounds.unwrap().2.to_le_bytes()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD32(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD32(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareD32(Some(EPDData::new(()))));
                    },
                    Some(a) => {
                        match a.advance(&self.bounds.unwrap().3.to_le_bytes()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareC4(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareC4(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareC4(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD4(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD4(state) => {
                match state {
                    None => {
                        let x = match self.bounds {
                            None => X_START_BOUNDS.to_le_bytes(),
                            Some(b) => (X_START_BOUNDS + b.0).to_le_bytes()
                        };
                        self.threads.change(PrepareDrawState::PrepareD4(Some((EPDData::new(()), x))));
                    },
                    Some((a, x)) => {
                        match a.advance(x) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareC5(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareC5(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::PrepareC5(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::PrepareD5(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::PrepareD5(state) => {
                match state {
                    None => {
                        let y: [u8; 2] = match self.bounds {
                            None => Y_START_BOUNDS.to_le_bytes(),
                            Some(b) => (Y_START_BOUNDS + b.2).to_le_bytes()
                        };
                        self.threads.change(PrepareDrawState::PrepareD5(Some((EPDData::new(()), y))));
                    },
                    Some((a, y)) => {
                        match a.advance(y) {
                            Some(true) => {
                                if display_is_busy() == Ok(false) {
                                    self.threads.change(PrepareDrawState::SendC1(None));
                                } else {
                                    return None
                                }
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::SendC1(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::SendC1(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::SendD1(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::SendD1(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::SendD1(Some(EPDDataBuffer::<BUFSIZE>::new(self.bounds))));
                    },
                    Some(a) => {
                        match a.advance(data) {
                            Some(true) => {
                                self.threads.change(PrepareDrawState::Update(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            PrepareDrawState::Update(state) => {
                match state {
                    None => {
                        self.threads.change(PrepareDrawState::Update(Some(R::new(()))));
                        Some(false)
                    },
                    Some(a) => {
                        a.advance(())
                    }
                }
            }
            PrepareDrawState::Error => {
                panic!("Unknown PrepareDrawState while display")
            }
        }
    }
}


pub struct UpdateFull {
    threads: Threads<UpdateFullState, 1>,
}

pub enum UpdateFullState {
    // set read temperature from internal TS
    UpdateC2(Option<EPDCommand<0x18>>),
    UpdateD2(Option<EPDDataB<0x80>>),
    
    UpdateC3(Option<EPDCommand<0x22>>),
    UpdateD3(Option<EPDDataB<0xF7>>),

    UpdateC4(Option<EPDCommand<0x20>>),

    Error,
}

impl Default for UpdateFullState {
    fn default() -> Self { UpdateFullState::Error }
}

impl AsyncOperation for UpdateFull {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(UpdateFullState::UpdateC2(None)),
        }
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.advance_state() {
            UpdateFullState::UpdateC2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFullState::UpdateC2(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFullState::UpdateD2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFullState::UpdateD2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFullState::UpdateD2(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFullState::UpdateC3(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFullState::UpdateC3(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFullState::UpdateC3(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFullState::UpdateD3(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFullState::UpdateD3(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFullState::UpdateD3(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFullState::UpdateC4(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFullState::UpdateC4(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFullState::UpdateC4(Some(EPDCommand::new(()))));
                        Some(false)
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                if display_is_busy() != Ok(false) {
                                    Some(true)
                                } else {
                                    None
                                }
                            },
                            r => {r}
                        }
                    }
                }
            },
            UpdateFullState::Error => {
                panic!("Unknown UpdateFullState while display")
            }
        }
    }
}

pub struct UpdateFast {
    threads: Threads<UpdateFastState, 1>,
}

pub enum UpdateFastState {
    //set read from internal temperature sensor
    PrepareC1(Option<EPDCommand<0x18>>),
    PrepareD1(Option<EPDDataB<0x80>>),
    //set temperature register at 100deg
    PrepareC2(Option<EPDCommand<0x1A>>),
    PrepareD21(Option<EPDDataB<0x64>>),
    PrepareD22(Option<EPDDataB<0x00>>),
    // set to display with new LUT
    UpdateC2(Option<EPDCommand<0x22>>),
    UpdateD2(Option<EPDDataB<0xD7>>),

    UpdateC3(Option<EPDCommand<0x20>>),
    
    Error,
}

impl Default for UpdateFastState {
    fn default() -> Self { UpdateFastState::Error }
}

impl AsyncOperation for UpdateFast {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(UpdateFastState::PrepareC1(None)),
        }
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.advance_state() {
            UpdateFastState::PrepareC1(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::PrepareC1(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::PrepareD1(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::PrepareD1(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::PrepareD1(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::PrepareC2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::PrepareC2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::PrepareC2(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::PrepareD21(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::PrepareD21(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::PrepareD21(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::PrepareD22(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::PrepareD22(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::PrepareD22(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::UpdateC2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::UpdateC2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::UpdateC2(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::UpdateD2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::UpdateD2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::UpdateD2(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateFastState::UpdateC3(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateFastState::UpdateC3(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateFastState::UpdateC3(Some(EPDCommand::new(()))));
                        Some(false)
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                if display_is_busy() == Ok(false) {
                                    Some(true)
                                } else {
                                    None
                                }
                            },
                            r => {r}
                        }
                    }
                }
            },
            UpdateFastState::Error => {
                panic!("Unknown UpdateFastState while display")
            }
        }
    }
}

pub struct UpdateUltraFast {
    threads: Threads<UpdateUltraFastState, 1>,
}

pub enum UpdateUltraFastState {
    //inverse RED RAM (for some reason red ram still used in mode 2)
    UpdateC1(Option<EPDCommand<0x21>>),
    UpdateD11(Option<EPDDataB<0x40>>),
    UpdateD12(Option<EPDDataB<0x00>>),

    UpdateC2(Option<EPDCommand<0x22>>),
    UpdateD2(Option<EPDDataB<0x99>>),
    UpdateC3(Option<EPDCommand<0x20>>),
    // Load custom LUT
    UpdateC4(Option<EPDCommand<0x32>>),
    UpdateD4(Option<EPDData<0x9F>>),
    // Display with mode 2
    UpdateC6(Option<EPDCommand<0x22>>),
    UpdateD6(Option<EPDDataB<0xC7>>),

    UpdateC7(Option<EPDCommand<0x20>>),

    Error,
}

impl Default for UpdateUltraFastState {
    fn default() -> Self { UpdateUltraFastState::Error }
}

impl AsyncOperation for UpdateUltraFast {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(UpdateUltraFastState::UpdateC1(None)),
        }
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.advance_state() {
            UpdateUltraFastState::UpdateC1(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC1(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateD11(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateD11(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateD11(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateD12(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateD12(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateD12(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateC2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateC2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC2(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateD2(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateD2(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateD2(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateC6(None));
                                cortex_m::asm::delay(3000);
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateC3(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC3(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                if display_is_busy() == Ok(false) {
                                    self.threads.change(UpdateUltraFastState::UpdateC4(None));
                                } else {
                                    return None
                                }
                            },
                            r => {return r}
                        }
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateC4(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC4(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateD4(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateD4(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateD4(Some(EPDData::new(()))));
                    },
                    Some(a) => {
                        match a.advance(&LUT) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateC6(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateC6(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC6(Some(EPDCommand::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateD6(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateD6(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateD6(Some(EPDDataB::new(()))));
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                self.threads.change(UpdateUltraFastState::UpdateC7(None));
                            },
                            r => {return r}
                        };
                    }
                }
                Some(false)
            },
            UpdateUltraFastState::UpdateC7(state) => {
                match state {
                    None => {
                        self.threads.change(UpdateUltraFastState::UpdateC7(Some(EPDCommand::new(()))));
                        Some(false)
                    },
                    Some(a) => {
                        match a.advance(()) {
                            Some(true) => {
                                if display_is_busy() == Ok(false) {
                                    Some(true)
                                } else {
                                    None
                                }
                            },
                            r => {r}
                        }
                    }
                }
            },
            UpdateUltraFastState::Error => {
                panic!("Unknown UpdateUltraFastState while display")
            }
        }
    }
}