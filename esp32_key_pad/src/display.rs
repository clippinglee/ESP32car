//! I2C 12864 OLED (SSD1306) 驱动

use core::fmt::Write;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub type OledDisplay = Ssd1306<
    I2CInterface<I2c<'static, Blocking>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub fn new(i2c: I2c<'static, Blocking>) -> OledDisplay {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let _ = display.init();
    let _ = display.clear(BinaryColor::Off);
    let _ = display.flush();
    display
}

pub fn update(
    display: &mut OledDisplay,
    axis_x: i16,
    axis_y: i16,
    speed_limit: u8,
    sent: bool,
    ack_count: u32,
) {
    let _ = display.clear(BinaryColor::Off);

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let mut buf = heapless::String::<32>::new();
    let _ = write!(buf, "X:{:+5} Y:{:+5}", axis_x, axis_y);
    let _ = Text::new(&buf, Point::new(0, 12), style).draw(display);

    buf.clear();
    let _ = write!(buf, "SPD: {}%", speed_limit);
    let _ = Text::new(&buf, Point::new(0, 28), style).draw(display);

    let label = if sent { "TX: OK" } else { "TX: FAIL" };
    let _ = Text::new(label, Point::new(0, 44), style).draw(display);

    buf.clear();
    let _ = write!(buf, "CNT: {}", ack_count);
    let _ = Text::new(&buf, Point::new(0, 60), style).draw(display);

    let _ = display.flush();
}
