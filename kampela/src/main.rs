#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![deny(unused_crate_dependencies)]

extern crate alloc;
extern crate core;

use alloc::{borrow::ToOwned, format};
use core::{cell::RefCell, ops::DerefMut, ptr::addr_of, alloc::Layout, panic::PanicInfo};
use cortex_m::{interrupt::{free, Mutex}, asm::delay};
use cortex_m_rt::{entry, exception, ExceptionFrame};

use embedded_alloc::Heap;
use lazy_static::lazy_static;

use kampela_system::{
    PERIPHERALS, CORE_PERIPHERALS,
    devices::{power::ADC, touch::{Read, FT6X36_REG_NUM_TOUCHES, LEN_NUM_TOUCHES, enable_touch_int}},
    debug_display::burning_tank,
    init::init_peripherals,
    parallel::Operation,
    BUF_THIRD, CH_TIM0, LINK_1, LINK_2, LINK_DESCRIPTORS, TIMER0_CC0_ICF, NfcXfer, NfcXferBlock,
};
use efm32pg23_fix::{interrupt, Interrupt, NVIC, Peripherals};
use kampela_ui::platform::Platform;

mod ui;
use ui::UI;
mod nfc;
use nfc::{BufferStatus, NfcReceiver, NfcStateOutput, NfcResult, NfcError};
mod touch;
use touch::try_push_touch_data;

#[global_allocator]
static HEAP: Heap = Heap::empty();

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
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 0x6500;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }


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
        NVIC::unpend(Interrupt::LDMA);
        NVIC::mask(Interrupt::LDMA);
        unsafe {
            core_periph.NVIC.set_priority(Interrupt::LDMA, 3);
            NVIC::unmask(Interrupt::LDMA);
        }

        NVIC::unpend(Interrupt::GPIO_EVEN);
        NVIC::mask(Interrupt::GPIO_EVEN);
        unsafe {
            core_periph.NVIC.set_priority(Interrupt::GPIO_EVEN, 4);
            NVIC::unmask(Interrupt::GPIO_EVEN);
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

    let mut main_state = MainState::init(&nfc_buffer);
    loop {
        main_state.advance();
    }
}

lazy_static!{
    // set by interrupt function, hence global
    pub static ref DISPLAY_OR_TOUCH_STATUS: Mutex<RefCell<DisplayOrTouchStatus>> = Mutex::new(RefCell::new(DisplayOrTouchStatus::DisplayOrListen));
}

#[derive(Clone, Copy)]
pub enum DisplayOrTouchStatus {
    DisplayOrListen,
    /// Touch event processing
    TouchOperation,
}

fn set_display_or_touch_status(new_status: DisplayOrTouchStatus) {
    free(|cs| {
        let mut ui_status = DISPLAY_OR_TOUCH_STATUS.borrow(cs).borrow_mut();
        *ui_status = new_status;
    })
}

fn get_display_or_touch_status() -> DisplayOrTouchStatus {
    free(|cs| DISPLAY_OR_TOUCH_STATUS.borrow(cs).borrow().to_owned())
}

enum MainStatus<'a> {
    NFCReadOrDisplay(NFC<'a>),
    DisplayOrTouch,
}

struct MainState<'a> {
    adc: ADC,
    status: MainStatus<'a>,
    ui: UI,
    touch: Read<LEN_NUM_TOUCHES, FT6X36_REG_NUM_TOUCHES>,
}

enum NFCReadOrDisplayStatus {
    NfcRead,
    DisplayMessage,
}

struct NFC<'a> {
    receiver: NfcReceiver<'a>,
    status: NFCReadOrDisplayStatus,
}

impl<'a> MainState<'a> {
    /// Start of UI.
    pub fn init(nfc_buffer: &'a [u16; 3*BUF_THIRD]) -> Self {
        let ui = UI::init();
        let receiver = NfcReceiver::new(nfc_buffer, ui.state.platform.public().map(|a| a.0));
        let nfc = NFC{receiver, status: NFCReadOrDisplayStatus::NfcRead};
        let status = MainStatus::NFCReadOrDisplay(nfc);
        return Self {
            adc: ADC::new(()),
            status,
            ui,
            touch: Read::new(()),
        }
    }

    /// Call in event loop to progress through Kampela states
    pub fn advance(&mut self) {
        self.adc.advance(());
        match &mut self.status {
            MainStatus::<'a>::NFCReadOrDisplay(nfc) => {
                let mut request_ui_interaction = false;
                match nfc.status {
                    NFCReadOrDisplayStatus::NfcRead => {
                        if let Some(s) = nfc.receiver.advance(self.adc.read()) {
                            match s {
                                Err(e) => {
                                    match e {
                                        NfcError::InvalidAddress => {
                                            self.ui.handle_message("Invalid sender address".to_owned())
                                        }
                                    }
                                    request_ui_interaction = true;
                                }
                                Ok(s) => {
                                    match s {
                                        NfcStateOutput::Operational(i) => {
                                            if i == 1 {
                                                self.ui.handle_message("Receiving NFC packets...".to_owned());
                                                nfc.status = NFCReadOrDisplayStatus::DisplayMessage;
                                            }
                                        },
                                        NfcStateOutput::Done(r) => {
                                            match r {
                                                NfcResult::Empty => {request_ui_interaction = true},
                                                NfcResult::DisplayAddress => {
                                                    self.ui.handle_address([0;76]);
                                                    request_ui_interaction = true;
                                                },
                                                NfcResult::Transaction(transaction) => {
                                                    self.ui.handle_transaction(transaction);
                                                    request_ui_interaction = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            nfc.status = NFCReadOrDisplayStatus::DisplayMessage;
                        }
                    },
                    NFCReadOrDisplayStatus::DisplayMessage => {
                        if !self.ui.advance(self.adc.read()).is_none() {  //halt nfc receiving untill have enough charge to start update display
                            nfc.status = NFCReadOrDisplayStatus::NfcRead;
                        }
                    }
                }
                if request_ui_interaction {
                    enable_touch_int();
                    self.status = MainStatus::DisplayOrTouch;
                }
            }
            MainStatus::DisplayOrTouch => {
                match get_display_or_touch_status() {
                    DisplayOrTouchStatus::DisplayOrListen => {
                        self.ui.advance(self.adc.read());
                    }
                    DisplayOrTouchStatus::TouchOperation => {
                        match self.touch.advance(()) {
                            Ok(Some(touch)) => {
                                try_push_touch_data(touch);
                                set_display_or_touch_status(DisplayOrTouchStatus::DisplayOrListen);
                            },
                            Ok(None) => {},
                            Err(e) => panic!("{:?}", e),
                        }
                    },
                }
            }

        }
    }
}
