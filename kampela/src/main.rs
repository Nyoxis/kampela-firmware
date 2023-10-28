#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate core;

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};
use core::{alloc::Layout, panic::PanicInfo};
use core::ptr::addr_of;
use cortex_m::asm::delay;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_alloc::Heap;
use lazy_static::lazy_static;
use parity_scale_codec::Decode;
use primitive_types::H256;

use efm32pg23_fix::{interrupt, Interrupt, NVIC, Peripherals};
use kampela_system::devices::flash::*;
use kampela_ui::platform::Platform;

mod ui;
use ui::UI;
mod nfc;
use nfc::{BufferStatus, turn_nfc_collector_correctly, NfcCollector, NfcReceiver, NfcResult, process_nfc_payload};

#[global_allocator]
static HEAP: Heap = Heap::empty();

use kampela_system::{
    PERIPHERALS, CORE_PERIPHERALS, in_free,
    devices::{power::ADC, psram::{psram_read_at_address, CheckedMetadataMetal, ExternalPsram, PsramAccess}, se_rng::SeRng},
    draw::burning_tank,
    init::init_peripherals,
    parallel::Operation,
    BUF_THIRD, CH_TIM0, LINK_1, LINK_2, LINK_DESCRIPTORS, TIMER0_CC0_ICF, NfcXfer, NfcXferBlock,
};
use kampela_ui::uistate::Screen;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::free;
use cortex_m::interrupt::Mutex;

//use p256::ecdsa::{signature::{hazmat::PrehashVerifier}, Signature, VerifyingKey};
//use sha2::Digest;
//use spki::DecodePublicKey;
use substrate_parser::{MarkedData, compacts::find_compact, parse_transaction_unmarked};
use schnorrkel::{
    context::attach_rng,
    derive::{ChainCode, Derivation},
    keys::Keypair,
    signing_context,
    ExpansionMode,
    MiniSecretKey,
};

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
            peripherals.LDMA_S.if_.reset();
            let mut buffer_status = BUFFER_STATUS.borrow(cs).borrow_mut();
            match buffer_status.pass_if_done7() {
                Ok(_) => {
                    if !buffer_status.is_write_halted() {
                        peripherals.LDMA_S.linkload.write(|w_reg| w_reg.linkload().variant(1 << CH_TIM0));
                    }
                },
                Err(_) => {}
            }
        }
        else {panic!("can not borrow peripherals in ldma interrupt")}
    });
}

// const ALICE_KAMPELA_KEY: &[u8] = &[24, 79, 109, 158, 13, 45, 121, 126, 185, 49, 212, 255, 134, 18, 243, 96, 119, 210, 175, 115, 48, 181, 19, 238, 61, 135, 28, 186, 185, 31, 59, 9, 172, 24, 200, 176, 25, 207, 214, 199, 221, 214, 171, 143, 80, 246, 86, 104, 48, 40, 21, 99, 114, 3, 232, 85, 101, 7, 128, 198, 36, 11, 101, 63, 180, 120, 97, 66, 191, 43, 74, 35, 69, 3, 219, 194, 72, 141, 68, 185, 188, 177, 117, 246, 178, 250, 89, 134, 116, 20, 248, 247, 151, 45, 130, 59];
const SIGNING_CTX: &[u8] = b"substrate";


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
        let mut core_periph = CORE_PERIPHERALS.borrow(cs).borrow_mut();
        NVIC::unpend(Interrupt::LDMA);
        NVIC::mask(Interrupt::LDMA);
        unsafe {
            core_periph.NVIC.set_priority(Interrupt::LDMA, 3);
            NVIC::unmask(Interrupt::LDMA);
        }
    });

    delay(1000);


    free(|cs| {
        PERIPHERALS.borrow(cs).replace(Some(peripherals));
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

    let mut ui = UI::init();
    let mut adc = ADC::new();

    // hard derivation
    //let junction = DeriveJunction::hard("kampela");
    // let pair_derived = pair
    //         //.hard_derive_mini_secret_key(Some(ChainCode(*junction.inner())), b"")
    //         .0
    //         .expand_to_keypair(ExpansionMode::Ed25519);


    let mut nfc = NfcReceiver::new(&nfc_buffer, ui.state.platform.public());

    loop {
        adc.advance(());
        let voltage = adc.read();
        ui.advance(adc.read());
        let nfc_result = nfc.advance(voltage);
        
        if nfc_result.is_some() {
            match nfc_result.unwrap() {
                NfcResult::KampelaStop => {},
                NfcResult::DisplayAddress(addr) => {
                    /*
                    // TODO: implement adress
                    // let mut stuff = [0u8];
                    let line1 = format!("substrate:0x{}", hex::encode(pair.public.to_bytes()));
                    // stuff[0..79].copy_from_slice(&line1.as_bytes());
                    ui.handle_address(line1.as_bytes().try_into().expect("static length"));*/
                    ui.handle_address(addr);
                },
                NfcResult::Transaction(transaction) => {
                    /*
                    let carded = transaction.decoded_transaction.card(&transaction.specs, &transaction.spec_name);

                    let call = carded.call.into_iter().map(|card| card.show()).collect::<Vec<String>>().join("\n");
                    let extensions = carded.extensions.into_iter().map(|card| card.show()).collect::<Vec<String>>().join("\n");

                    let context = signing_context(SIGNING_CTX);
                    let signature = pair.sign(attach_rng(context.bytes(&transaction.data_to_sign), &mut SeRng{}));
                    let mut signature_with_id: [u8; 65] = [1; 65];
                    signature_with_id[1..].copy_from_slice(&signature.to_bytes());
                    // let signature_into_qr: [u8; 130] = ;

                    ui.handle_transaction(call, extensions, hex::encode(signature_with_id).into_bytes().try_into().expect("static length"));*/
                    ui.handle_transaction(transaction);


/* // calculate correct hash of the payload
{
            let mut hasher = sha2::Sha256::new();
            in_free(|peripherals| {
                for shift in 0..nfc_payload.encoded_data.total_len {
                    let address = nfc_payload.encoded_data.start_address.try_shift(shift).unwrap();
                    let single_element_vec = psram_read_at_address(peripherals, address, 1usize).unwrap();
                    if shift == 0 {first_byte = Some(single_element_vec[0])}
                    hasher.update(&single_element_vec);
                }
            });
            let hash = hasher.finalize();

            // transform signature and verifying key from der-encoding into usable form
            let signature = Signature::from_der(&nfc_payload.companion_signature).unwrap();
            let verifying_key = VerifyingKey::from_public_key_der(&nfc_payload.companion_public_key).unwrap();

            // and check
            assert!(verifying_key
                .verify_prehash(&hash, &signature)
                .is_ok());

}
*/

                },
            }
        }
    }
}