//! RP2040 车控固件入口 (embassy)
//!
//! 角色: nRF24L01 PRX(常听)。接收遥控器 ControlPacket -> 差速驱动四驱电机。
//! 失控保护: 超过 FAILSAFE_TIMEOUT_MS 无有效包则停车。
//!
//! 编译: 见 .cargo/config.toml, target=thumbv6m-none-eabi
//! 烧录: probe-rs run --chip RP2040 (或 elf2uf2-rs)
//!
//! 状态: 电机/PWM/方向脚/失控保护/LED 已实现并通过编译。
//!       nRF24L01 SPI 已初始化, 收发驱动实例化见 TODO(下文)。

#![no_std]
#![no_main]

use portable_atomic::AtomicU64;
use core::sync::atomic::Ordering;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::spi::{Config as SpiConfig, Spi};
use embassy_time::{Duration, Instant, Ticker};
use panic_probe as _;
use rf24::radio::prelude::*;
use rf24::radio::{RadioConfig, RF24};

use protocol::{ControlPacket, PAIR_ADDR, CHANNEL, FAILSAFE_TIMEOUT_MS, TX_RATE_HZ};

mod config;
mod control;
mod motors;
mod radio;

static LAST_PKT_MS: AtomicU64 = AtomicU64::new(0);
static LATEST_SEQ: AtomicU64 = AtomicU64::new(0);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("RP2040 car controller booting");
    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    // ---- 电机 PWM ----
    let make_cfg = || {
        let mut c = PwmConfig::default();
        c.top = config::PWM_TOP;
        c.divider = 125.into();
        c
    };
    let left_pwm = Pwm::new_output_a(p.PWM_SLICE4, p.PIN_8, make_cfg());
    let right_pwm = Pwm::new_output_a(p.PWM_SLICE5, p.PIN_10, make_cfg());

    let lf = Output::new(p.PIN_12, Level::Low);
    let lr = Output::new(p.PIN_13, Level::Low);
    let rf = Output::new(p.PIN_14, Level::Low);
    let rr = Output::new(p.PIN_15, Level::Low);

    let mut motors = motors::Motors::new(left_pwm, right_pwm, lf, lr, rf, rr);
    motors.stop();

    // ---- nRF24L01 SPI (阻塞, 8MHz) ----
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 8_000_000;
    let spi_bus = Spi::new_blocking(p.SPI0, p.PIN_2, p.PIN_3, p.PIN_4, spi_cfg);
    let csn = Output::new(p.PIN_5, Level::High);
    let ce = Output::new(p.PIN_6, Level::Low);

    let spi_dev = embedded_hal_bus::spi::ExclusiveDevice::new(spi_bus, csn, radio::BlockingDelay);

    // ---- nRF24L01 初始化 (PRX 模式) ----
    let mut nrf = RF24::new(ce, spi_dev, radio::BlockingDelay);
    nrf.init().expect("nRF24 init failed");

    let nrf_config = RadioConfig::default()
        .with_channel(CHANNEL)
        .with_tx_address(&PAIR_ADDR)
        .with_rx_address(0, &PAIR_ADDR)
        .with_auto_retries(2, 5)
        .with_ack_payloads(true)
        .with_dynamic_payloads(true);

    nrf.with_config(&nrf_config).expect("nRF24 config failed");
    nrf.as_rx().expect("nRF24 as_rx failed");

    defmt::info!("nRF24 ready (PRX, ch={}, addr={:?})", CHANNEL, PAIR_ADDR);

    // ---- 主循环: 失控保护 + 差速驱动 (100Hz) ----
    let mut ticker = Ticker::every(Duration::from_hz(TX_RATE_HZ));
    loop {
        // 尝试接收控制包
        let mut latest: Option<ControlPacket> = None;
        if nrf.available().unwrap_or(false) {
            let mut buf = [0u8; 32];
            if let Ok(len) = nrf.read(&mut buf, None) {
                if len > 0 {
                    if let Ok(pkt) = radio::parse_payload(&buf[..len as usize]) {
                        LAST_PKT_MS.store(Instant::now().as_millis(), Ordering::Relaxed);
                        LATEST_SEQ.store(pkt.seq as u64, Ordering::Relaxed);
                        latest = Some(pkt);
                    }
                }
            }
        }

        if let Some(pkt) = latest {
            let (left, right) = control::arcade_mix(pkt.axis_x, pkt.axis_y);
            motors.drive(left, right, pkt.speed_limit);
            led.set_high();
        } else {
            let last = LAST_PKT_MS.load(Ordering::Relaxed);
            let timed_out = if last == 0 {
                true
            } else {
                Instant::now().as_millis().wrapping_sub(last) > FAILSAFE_TIMEOUT_MS
            };
            if timed_out {
                motors.stop();
                led.set_low();
            }
        }

        ticker.next().await;
    }
}
