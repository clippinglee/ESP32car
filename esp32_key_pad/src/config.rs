//! ESP32 遥控器引脚与硬件配置
//!
//! 所有 GPIO 编号集中在此, 便于按实际接线调整。
//! 接线说明见 README.md。

// ---- nRF24L01 (VSPI/SPI2) ----
// esp-hal 1.0: Spi::new(...).with_sck(...).with_mosi(...).with_miso(...) 构建器模式
pub const NRF_SCK: u8 = 18;
pub const NRF_MOSI: u8 = 23;
pub const NRF_MISO: u8 = 19;
pub const NRF_CSN: u8 = 5;  // SPI 片选(手动拉低)
pub const NRF_CE: u8 = 4;   // nRF24 收发使能
pub const NRF_IRQ: u8 = 16; // nRF24 中断(可选)

// ---- 摇杆 (ADC1) ----
// ESP32 ADC1 通道: CH0=GPIO36, CH3=GPIO39, CH6=GPIO34, CH7=GPIO35
// GPIO36/39 是仅输入的 RTC 引脚, ADC1 支持。
pub const JOY_X_PIN: u8 = 36; // ADC1_CH0, 左右
pub const JOY_Y_PIN: u8 = 39; // ADC1_CH3, 前后

// ---- 按键 (低电平有效, 内置上拉) ----
pub const BTN_A: u8 = 0;   // Boot 按钮(板载)
pub const BTN_B: u8 = 17;
pub const BTN_C: u8 = 25;
pub const BTN_D: u8 = 26;

// ---- LED ----
pub const LED_PIN: u8 = 2; // ESP32 devkit 板载 LED

/// ADC 原始值范围 (12 bit: 0..4095)
pub const ADC_MAX: u16 = 4095;

/// 摇杆中值 (归一化后的零点, 允许小偏移)
pub const JOY_CENTER: u16 = ADC_MAX / 2;

/// 死区: 摇杆偏移小于此值视为零
pub const JOY_DEADZONE: u16 = 100;
