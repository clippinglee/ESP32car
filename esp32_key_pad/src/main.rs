#![no_std]
#![no_main]

esp_bootloader_esp_idf::esp_app_desc!();

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Config as HalConfig;
use esp_println::println;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::pixelcolor::BinaryColor;

mod config;
mod emotions;

fn pin(n: u8) -> AnyPin<'static> {
    unsafe { AnyPin::steal(n) }
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(HalConfig::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    println!("Emoji display start");

    let mut led = Output::new(pin(config::LED_PIN), Level::Low, OutputConfig::default());

    // Joystick
    let mut adc_config = AdcConfig::new();
    let mut joy_x_pin = adc_config.enable_pin(peripherals.GPIO36, Attenuation::_11dB);
    let mut joy_y_pin = adc_config.enable_pin(peripherals.GPIO39, Attenuation::_11dB);
    let mut adc = Adc::new(peripherals.ADC1, adc_config);

    // Buttons
    let input_cfg = InputConfig::default().with_pull(Pull::Up);
    let btn_a = Input::new(pin(config::BTN_A), input_cfg);
    let btn_b = Input::new(pin(config::BTN_B), input_cfg);
    let btn_c = Input::new(pin(config::BTN_C), input_cfg);
    let btn_d = Input::new(pin(config::BTN_D), input_cfg);

    // OLED
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default().with_frequency(Rate::from_khz(400)))
        .expect("I2C failed")
        .with_sda(pin(config::I2C_SDA))
        .with_scl(pin(config::I2C_SCL));
    let interface = I2CDisplayInterface::new(i2c);
    let mut oled = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let _ = oled.init();
    println!("OLED ready");

    let mut emotion_idx: usize = 0;
    let mut frame: usize = 0;
    let mut tick_count: u32 = 0;
    let mut speed_limit: u8 = 50;

    // 10fps动画, 3秒切换表情 (30帧)
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        // ADC
        let raw_x: u16 = nb::block!(adc.read_oneshot(&mut joy_x_pin)).unwrap();
        let raw_y: u16 = nb::block!(adc.read_oneshot(&mut joy_y_pin)).unwrap();
        let _axis_x = analog_to_axis(raw_x);
        let _axis_y = analog_to_axis(raw_y);

        // Buttons
        let mut buttons: u16 = 0;
        if btn_a.is_low() { buttons |= 1 << 0; }
        if btn_b.is_low() { buttons |= 1 << 1; }
        if btn_c.is_low() { buttons |= 1 << 2; }
        if btn_d.is_low() { buttons |= 1 << 3; }

        if buttons & (1 << 0) != 0 && buttons & (1 << 3) != 0 {
            speed_limit = match speed_limit { 25 => 50, 50 => 75, 75 => 100, _ => 25 };
        }

        // LED blink
        if tick_count % 50 == 0 { led.toggle(); }

        // Animation: 每3帧切换眼睛状态, 每30帧切换表情
        if tick_count % 3 == 0 {
            frame = (frame + 1) % 2;
        }
        if tick_count % 30 == 0 && tick_count > 0 {
            emotion_idx = (emotion_idx + 1) % emotions::EMOTIONS.len();
            frame = 0;
        }

        // 只显示表情，不显示数值
        let _ = oled.clear(BinaryColor::Off);
        let emotion = &emotions::EMOTIONS[emotion_idx];
        emotions::draw_emotion(&mut oled, emotion, frame);
        let _ = oled.flush();

        tick_count += 1;
        ticker.next().await;
    }
}

fn analog_to_axis(raw: u16) -> i16 {
    let center = config::JOY_CENTER as i32;
    let dead = config::JOY_DEADZONE as i32;
    let diff = (raw as i32) - center;
    if diff.abs() < dead {
        0
    } else {
        let sign = if diff > 0 { 1 } else { -1 };
        let magnitude = (diff.abs() - dead) as i32;
        let max_mag = (config::ADC_MAX / 2) as i32 - dead;
        let scaled = (magnitude * 1000 / max_mag).min(1000);
        (sign * scaled) as i16
    }
}
