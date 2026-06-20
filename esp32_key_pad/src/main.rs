#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use esp_backtrace as _;
use esp_hal::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::delay::Block;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::spi::{Config as SpiConfig, Mode as SpiMode, Spi, Polarity};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{init, peripherals};
use esp_println::println;
use panic_probe as _;
use rf24::radio::prelude::*;
use rf24::radio::{RadioConfig, RF24};

use protocol::{button, ControlPacket, CHANNEL, PAIR_ADDR, TX_RATE_HZ};

mod config;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = peripherals::SYSTEM::conjure();
    let system = init!(peripherals);
    let timg0 = TimerGroup::new(system.timg0);
    esp_hal_embassy::init(timg0.timer0);

    println!("ESP32 remote controller booting");

    // ---- LED ----
    let mut led = Output::new(config::LED_PIN, Level::Low);

    // ---- Joystick ADC ----
    let mut adc_config = AdcConfig::new();
    let joy_x_pin = adc_config.enable_pin_analog(config::JOY_X_PIN, Attenuation::B11dB);
    let joy_y_pin = adc_config.enable_pin_analog(config::JOY_Y_PIN, Attenuation::B11dB);
    let mut adc = Adc::new(system.adc1, adc_config);

    // ---- Buttons (active low, internal pull-up) ----
    let mut btn_a = Input::new(config::BTN_A, Pull::Up);
    let mut btn_b = Input::new(config::BTN_B, Pull::Up);
    let mut btn_c = Input::new(config::BTN_C, Pull::Up);
    let mut btn_d = Input::new(config::BTN_D, Pull::Up);

    // ---- nRF24L01 SPI (VSPI/SPI2) ----
    let spi_cfg = SpiConfig {
        frequency: Rate::from_mhz(8),
        mode: SpiMode {
            polarity: Polarity::IdleLow,
            phase: esp_hal::spi::Phase::CaptureOnFirstTransition,
        },
        ..Default::default()
    };
    let spi = Spi::new(
        system.spi2,
        config::NRF_SCK,
        config::NRF_MOSI,
        config::NRF_MISO,
        config::NRF_CSN,
        spi_cfg,
    );

    let ce_pin = Output::new(config::NRF_CE, Level::Low);

    // ---- nRF24L01 初始化 (PTX 模式) ----
    let mut nrf = RF24::new(ce_pin, spi, Block::new());
    nrf.init().expect("nRF24 init failed");

    let config = RadioConfig::default()
        .with_channel(CHANNEL)
        .with_tx_address(&PAIR_ADDR)
        .with_ack_payloads(true)
        .with_dynamic_payloads(true)
        .with_auto_retries(2, 5);

    nrf.with_config(&config).expect("nRF24 config failed");
    nrf.as_tx(Some(&PAIR_ADDR)).expect("nRF24 as_tx failed");

    println!("nRF24 ready (PTX, ch={}, addr={:?})", CHANNEL, PAIR_ADDR);

    // ---- Main loop: sample inputs + TX at TX_RATE_HZ ----
    let mut seq: u16 = 0;
    let mut speed_limit: u8 = 50; // default 50%
    let mut ticker = Ticker::every(Duration::from_hz(TX_RATE_HZ));

    loop {
        // Sample joystick
        let raw_x: u16 = adc.read_blocking(&joy_x_pin);
        let raw_y: u16 = adc.read_blocking(&joy_y_pin);
        let axis_x = analog_to_axis(raw_x);
        let axis_y = analog_to_axis(raw_y);

        // Sample buttons
        let mut buttons: u16 = 0;
        if btn_a.is_low() {
            buttons |= button::BTN_A;
        }
        if btn_b.is_low() {
            buttons |= button::BTN_B;
        }
        if btn_c.is_low() {
            buttons |= button::BTN_C;
        }
        if btn_d.is_low() {
            buttons |= button::BTN_D;
        }

        // BTN_A + BTN_D 同时按: 切换限速档位
        if buttons & button::BTN_A != 0 && buttons & button::BTN_D != 0 {
            speed_limit = match speed_limit {
                25 => 50,
                50 => 75,
                75 => 100,
                _ => 25,
            };
        }

        // 构造控制包
        let pkt = ControlPacket::new(seq, axis_x, axis_y, buttons, speed_limit);
        seq = seq.wrapping_add(1);

        // 序列化并发送
        let mut payload = [0u8; 32];
        let result = postcard::to_slice(&pkt, &mut payload);
        let sent = if let Ok(serialized) = result {
            nrf.send(serialized, false).unwrap_or(false)
        } else {
            false
        };

        if sent {
            led.set_high();
        } else {
            led.set_low();
            // 发送失败: 重新进入 TX 模式
            nrf.as_tx(Some(&PAIR_ADDR)).ok();
        }

        ticker.next().await;
    }
}

/// ADC 原始值 -> 有符号轴值 (-1000..1000), 带死区
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
