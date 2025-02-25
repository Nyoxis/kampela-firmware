//! Map GPIO pins

#![allow(dead_code)]

use cortex_m::asm::delay;
use efm32pg23_fix::GpioS;

pub const PORT_A: u8 = 0;
/*
pub const MCU_OK: u8 = 3;*/
pub const SCL_PIN: u8 = 4;
pub const SDA_PIN: u8 = 5;/*
pub const TOUCH_RES_PIN: u8 = 7;*/
pub const NFC_PIN: u8 = 8;/*
pub const POW_PIN: u8 = 9;
*/


pub const PORT_B: u8 = 1;

pub const TOUCH_INT_PIN: u8 = 1;

pub const PORT_C: u8 = 2;

pub const FLASH_CS_PIN: u8 = 0;
pub const E_MISO_PIN: u8 = 1;
pub const E_MOSI_PIN: u8 = 2;
pub const E_SCK_PIN: u8 = 3;
pub const PSRAM_CS_PIN: u8 = 4;
pub const PSRAM_MISO_PIN: u8 = 5;
pub const PSRAM_MOSI_PIN: u8 = 6;
pub const PSRAM_SCK_PIN: u8 = 7;

pub const PORT_D: u8 = 3;

pub const DISP_CS_PIN: u8 = 0;
pub const DISP_INT_PIN: u8 = 1;


/// Macro to switch a specific pin on a specific port.
///
/// At this point GPIO must be already clocked from elsewhere and port must be
/// in correct mode.
///
/// This does not change previously set bits.
macro_rules! gpio_pin {
    ($(#[$attr_set:meta] #[$attr_clear:meta] #[$attr_common:meta] $func_set: ident, $func_clear: ident, $port: tt, $pin: tt), *) => {
        $(
            #[$attr_set]
            #[$attr_common]
            pub fn $func_set(gpio: &mut GpioS) {
                gpio
                    .$port()
                    .modify(|r, w| unsafe { w.dout().bits(r.dout().bits() | (1 << $pin)) });
            }

            #[$attr_clear]
            #[$attr_common]
            pub fn $func_clear(gpio: &mut GpioS) {
                gpio
                    .$port()
                    .modify(|r, w| unsafe { w.dout().bits(r.dout().bits() & !(1 << $pin)) });
            }
        )*
    }
}

// Prepare GPIO pins
/*
gpio_pin!(
    /// Set MCU_OK status:
    /// Clear MCU_OK status:
    /// port A, pin [`MCU_OK`].
    mcu_ok_set,
    mcu_ok_clear,
    porta_dout,
    MCU_OK
);
*/
gpio_pin!(
    /// Set flash chip select:
    /// Clear flash chip select:
    /// port C, pin [`FLASH_CS_PIN`].
    flash_chip_select_set,
    flash_chip_select_clear,
    portc_dout,
    FLASH_CS_PIN
);

gpio_pin!(
    /// Set display chip select:
    /// Clear display chip select:
    /// port D, pin [`DISP_CS_PIN`].
    display_chip_select_set,
    display_chip_select_clear,
    portd_dout,
    DISP_CS_PIN
);
/*
gpio_pin!(
    /// Set touch reset:
    /// Clear touch reset:
    /// port A, pin [`TOUCH_RES_PIN`].
    touch_res_set,
    touch_res_clear,
    porta_dout,
    TOUCH_RES_PIN
);
*/
gpio_pin!(
    /// scl set:
    /// scl clear:
    /// port A, pin [`SCL_PIN`].
    scl_set,
    scl_clear,
    porta_dout,
    SCL_PIN
);

gpio_pin!(
    /// sda set:
    /// sda clear:
    /// port A, pin [`SDA_PIN`].
    sda_set,
    sda_clear,
    porta_dout,
    SDA_PIN
);
/*
gpio_pin!(
    /// Set power:
    /// Clear power:
    /// port A, pin [`POW_PIN`].
    pow_set,
    pow_clear,
    porta_dout,
    POW_PIN
);
*/
gpio_pin!(
    /// Set TOUCH INT:
    /// Clear TOUCH INT:
    /// port B, pin [`TOUCH_INT_PIN`].
    touch_int_pin_set,
    touch_int_pin_clear,
    portb_dout,
    TOUCH_INT_PIN
);

gpio_pin!(
    /// Set MISO:
    /// Clear MISO:
    /// port C, pin [`E_MISO_PIN`].
    miso_set,
    miso_clear,
    portc_dout,
    E_MISO_PIN
);

gpio_pin!(
    /// Set MOSI:
    /// Clear MOSI:
    /// port C, pin [`E_MOSI_PIN`].
    mosi_set,
    mosi_clear,
    portc_dout,
    E_MOSI_PIN
);

gpio_pin!(
    /// Set SCK:
    /// Clear SCK:
    /// port C, pin [`E_SCK_PIN`].
    sck_set,
    sck_clear,
    portc_dout,
    E_SCK_PIN
);


gpio_pin!(
    /// Set PSRAM CS:
    /// Clear PSRAM CS:
    /// port C, pin [`PSRAM_CS_PIN`].
    psram_chip_select_set,
    psram_chip_select_clear,
    portc_dout,
    PSRAM_CS_PIN
);

gpio_pin!(
    /// Set PSRAM MISO:
    /// Clear PSRAM MISO:
    /// port C, pin [`PSRAM_MISO_PIN`].
    psram_miso_set,
    psram_miso_clear,
    portc_dout,
    PSRAM_MISO_PIN
);

gpio_pin!(
    /// Set PSRAM MOSI:
    /// Clear PSRAM MOSI:
    /// port C, pin [`PSRAM_MOSI_PIN`].
    psram_mosi_set,
    psram_mosi_clear,
    portc_dout,
    PSRAM_MOSI_PIN
);

gpio_pin!(
    /// Set PSRAM SCK:
    /// Clear PSRAM SCK:
    /// port C, pin [`PSRAM_SCK_PIN`].
    psram_sck_set,
    psram_sck_clear,
    portc_dout,
    PSRAM_SCK_PIN
);

gpio_pin!(
    /// Set NFC pin:
    /// Clear NFC pin:
    /// port A, pin [`NFC_PIN`].
    nfc_pin_set,
    nfc_pin_clear,
    porta_dout,
    NFC_PIN
);

/// GPIO initializations
pub fn init_gpio(gpio: &mut GpioS) {
    map_gpio(gpio);
    set_gpio_pins(gpio);
}

/// Set GPIO functions
fn map_gpio(gpio: &mut GpioS) {
    gpio
        .porta_model()
        .write(|w_reg| {
            w_reg
                .mode3().pushpull() // MCU operational indicator
                .mode4().wiredandpullup() // SCL for USART (display)
                .mode5().wiredandpullup() // SDA for USART (display)
                .mode6().pushpull() // Display reset
                .mode7().pushpull() // Touch reset
    });
    gpio
        .porta_modeh()
        .write(|w_reg| {
            w_reg
                .mode0().inputpullfilter() // NFC
                .mode1().pushpull() // Power 2.8 V
    });
    gpio
        .portb_model()
        .write(|w_reg| {
            w_reg
                .mode1().inputpullfilter() // interrupts from display sensor
                .mode4().input() // BUSY spi
    });
    touch_int_pin_set(gpio); // pull-up;
    gpio
        .portc_model()
        .write(|w_reg| {
            w_reg
                .mode0().pushpull() // Flash chip select
                .mode1().inputpull() // Display MISO
                .mode2().pushpull() // Display MOSI
                .mode3().pushpull() //Display SCK
                .mode4().pushpull() // PSRAM chip select
                .mode5().inputpull() // PSRAM MISO
                .mode6().pushpull() // PSRAM MOSI
                .mode7().pushpull() // PSRAM SCK
    });
    gpio
        .portd_model()
        .write(|w_reg| {
            w_reg
                .mode0().pushpull() // Display chip select
                .mode1().pushpull() // Display extin
    });
}

/// Set GPIO pins to their starting values
fn set_gpio_pins(gpio: &mut GpioS) {
    //mcu_ok_clear(gpio);
    //mcu_ok_clear(gpio);
    //pow_set(gpio);
    delay(100000); // wait after power set! (epaper manual for 2.8V setup)
    display_chip_select_clear(gpio);
    //touch_res_set(gpio);
    sda_set(gpio);
    scl_set(gpio);
    flash_chip_select_set(gpio);
    miso_set(gpio);
    mosi_set(gpio);
    sck_clear(gpio);
    psram_chip_select_set(gpio);
    psram_miso_set(gpio);
    psram_mosi_clear(gpio);
    psram_sck_clear(gpio);
    nfc_pin_clear(gpio);
}

pub fn touch_int_pin_set_input(gpio: &mut GpioS) {
    gpio
        .portb_model()
        .write(|w_reg| {
            w_reg
                .mode1().inputpullfilter() // interrupts from display sensor
            });
    touch_int_pin_set(gpio);
    enable_touch_int_flag(gpio);
}

pub fn send_touch_int(gpio: &mut GpioS) {
    gpio
        .portb_model()
        .write(|w_reg| {
            w_reg
                .mode1().pushpull() // interrupts from display sensor
            });
    disable_touch_int_flag(gpio);
    touch_int_pin_clear(gpio);
}

/// Set up external interrupt pins (used to get touch events from touch pad)
pub fn enable_touch_int_flag(gpio: &mut GpioS) {
    gpio
        .if_clr()
        .write(|w_reg| w_reg.extif0().set_bit());
    gpio
        .extipsell()
        .write(|w_reg| w_reg.extipsel0().portb());
    gpio
        .extipinsell()
        .write(|w_reg| w_reg.extipinsel0().pin1());
    gpio
        .extirise()
        .modify(|r_reg, w_reg| unsafe { w_reg.extirise().bits(r_reg.extirise().bits() & !(1 << 0)) });
    gpio
        .extifall()
        .modify(|r_reg, w_reg| unsafe { w_reg.extifall().bits(r_reg.extifall().bits() | (1 << 0)) });
    gpio
        .ien()
        .write(|w_reg| w_reg.extien0().set_bit())
}

pub fn disable_touch_int_flag(gpio: &mut GpioS) {
    gpio
        .ien()
        .write(|w_reg| w_reg.extien0().clear_bit())
}
