use cortex_m::interrupt::free;
use efm32pg23_fix::Peripherals;

pub const LINK_DESCRIPTORS: u32 = 0b00000111000100000111111111110000;
pub const CH_TIM0: u8 = 7;
pub const LINK_1: u32 = 0b00000000000000000000000000010011;
pub const LINK_2: u32 = 0b11111111111111111111111111100011;

pub const TIMER0_CC0_ICF: u32 = 0x40048074;

pub const BUF_THIRD: usize = 2048;

#[repr(C)]
#[derive(Debug)]
pub struct NfcXfer {
    pub descriptors: u32,
    pub source: u32,
    pub dest: u32,
    pub link: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct NfcXferBlock {
    pub block0: NfcXfer,
    pub block1: NfcXfer,
    pub block2: NfcXfer,
}

/// Set up LDMA for NFC capture
pub fn init_ldma(peripherals: &mut Peripherals, nfc_descriptor_address: *const NfcXferBlock) {
    // set up ldma
    peripherals
        .ldma_s
        .en()
        .write(|w_reg| {
            w_reg
                .en().set_bit()
    });

    peripherals
        .ldma_s
        .ctrl()
        .write(|w_reg| unsafe {
            w_reg
                .numfixed().bits(0)
    });

    peripherals
        .ldma_s
        .synchwen()
        .write(|w_reg| unsafe {
            w_reg
                .syncseten().bits(0)
                .syncclren().bits(0)
    });

    peripherals
        .ldma_s
        .chdis()
        .write(|w_reg| unsafe {
            w_reg
                .chdis().bits(0xFF)
    });

    peripherals
        .ldma_s
        .dbghalt()
        .write(|w_reg| unsafe {
            w_reg
                .dbghalt().bits(0)
    });
    
    peripherals
        .ldma_s
        .reqdis()
        .write(|w_reg| unsafe {
            w_reg
                .reqdis().bits(0)
    });

    peripherals
        .ldma_s
        .ien()
        .write(|w_reg| {
            w_reg
                .error().set_bit()
    });

    peripherals
        .ldma_s
        .if_()
        .reset();

    // start ldma transfer
    peripherals
        .ldma_s
        .if_()
        .modify(|_, w_reg| {
            w_reg
                .done7().clear_bit()
        }
    );

    peripherals
        .ldmaxbar_s
        .ch7_reqsel()
        .write(|w_reg| unsafe {
            w_reg
                .sigsel().bits(0) // _LDMAXBAR_CH_REQSEL_SIGSEL_TIMER0CC0
                .sourcesel().bits(2) // _LDMAXBAR_CH_REQSEL_SOURCESEL_TIMER0
        }
    );

    peripherals
        .ldma_s
        .ch7_loop()
        .write(|w_reg| unsafe {
            w_reg
                .loopcnt().bits(0)
        }
    );

    peripherals
        .ldma_s
        .ch7_cfg()
        .write(|w_reg| {
            w_reg
                .arbslots().one()
                .srcincsign().positive()
                .dstincsign().positive()
        }
    );
    
    peripherals
        .ldma_s
        .ch7_link()
        .write(|w_reg| {
            w_reg
                .link().clear_bit();
            unsafe {
                w_reg.linkaddr().bits(nfc_descriptor_address as u32 >> 2)
            }
        }
    );

    // there starts a critical section
    free(|_cs| {
        peripherals
            .ldma_s
            .ien()
            .write(|w_reg| unsafe {
                w_reg
                    .chdone().bits(1 << CH_TIM0)
            }
        );

        peripherals
            .ldma_s
            .synchwen()
            .reset(); // default values, i.e. 0 for clr_off, clr_on, set_off, set_on

        peripherals
            .ldma_s
            .chdone()
            .write(|w_reg| {
                w_reg
                    .chdone7().clear_bit()
            }
        );

        peripherals
            .ldma_s
            .linkload()
            .write(|w_reg| unsafe {
                w_reg
                    .linkload().bits(1 << CH_TIM0)
            }
        );
    });
}
