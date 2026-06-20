//! RP2040 车控引脚与硬件配置
//!
//! 所有 GPIO 编号集中在此, 便于按实际接线调整。
//! 接线说明见 README.md。

// ---- nRF24L01 (SPI0, 阻塞 SPI + 手动 CSN) ----
// embassy-rp Spi::new_blocking 参数顺序: (spi, sck, mosi, miso)
pub const NRF_SCK: usize = 2;
pub const NRF_MOSI: usize = 3;
pub const NRF_MISO: usize = 4;
pub const NRF_CSN: usize = 5; // SPI 片选(手动拉低)
pub const NRF_CE: usize = 6;  // nRF24 收发使能
pub const NRF_IRQ: usize = 7; // nRF24 中断(可选, 当前未用)

// ---- L298N ×2 四驱 ----
// 两个 slice 各取一个通道做左右轮 PWM, 避免单实例无法分别调速:
//   左轮 PWM: GP8  = slice4 channel A
//   右轮 PWM: GP11 = slice5 channel A
//
// 方向脚(每侧 IN1/IN2):
//   左侧: GP12(正转), GP13(反转)
//   右侧: GP14(正转), GP15(反转)
// 接线时把同侧两片 L298N 的 EN 并联到对应 PWM, IN1/IN2 并联到方向脚。
pub const PWM_LEFT_PIN: usize = 8;   // slice4 chA
pub const PWM_RIGHT_PIN: usize = 10; // slice5 chA

pub const DIR_LEFT_FWD: usize = 12;
pub const DIR_LEFT_REV: usize = 13;
pub const DIR_RIGHT_FWD: usize = 14;
pub const DIR_RIGHT_REV: usize = 15;

// 板载 LED (Pico GP25)
pub const LED_PIN: usize = 25;

/// PWM 计数上限, 用于把 0..1000 的速度映射到 duty。
pub const PWM_TOP: u16 = 32767;
