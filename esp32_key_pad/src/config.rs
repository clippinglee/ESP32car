//! ESP32 遥控器引脚与硬件配置

// ---- nRF24L01 (SPI2) - 待接入 ----
pub const NRF_SCK: u8 = 18;
pub const NRF_MOSI: u8 = 23;
pub const NRF_MISO: u8 = 19;
pub const NRF_CSN: u8 = 5;
pub const NRF_CE: u8 = 4;

// ---- 按键 (低电平有效, 内置上拉) ----
pub const BTN_A: u8 = 0;
pub const BTN_B: u8 = 17;
pub const BTN_C: u8 = 25;
pub const BTN_D: u8 = 26;

// ---- I2C 12864 OLED (SDA=D22, SCL=D21) ----
pub const I2C_SDA: u8 = 22;
pub const I2C_SCL: u8 = 21;

// ---- LED ----
pub const LED_PIN: u8 = 2;

// ---- 摇杆 ADC (硬件直连) ----
// GPIO36 = ADC1_CH0 (X轴)
// GPIO39 = ADC1_CH3 (Y轴)

pub const ADC_MAX: u16 = 4095;
pub const JOY_CENTER: u16 = ADC_MAX / 2;
pub const JOY_DEADZONE: u16 = 100;
