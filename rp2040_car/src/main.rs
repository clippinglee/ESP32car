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

use defmt_rtt as _; // 必须, 提供 defmt 输出通道
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::spi::{Config as SpiConfig, Spi};
use embassy_time::{Duration, Instant, Ticker};
use panic_probe as _; // panic 处理器

use protocol::{ControlPacket, FAILSAFE_TIMEOUT_MS, TX_RATE_HZ};

mod config;
mod control;
mod motors;
mod radio;

/// 上次有效控制包到达时间戳(ms), 0 表示从未收到。
/// 用 AtomicU64 而非 MutexCell, 在单核上足够且更简单。
static LAST_PKT_MS: AtomicU64 = AtomicU64::new(0);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("RP2040 car controller booting");
    let p = embassy_rp::init(Default::default());

    // ---- 板载 LED: 链路状态指示 ----
    let mut led = Output::new(p.PIN_25, Level::Low);

    // ---- 电机 PWM (左右各一个 slice, 单通道 A) ----
    let make_cfg = || {
        let mut c = PwmConfig::default();
        c.top = config::PWM_TOP;
        c.divider = 125.into(); // 125MHz/125=1MHz tick; top=32767 => ~30Hz
        c
    };
    let left_pwm = Pwm::new_output_a(p.PWM_SLICE4, p.PIN_8, make_cfg());
    let right_pwm = Pwm::new_output_a(p.PWM_SLICE5, p.PIN_10, make_cfg());

    // 方向脚 (具体 PIN 类型直接传给 Motors)
    let lf = Output::new(p.PIN_12, Level::Low);
    let lr = Output::new(p.PIN_13, Level::Low);
    let rf = Output::new(p.PIN_14, Level::Low);
    let rr = Output::new(p.PIN_15, Level::Low);

    let mut motors = motors::Motors::new(left_pwm, right_pwm, lf, lr, rf, rr);
    motors.stop();

    // ---- nRF24L01 SPI (阻塞, 8MHz) ----
    // SPI0: SCK=GP2, MOSI=GP3, MISO=GP4; CSN=GP5 手动; CE=GP6
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 8_000_000;
    let _spi = Spi::new_blocking(p.SPI0, p.PIN_2, p.PIN_3, p.PIN_4, spi_cfg);
    let mut _csn = Output::new(p.PIN_5, Level::High); // 空闲拉高
    let mut _ce = Output::new(p.PIN_6, Level::Low);

    // TODO(radio): 用 rf24-rs 驱动实例化 nRF24L01, 配置为 PRX:
    //   - 设置地址 = protocol::PAIR_ADDR, 通道 = protocol::CHANNEL
    //   - 开 CRC, 动态载荷, 自动 ack
    //   - 起一个 task 轮询收包 -> radio::parse_payload() -> 更新 LAST_PKT_MS
    //     并把 ControlPacket 送入控制循环。
    //   骨架阶段: 无线收包未接入, 控制循环按"失控"处理(停车), 保证安全。

    // ---- 主循环: 失控保护 + 差速驱动 (100Hz) ----
    let mut ticker = Ticker::every(Duration::from_hz(TX_RATE_HZ));
    loop {
        // TODO(radio): 此处从收包 task 取最新 ControlPacket。
        //   骨架阶段 latest 恒为 None。
        let latest: Option<ControlPacket> = None;

        if let Some(pkt) = latest {
            LAST_PKT_MS.store(Instant::now().as_millis(), Ordering::Relaxed);
            let (left, right) = control::arcade_mix(pkt.axis_x, pkt.axis_y);
            motors.drive(left, right, pkt.speed_limit);
            led.set_high();
        } else {
            // 失控保护: 超时或从未收到包 -> 停车
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
