use efm32pg23_fix::{GpioS, Peripherals};
use cortex_m::asm::delay;

use crate::peripherals::usart::*;
use crate::peripherals::gpio_pins::{display_res_clear, display_res_set};
use crate::{FreeError, if_in_free, in_free};
use crate::parallel::{AsyncOperation, Threads};

use crate::devices::display::Bounds;
use kampela_display_common::display_def::*;
const X_ADDRESS_WIDTH: usize = (SCREEN_SIZE_Y / 8) as usize;

/// BUSY is on port B, pin [`SPI_BUSY_PIN`].
pub fn display_is_busy() -> Result<bool, FreeError> {
    if_in_free(|peripherals| spi_is_busy(&mut peripherals.gpio_s))
}

/// BUSY is on port B, pin [`SPI_BUSY_PIN`].
///
/// Blocking variant to be called from critical section (init, panic)
pub fn display_is_busy_cs(peripherals: &mut Peripherals) -> bool {
    spi_is_busy(&mut peripherals.gpio_s)
}

/// Send EPD to low power state; should be performed when screen is not drawing at all times to
/// extend component life
pub fn epaper_deep_sleep(peripherals: &mut Peripherals) {
    epaper_write_command(peripherals, &[0x10]); // from manual, enter deep sleep
    epaper_write_data(peripherals, &[0x03]); // Deep sleep mode 2, cannot retain RAM data
    delay(100); // why delay, from where the number?
}

/// EPD init, also should be performed to wake screen from sleep
///
/// used within critical section
pub fn epaper_hw_init_cs(peripherals: &mut Peripherals) {
    epaper_reset(&mut peripherals.gpio_s);
    while display_is_busy_cs(peripherals) {}
    epaper_write_command(peripherals, &[0x12]);
    delay(10000);
    while display_is_busy_cs(peripherals) {}
}

/// Reset EPD, should be performed in many situations
///
/// for critical section
///
/// Why these specific numbers for delays?
pub fn epaper_reset(gpio: &mut GpioS) {
    delay(1000);
    display_res_clear(gpio);
    delay(5000);
    display_res_set(gpio);
    delay(10000);
    display_res_clear(gpio);
    delay(5000);
    deselect_display(gpio); // this is not the default state, should not be here
    delay(5000);
}

/// Send command to EPD
///
/// for critical section
pub fn epaper_write_command(peripherals: &mut Peripherals, command_set: &[u8]) {
    // CS clear corresponds to selected chip, see epaper docs

    deselect_display(&mut peripherals.gpio_s);
    select_display(&mut peripherals.gpio_s); // not necessary if state is known and default at start
    
    display_select_command(&mut peripherals.gpio_s);
    for command in command_set.iter() {
        write_to_usart(peripherals, *command);
    }
    deselect_display(&mut peripherals.gpio_s);
}

/// Send data to EPD
///
/// for critical section
pub fn epaper_write_data(peripherals: &mut Peripherals, data_set: &[u8]) {
    deselect_display(&mut peripherals.gpio_s);
    select_display(&mut peripherals.gpio_s); // not necessary if state is known and default at start

    display_select_data(&mut peripherals.gpio_s);
    for data in data_set.iter() {
        write_to_usart(peripherals, *data);
    }
    deselect_display(&mut peripherals.gpio_s);
    //    display_data_command_clear(peripherals);
}

/// Send command `C` to EPD
pub struct EPDCommand<const C: u8>{
    threads: Threads<EPDByteState, 1>,
}

pub enum EPDByteState {
    Init,
    /// State where command is actually sent
    Send,
    /// Receive something to keep protocol running and close connection
    WaitSend,
    End,
}

impl Default for EPDByteState {
    fn default() -> Self { EPDByteState::End }
}

impl <const C: u8> AsyncOperation for EPDCommand<C> {
    type Init = ();
    type Input<'a> = ();
    type Output = Option<bool>;

    fn new(_: ()) -> Self {
        Self {
            threads: Threads::new(EPDByteState::Init),
        }
    }
    
    fn advance(&mut self, _: ()) -> Self::Output {
        match self.threads.turn() {
            EPDByteState::Init => {
                in_free(|peripherals| {
                    display_select_command(&mut peripherals.gpio_s);
                });
                self.threads.change(EPDByteState::Send);
                Some(false)
            },
            EPDByteState::Send => {
                if if_in_free(|peripherals|
                    peripherals.usart0_s.status().read().txbl().bit_is_set()
                ) != Ok(true) {
                    return None
                }
                in_free(|peripherals|
                    peripherals
                        .usart0_s
                        .txdata()
                        .write(|w_reg| unsafe { w_reg.txdata().bits(C) })
                );
                self.threads.change(EPDByteState::WaitSend);
                Some(false)
            }
            EPDByteState::WaitSend => {
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
                self.threads.change(EPDByteState::End);
                Some(true)
            },
            EPDByteState::End => {
                Some(true)
            },
        }
    }
}

/// Send data array to EPD
pub enum EPDDataState {
    Init,
    /// Send byte
    Send,
    /// Receive something to keep protocol running and close connection
    WaitSend,

    End,
}

impl Default for EPDDataState {
    fn default() -> Self { EPDDataState::End }
}

pub struct EPDData<const LEN: usize>{
    threads: Threads<EPDDataState, 1>,
    position: usize,
}

impl <const LEN: usize> AsyncOperation for EPDData<LEN> {
    type Init = ();
    type Input<'a> = &'a [u8; LEN];
    type Output = Option<bool>;

    fn new(_: Self::Init) -> Self {
        Self {
            threads: Threads::new(EPDDataState::Init),
            position: 0,
        }
    }

    fn advance(&mut self, data: Self::Input<'_>) -> Self::Output {
        match self.threads.turn() {
            EPDDataState::Init => {
                in_free(|peripherals| {
                    display_select_data(&mut peripherals.gpio_s);
                });
                self.threads.change(EPDDataState::Send);
                Some(false)
            },
            EPDDataState::Send => {
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
                self.threads.change(EPDDataState::WaitSend);
                Some(false)
            },
            EPDDataState::WaitSend => {
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
                if self.position < LEN-1 {
                    self.position += 1;
                    self.threads.change(EPDDataState::Send);
                } else {
                    self.threads.change(EPDDataState::End);
                }
                Some(false)
            },
            EPDDataState::End => {
                Some(true)
            },
        }
    }
}

pub struct EPDDataBuffer<const LEN: usize>{
    threads: Threads<EPDDataState, 1>,
    position: usize,
    position_borders: Option<[usize;3]>,
}

impl <const LEN: usize> AsyncOperation for EPDDataBuffer<LEN> {
    type Init = Option<Bounds>; // Borders of 2D array
    type Input<'a> = &'a [u8; LEN];
    type Output = Option<bool>;

    fn new(addresses: Self::Init) -> Self {
        let (position, position_borders) = match addresses {
            None => {
                (0, None)
            },
            Some(b) => {
                let x_start_position = b.0 as usize;
                let x_end_position = b.1 as usize;
                let y_start_position = (SCREEN_SIZE_X - 1) as usize - b.2 as usize; //Y coordinates inversed for some reason
                let y_end_position = (SCREEN_SIZE_X - 1) as usize - b.3 as usize;

                let end_position = X_ADDRESS_WIDTH * y_end_position + x_end_position;
                (
                    y_start_position * X_ADDRESS_WIDTH as usize + x_start_position,
                    Some([x_start_position, x_end_position, end_position]),
                )
            }
        };

        Self {
            threads: Threads::new(EPDDataState::Init),
            position,
            position_borders,
        }
    }

    fn advance(&mut self, data: Self::Input<'_>) -> Self::Output {
        match self.threads.turn() {
            EPDDataState::Init => {
                in_free(|peripherals| {
                    display_select_data(&mut peripherals.gpio_s);
                });
                self.threads.change(EPDDataState::Send);
                Some(false)
            },
            EPDDataState::Send => {
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
                self.threads.change(EPDDataState::WaitSend);
                Some(false)
            },
            EPDDataState::WaitSend => {
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
                match self.position_borders {
                    None => {
                        if self.position < LEN-1 {
                            self.position += 1;
                            self.threads.change(EPDDataState::Send);
                            if self.position % X_ADDRESS_WIDTH == 0 {
                                return None // unblock thread
                            }
                        } else {
                            self.threads.change(EPDDataState::End);
                        }
                    },
                    Some(b) => {
                        if self.position < b[2] {
                            let x_position = self.position % X_ADDRESS_WIDTH;

                            self.threads.change(EPDDataState::Send);
                            if x_position >= b[1] {
                                let y_position = self.position / X_ADDRESS_WIDTH;
                                self.position = (y_position + 1) * X_ADDRESS_WIDTH + b[0];
                                return None // unblock thread
                            } else {
                                self.position += 1;
                            }
                        } else {
                            self.threads.change(EPDDataState::End);
                        }
                    }
                }
                Some(false)
            },
            EPDDataState::End => {
                Some(true)
            },
        }
    }
}