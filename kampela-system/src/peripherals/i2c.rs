
use efm32pg23_fix::Peripherals;
use crate::peripherals::gpio_pins::*;
use cortex_m::asm::delay;
use crate::{if_in_free, in_free, FreeError};
use crate::parallel::Operation;


#[derive(Debug)]
pub enum I2CError {
    ArbitrationLost,
    BusError,
    TransferNack,
    /// Peripheral mutex could not be borrowed
    PeripheralsLocked,
    /// Errors probably caused by skip in the state sequence
    SequenceError,
}

impl From<FreeError> for I2CError {
    fn from(_item: FreeError) -> Self {
        Self::PeripheralsLocked
    }
}

pub fn init_i2c(peripherals: &mut Peripherals) {
    peripherals
        .gpio_s
        .i2c0_routeen()
        .write(|w_reg| w_reg.sclpen().set_bit().sdapen().set_bit());
    peripherals
        .gpio_s
        .i2c0_sdaroute()
        .write(|w_reg| unsafe { w_reg.port().bits(0).pin().bits(SDA_PIN) });
    peripherals
        .gpio_s
        .i2c0_sclroute()
        .write(|w_reg| unsafe { w_reg.port().bits(0).pin().bits(SCL_PIN) });
    
    peripherals
        .i2c0_s
        .ien()
        .reset();
    peripherals
        .i2c0_s
        .if_()
        .reset();
    peripherals
        .i2c0_s
        .ctrl()
        .write(|w_reg| w_reg.slave().disable().clhr().standard());
    peripherals
        .i2c0_s
        .clkdiv()
        .write(|w_reg| unsafe { w_reg.div().bits(12) }); // divider calculated as 10, set to 12 for debug
    peripherals
        .i2c0_s
        .en()
        .write(|w_reg| w_reg.en().enable());
    peripherals
        .i2c0_s
        .ctrl()
        .write(|w_reg| w_reg.corerst().enable());
    delay(10000);
    peripherals
        .i2c0_s
        .ctrl()
        .write(|w_reg| w_reg.corerst().disable());
    delay(100000);
}

/// I2C bus reader for our touchpad, not generalized until someone needs it
pub struct ReadI2C {
    state: ReadI2CState,
    value: Option<u8>,
    timer: usize
}

pub enum ReadI2CState {
    /// Handle errata and cleanup before read;
    ErrataCheck,
    ///here the read is done
    Read,

    OutputData
}

impl ReadI2C {
    fn count(&mut self) -> bool {
        if self.timer == 0 {
            false
        } else {
            self.timer -= 1;
            true
        }
    }
}

impl Operation for ReadI2C {
    type Init = ();
    type Input<'a> = ();
    type Output = Result<Option<u8>, I2CError>;
    type StateEnum = ReadI2CState;

    fn new(_: ()) -> Self {
        Self {
            state: ReadI2CState::ErrataCheck,
            value: None,
            timer: 0,
        }
    }

    fn wind(&mut self, state: ReadI2CState, delay: usize) {
        self.state = state;
        self.timer = delay;
    }

    fn advance(&mut self, _: ()) -> Self::Output {
        if self.count() { return Ok(None) };
        match &self.state {
            ReadI2CState::ErrataCheck => {
                // Errata I2C_E303, patch follows sdk
                if if_in_free(|peripherals| 
                    peripherals
                        .i2c0_s
                        .status()
                        .read()
                        .rxdatav()
                        .bit_is_clear() 
                    &
                    peripherals
                        .i2c0_s
                        .status()
                        .read()
                        .rxfull()
                        .bit_is_set()
                )? {
                    in_free(|peripherals| {
                        let _dummy_data = peripherals
                            .i2c0_s
                            .rxdata()
                            .read()
                            .bits();
                        }
                    );
                    in_free(|peripherals| {
                        peripherals
                            .i2c0_s
                            .if_()
                            .write(|w_reg| w_reg.rxuf().clear_bit());
                    });
                }
                self.change(ReadI2CState::Read);
                Ok(None)
            },
            ReadI2CState::Read => {
                check_i2c_errors()?;
                if if_in_free(|peripherals|
                    peripherals
                        .i2c0_s
                        .if_()
                        .read()
                        .rxdatav()
                        .bit_is_set()
                )? {
                    in_free(|peripherals| 
                        self.value = Some(
                            peripherals
                                .i2c0_s
                                .rxdata()
                                .read()
                                .rxdata()
                                .bits()
                        )
                    );
                    self.wind_d(ReadI2CState::OutputData);
                }
                Ok(None)
            },

            ReadI2CState::OutputData => {
                in_free(|peripherals|
                    peripherals
                        .i2c0_s
                        .if_()
                        .write(|w_reg| w_reg.rxdatav().clear_bit().rxfull().clear_bit())
                );
                if let Some(out) = self.value {
                    Ok(Some(out))
                } else {
                    Err(I2CError::SequenceError)
                }
            }
        }
    }
}

/*
pub fn read_i2c_sync() -> Result<u8, I2CError> {
    let mut reader = ReadI2C::new();
    loop {
        if let Some(out) = reader.advance(())? {
            return Ok(out)
        }
    }
}
*/

pub fn check_i2c_errors() -> Result<(), I2CError> {
    let mut out = Ok(());
    in_free(|peripherals| {
        out = check_i2c_errors_free(peripherals)
    });
    out
}

pub fn acknowledge_i2c_tx() -> Result<(), I2CError> {
    let mut out = Ok(());
    in_free(|peripherals| {
        out = acknowledge_i2c_tx_free(peripherals)
    });
    out
}

pub fn mstop_i2c_wait_and_clear() -> Result<(), I2CError> {
    let mut out = Ok(());
    in_free(|peripherals| {
        out = mstop_i2c_wait_and_clear_free(peripherals)
    });
    out
}

pub fn check_i2c_errors_free(peripherals: &mut Peripherals) -> Result<(), I2CError> {
    let if_read = peripherals
        .i2c0_s
        .if_()
        .read();
    if if_read.arblost().bit_is_set() {return Err(I2CError::ArbitrationLost)}
    if if_read.buserr().bit_is_set() {return Err(I2CError::BusError)}
    Ok(())
}

pub fn acknowledge_i2c_tx_free(peripherals: &mut Peripherals) -> Result<(), I2CError> {
    check_i2c_errors_free(peripherals)?;
    while peripherals
        .i2c0_s
        .if_()
        .read()
        .ack()
        .bit_is_clear()
    {
        check_i2c_errors_free(peripherals)?;

        if peripherals
            .i2c0_s
            .if_()
            .read()
            .nack()
            .bit_is_set()
        {
            // clear interrupt flag
            peripherals
                .i2c0_s
                .if_()
                .write(|w_reg| w_reg.nack().clear_bit());
            // stop
            peripherals
                .i2c0_s
                .cmd()
                .write(|w_reg| w_reg.stop().set_bit());
            delay(100000);
            return Err(I2CError::TransferNack)
        }
    }
    // clear interrupt flag
    peripherals
        .i2c0_s
        .if_()
        .write(|w_reg| w_reg.ack().clear_bit());

    Ok(())
}

pub fn mstop_i2c_wait_and_clear_free(peripherals: &mut Peripherals) -> Result<(), I2CError> {
    check_i2c_errors_free(peripherals)?;
    while peripherals
        .i2c0_s
        .if_()
        .read()
        .mstop()
        .bit_is_clear()
    {
        check_i2c_errors_free(peripherals)?;
    }
    peripherals
        .i2c0_s
        .if_()
        .write(|w_reg| w_reg.mstop().clear_bit());
    Ok(())
}

