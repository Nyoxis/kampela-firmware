use alloc::{boxed::Box, string::String};
use efm32pg23_fix::Peripherals;
use cortex_m::asm::delay;

use kampela_display_common::display_def::*;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::rectangle::Rectangle,
    Drawable,
};

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};

use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

use crate::peripherals::{gpio_pins::DISP_INT_PIN, usart::{deselect_display, select_display, write_to_usart}};
use crate::devices::display_transmission::{DATA_UPDATE_MODE, HOLD_MODE, ALL_CLEAR_MODE, X_ADDRESS_WIDTH};
use crate::draw::FrameBuffer;
//**** Debug stuff ****//

/// Emergency debug function that spits out errors
pub fn burning_tank(peripherals: &mut Peripherals, text: String) {
    make_text(peripherals, &text);
}

/// see this <https://github.com/embedded-graphics/embedded-graphics/issues/716>
fn make_text(peripherals: &mut Peripherals, text: &str) {
    let mut buffer = Box::new(FrameBuffer::new_white());
    let to_print = TextToPrint{line: text};
    to_print.draw(buffer.as_mut()).unwrap();
    buffer.apply(peripherals);
}

struct TextToPrint<'a> {
    pub line: &'a str,
}

/// For custom font, see this <https://github.com/embedded-graphics/examples/blob/main/eg-0.7/examples/text-custom-font.rs>
impl Drawable for TextToPrint<'_> {
    type Color = BinaryColor;
    type Output = ();
    fn draw<D>(
        &self, 
        target: &mut D
    ) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color> 
    {
        let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Left)
            .paragraph_spacing(5)
            .build();
        let bounds = Rectangle::new(Point::zero(), Size::new(SCREEN_SIZE_X, 0));
        TextBox::with_textbox_style(self.line, bounds, character_style, textbox_style).draw(target)?;
        Ok(())
    }
}

/// Normal drawing protocol, with full screen clearing
pub fn debug_draw(peripherals: &mut Peripherals, stuff: [u8; SCREEN_BUFFER_SIZE]) {
    deselect_display(&mut peripherals.gpio_s);
    delay(50000);
    sharp_write_data(peripherals, &stuff);
}

/// Send EPD to low power state; should be performed when screen is not drawing at all times to
/// extend component life
pub fn sharp_hold(peripherals: &mut Peripherals) {
    select_display(&mut peripherals.gpio_s);
    let data_transfer: u16 = HOLD_MODE;
    for data in data_transfer.to_be_bytes().iter() {
        write_to_usart(peripherals, *data);
    }
    deselect_display(&mut peripherals.gpio_s);
}

/// Send data to EPD
///
/// for critical section
pub fn sharp_write_data(peripherals: &mut Peripherals, data_set: &[u8]) {
    select_display(&mut peripherals.gpio_s);
    delay(2000);
    let mut data_transfer: u16 = DATA_UPDATE_MODE;
    for (gateline, line ) in data_set.chunks(X_ADDRESS_WIDTH).enumerate() {
        data_transfer |= ((0x00FF & (gateline + 1) as u16) << 8).reverse_bits();
        for data in data_transfer.to_be_bytes().iter() {
            write_to_usart(peripherals, *data);
        }
        data_transfer = 0;
        for data in line.iter() {
            write_to_usart(peripherals, *data);
        }
    }
    deselect_display(&mut peripherals.gpio_s);
    //    display_data_command_clear(peripherals);
}

pub fn sharp_clear(peripherals: &mut Peripherals) {
    select_display(&mut peripherals.gpio_s);
    let data_transfer: u16 = ALL_CLEAR_MODE;
    for data in data_transfer.to_be_bytes().iter() {
        write_to_usart(peripherals, *data);
    }
    deselect_display(&mut peripherals.gpio_s);
}