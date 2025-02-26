#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![deny(unused_crate_dependencies)]

extern crate alloc;
extern crate core;

use alloc::{borrow::ToOwned, boxed::Box, format};
use core::{alloc::Layout, cell::RefCell, ops::DerefMut, panic::PanicInfo, ptr::addr_of};
use cortex_m::{interrupt::{free, Mutex}, asm::delay};
use cortex_m_rt::{entry, exception, ExceptionFrame};

use embedded_alloc::Heap;
use lazy_static::lazy_static;

use kampela_system::{
    debug_display::burning_tank, devices::{power::ADC, touch::{clear_touch_if, enable_touch_int, is_touch_int, Read, FT6X36_REG_NUM_TOUCHES, LEN_NUM_TOUCHES}}, init::init_peripherals, parallel::{AsyncOperation, Threads}, NfcXfer, NfcXferBlock, BUF_THIRD, CH_TIM0, CORE_PERIPHERALS, LINK_1, LINK_2, LINK_DESCRIPTORS, PERIPHERALS, TIMER0_CC0_ICF
};
use efm32pg23_fix::{interrupt, Interrupt, Peripherals, NVIC, SYST};

mod ui;
use ui::{UIOperationThreads, UI};
mod nfc;
use nfc::{BufferStatus, NfcReceiver, NfcStateOutput, NfcResult, NfcError};
mod touch;
use touch::Touches;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use core::mem::MaybeUninit;
const HEAP_SIZE: usize = 0x6500;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

unsafe fn init_heap() {
    HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE)
}

lazy_static!{
    #[derive(Debug)]
    static ref BUFFER_STATUS: Mutex<RefCell<BufferStatus>> = Mutex::new(RefCell::new(BufferStatus::new()));
}

/*
static mut GPIO_ODD_INT: bool = false;
static mut COUNT_ODD: bool = false;
static mut GPIO_EVEN_INT: bool = false;
static mut COUNT_EVEN: bool = false;
static mut READER: Option<[u8;5]> = None;
*/

#[alloc_error_handler]
fn oom(l: Layout) -> ! {
    panic!("out of memory: {:?}, heap used: {}, free: {}", l, HEAP.used(), HEAP.free());
}

#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    let mut peripherals = unsafe{Peripherals::steal()};
    unsafe { init_heap(); } // free up heap for critical drawing buffer
    burning_tank(&mut peripherals, format!("{:?}", panic));
    loop {}
}

#[exception]
unsafe fn HardFault(exception_frame: &ExceptionFrame) -> ! {
    panic!("hard fault: {:?}", exception_frame)
}

#[interrupt]
fn LDMA() {
    free(|cs| {
        if let Some(ref mut peripherals) = PERIPHERALS.borrow(cs).borrow_mut().deref_mut() {
            peripherals.ldma_s.if_().reset();
            let mut buffer_status = BUFFER_STATUS.borrow(cs).borrow_mut();
            match buffer_status.pass_if_done7() {
                Ok(_) => {
                    if !buffer_status.is_write_halted() {
                        peripherals.ldma_s.linkload().write(|w_reg| unsafe { w_reg.linkload().bits(1 << CH_TIM0) });
                    }
                },
                Err(_) => {}
            }
        }
        else {panic!("can not borrow peripherals in ldma interrupt")}
    });
}

#[entry]
fn main() -> ! {
    unsafe { init_heap(); }

    let nfc_buffer: [u16; 3*BUF_THIRD] = [1; 3*BUF_THIRD];

    let nfc_transfer_block = NfcXferBlock {
        block0: NfcXfer {
            descriptors: LINK_DESCRIPTORS,
            source: TIMER0_CC0_ICF,
            dest: addr_of!(nfc_buffer[0]) as u32,
            link: LINK_1,
        },
        block1: NfcXfer {
            descriptors: LINK_DESCRIPTORS,
            source: TIMER0_CC0_ICF,
            dest: addr_of!(nfc_buffer[BUF_THIRD]) as u32,
            link: LINK_1,
        },
        block2: NfcXfer {
            descriptors: LINK_DESCRIPTORS,
            source: TIMER0_CC0_ICF,
            dest: addr_of!(nfc_buffer[2*BUF_THIRD]) as u32,
            link: LINK_2,
        },
    };

    let mut peripherals = Peripherals::take().unwrap();

    init_peripherals(&mut peripherals, addr_of!(nfc_transfer_block));

    delay(1000);

    free(|cs| {
        PERIPHERALS.borrow(cs).replace(Some(peripherals));
    });

    delay(1000);
    
    free(|cs| {
        let mut core_periph = CORE_PERIPHERALS.borrow(cs).borrow_mut();
        // Errata CUR_E302 fix
        // enable FPU to reduce power consumption in EM1
        unsafe {
            core_periph.SCB.cpacr.modify(|w_reg| w_reg | (3 << 20) | (3 << 22));
        }

        NVIC::unpend(Interrupt::LDMA);
        NVIC::mask(Interrupt::LDMA);
        unsafe {
            core_periph.NVIC.set_priority(Interrupt::LDMA, 3);
            NVIC::unmask(Interrupt::LDMA);
        }
    });

    //let pair_derived = Keypair::from_bytes(ALICE_KAMPELA_KEY).unwrap();

    // Development: erase seed when Pilkki can't
  
/*
    in_free(|peripherals| {
            flash_wakeup(peripherals);

            flash_unlock(peripherals);
            flash_erase_page(peripherals, 0);
            flash_wait_ready(peripherals);
    });
*/

    // hard derivation
    //let junction = DeriveJunction::hard("kampela");
    // let pair_derived = pair
    //         //.hard_derive_mini_secret_key(Some(ChainCode(*junction.inner())), b"")
    //         .0
    //         .expand_to_keypair(ExpansionMode::Ed25519);

    // initialize SYST for Timer
    free(|cs| {  
        let mut core_periph = CORE_PERIPHERALS.borrow(cs).borrow_mut();
        core_periph.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        core_periph.SYST.set_reload(SYST::get_ticks_per_10ms());
        core_periph.SYST.clear_current();
        core_periph.SYST.enable_counter();
    });

    let mut main_state = MainState::new(&nfc_buffer);
    loop {
        main_state.advance(());
    }
}

enum MainStatus<'a> {
    ADCProbe,
    NFCRead(NfcReceiver<'a>),
    Display(Option<UIOperationThreads>, Box<UI>),
    TouchRead(Option<Read<LEN_NUM_TOUCHES, FT6X36_REG_NUM_TOUCHES>>),
}

impl<'a> Default for MainStatus<'a> {
    fn default() -> Self {
        MainStatus::ADCProbe
    }
}

struct MainState<'a> {
    threads: Threads<MainStatus<'a>, 3>,
    adc: ADC,
    ui: Option<Box<UI>>,
    touches: Touches,
}

impl<'a> AsyncOperation for MainState<'a> {
    type Init = &'a [u16; 3*BUF_THIRD];
    type Input<'b> = ();
    type Output = ();
    /// Start of UI.
    fn new(nfc_buffer: Self::Init) -> Self {
        let ui = UI::new(());
        let receiver = NfcReceiver::new(nfc_buffer);
        clear_touch_if();
        
        return Self {
            threads: Threads::from([
                MainStatus::ADCProbe,
                MainStatus::NFCRead(receiver),
            ]),
            adc: ADC::new(()),
            ui: Some(Box::new(ui)),
            touches: Touches::new()
        }
    }

    /// Call in event loop to progress through Kampela states
    fn advance(&mut self, _: ()) {
        match self.threads.turn() {
            MainStatus::ADCProbe => {
                self.adc.advance(());
            },
            MainStatus::NFCRead(receiver) => {
                if let Some(s) = receiver.advance(self.adc.read()) {
                    match s {
                        Err(e) => {
                            match e {
                                NfcError::InvalidAddress => {
                                    if let Some(ref mut u) = self.ui {
                                        u.handle_message("Invalid sender address".to_owned())
                                    }
                                }
                            }
                            if let Some(u) = self.ui.take() {
                                self.threads.change(MainStatus::Display(None, u));
                            }
                        }
                        Ok(s) => {
                            match s {
                                NfcStateOutput::Operational(i) => {
                                    if i == 1 {
                                        if let Some(ref mut u) = self.ui {
                                            u.handle_message("Receiving NFC packets...".to_owned());
                                        }
                                        if !self.threads.is_all_running(&[
                                            |s| matches!(s, MainStatus::Display(..))
                                        ]) {
                                            if let Some(u) = self.ui.take() {
                                                self.threads.wind(MainStatus::Display(None, u));
                                            }
                                        };
                                    }
                                },
                                NfcStateOutput::Done(r) => {
                                    match r {
                                        NfcResult::Empty => {
                                            if !self.threads.is_all_running(&[
                                                |s| matches!(s, MainStatus::Display(..))
                                            ]) {
                                                if let Some(u) = self.ui.take() {
                                                    self.threads.wind(MainStatus::Display(None, u));
                                                }
                                            };
                                        },
                                        NfcResult::DisplayAddress => {
                                            self.threads.try_change_any(|status| {
                                                if let MainStatus::Display(_, ui) = status {
                                                    ui.handle_address([0;76]);
                                                }
                                            });
                                        },
                                        NfcResult::Transaction(transaction) => {
                                            self.threads.try_change_any(|status| {
                                                if let MainStatus::Display(_, ui) = status {
                                                    ui.handle_transaction(transaction.clone());
                                                }
                                            });
                                        }
                                    }
                                    enable_touch_int();
                                    self.threads.change(MainStatus::TouchRead(None));
                                }
                            }
                        }
                    }
                }
            },
            MainStatus::Display(state, ui) => {
                match state {
                    None => {
                        *state = Some(UIOperationThreads::new());
                    }
                    Some(t) => {
                        if ui.advance((self.adc.read(), &mut self.touches, t)) == Some(false) {
                            self.threads.hold();
                        }
                    }
                }
            },
            MainStatus::TouchRead(state) => {
                match state {
                    None => {
                        if is_touch_int() {
                            self.threads.change(MainStatus::TouchRead(Some(Read::new(()))));
                        }
                    },
                    Some(reader) => {
                        match reader.advance(()) {
                            Ok(Some(Some(touch))) => {
                                self.touches.try_push_touch_data(touch);
                                self.threads.change(MainStatus::TouchRead(None));
                            },
                            Ok(Some(None)) => {self.threads.hold()},
                            Ok(None) => {}
                            Err(e) => panic!("{:?}", e),
                        }
                    }
                }
            },
        }
    }
}
