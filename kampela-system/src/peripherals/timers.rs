use efm32pg23_fix::Peripherals;
use crate::peripherals::gpio_pins::*;

const DISP_INT_CLOCK: usize = 1; //hz - datasheet
// HFRCODPLL = 19 MHz default
// PRESCEM01GRPACLK = EM01GRPACLK / presc = HFRCODPLL / presc
// presc = 1023
const TIMER_TOP: u16 = ((19_000_000 / 1024) / DISP_INT_CLOCK / 2) as u16  - 1;

/// Init timers
pub fn init_timers(peripherals: &mut Peripherals) {
    init_timer0(peripherals);
    init_timer1(peripherals);
}

/// set up TIMER0 for NFC reading
fn init_timer0(peripherals: &mut Peripherals) {
    peripherals
        .gpio_s
        .timer0_cc0route()
        .write(|w_reg| unsafe {
            w_reg
                .port().bits(PORT_A)
                .pin().bits(NFC_PIN)
    });
    peripherals
        .gpio_s
        .timer0_routeen()
        .write(|w_reg| w_reg.cc0pen().set_bit());

    // synchronizing
    while peripherals.timer0_s.en().read().en().bit_is_set() & peripherals.timer0_s.status().read().syncbusy().bit_is_set() {}

    peripherals
        .timer0_s
        .en()
        .write(|w_reg| w_reg.en().clear_bit());

    while peripherals.timer0_s.en().read().disabling().bit_is_set() {}

    peripherals
        .timer0_s
        .cc0_cfg()
        .write(|w_reg| {
            w_reg
                .mode().inputcapture()
                .coist().clear_bit()
                .filt().disable()
                .insel().pin()
    });
    
    peripherals
        .timer0_s
        .cfg()
        .write(|w_reg| {
            w_reg
                .mode().up()
                .sync().disable()
                .osmen().clear_bit()
                .qdm().x2()
                .debugrun().run()
                .dmaclract().clear_bit()
                .clksel().prescem01grpaclk()
                .dissyncout().dis()
                .ati().clear_bit()
                .presc().div1()
                
    });

    peripherals
        .timer0_s
        .en()
        .write(|w_reg| w_reg.en().set_bit());

    peripherals
        .timer0_s
        .cc0_ctrl()
        .write(|w_reg| {
            w_reg
                .icevctrl().falling()
                .icedge().falling()
                .cufoa().none()
                .cofoa().none()
                .cmoa().none()
                .outinv().set_bit()
    });

    peripherals
        .timer0_s
        .cmd()
        .write(|w_reg| w_reg.stop().set_bit());

    peripherals
        .timer0_s
        .cnt()
        .reset();

    peripherals
        .timer0_s
        .ctrl()
        .write(|w_reg| {
            w_reg
                .risea().none()
                .falla().reloadstart()
                .x2cnt().clear_bit()
    });
}

/// set up TIMER1 for Display COM inversal
fn init_timer1(peripherals: &mut Peripherals) {
    peripherals
        .gpio_s
        .timer1_cc0route()
        .write(|w_reg| unsafe {
            w_reg
                .port().bits(PORT_D)
                .pin().bits(DISP_INT_PIN)
            });
    peripherals
        .gpio_s
        .timer1_routeen()
        .write(|w_reg| w_reg.cc0pen().set_bit());

    // synchronizing
    while peripherals.timer1_s.en().read().en().bit_is_set() & peripherals.timer1_s.status().read().syncbusy().bit_is_set() {}

    peripherals
        .timer1_s
        .en()
        .write(|w_reg| w_reg.en().clear_bit());

    while peripherals.timer1_s.en().read().disabling().bit_is_set() {}

    peripherals
        .timer1_s
        .cc0_cfg()
        .write(|w_reg| {
            w_reg
                .mode().outputcompare()
                .coist().clear_bit()
            });

    peripherals
        .timer1_s
        .cfg()
        .write(|w_reg| {
            w_reg
                .mode().up()
                .sync().disable()
                .osmen().clear_bit()
                .debugrun().run()
                .clksel().prescem01grpaclk()
                .presc().div1024()
            });

    peripherals
        .timer1_s
        .en()
        .write(|w_reg| w_reg.en().set_bit());
    
    peripherals
        .timer1_s
        .cc0_ctrl()
        .write(|w_reg| {
            w_reg
                .cofoa().toggle()
                .cmoa().none()
                .cufoa().none()
            });

    peripherals
        .timer1_s
        .cnt()
        .reset();

    peripherals
        .timer1_s
        .top()
        .write(|w_reg| unsafe { w_reg.top().bits(TIMER_TOP) });
    
    peripherals
        .timer1_s
        .cmd()
        .write(|w_reg| w_reg.start().set_bit());
}