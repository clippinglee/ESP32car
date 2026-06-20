//! 智能小车共享无线协议
//!
//! 遥控器(ESP32, PTX) -> 车控(RP2040, PRX) 的控制包，
//! 以及车 -> 遥控器的遥测 ack。
//! 用 postcard 序列化，payload 必须 <= nRF24L01 单包上限(32 字节)。

#![no_std]

use serde::{Deserialize, Serialize};

/// 协议魔数，用于快速过滤噪声包
pub const MAGIC: u8 = 0xA5;

/// nRF24L01 单包有效载荷上限(动态载荷模式下)
pub const MAX_PAYLOAD: usize = 32;

/// 配对地址(5 字节)。遥控器与车必须一致。
pub const PAIR_ADDR: [u8; 5] = [0xE7, 0xE7, 0xE7, 0xE7, 0xE7];

/// 无线信道(0..=125, 对应 2400..2525 MHz)。两端必须一致。
pub const CHANNEL: u8 = 76;

/// 遥控器发送频率(Hz)
pub const TX_RATE_HZ: u64 = 100;

/// 车控失控保护超时: 超过此时间无有效包则停车
pub const FAILSAFE_TIMEOUT_MS: u64 = 300;

/// 遥控 -> 车 控制包
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ControlPacket {
    pub magic: u8,
    /// 序列号，每包 +1，用于丢包检测
    pub seq: u16,
    /// 摇杆 X: -1000..1000 (右为正)
    pub axis_x: i16,
    /// 摇杆 Y: -1000..1000 (前为正)
    pub axis_y: i16,
    /// 按键位图 (bit0 = 按键0, ...)
    pub buttons: u16,
    /// 限速 0..=100 (%)
    pub speed_limit: u8,
    /// CRC16 校验 (CRC-16/CCITT-FALSE)
    pub crc: u16,
}

impl ControlPacket {
    pub fn new(seq: u16, axis_x: i16, axis_y: i16, buttons: u16, speed_limit: u8) -> Self {
        let mut p = Self {
            magic: MAGIC,
            seq,
            axis_x: clamp_i16(axis_x),
            axis_y: clamp_i16(axis_y),
            buttons,
            speed_limit: if speed_limit > 100 { 100 } else { speed_limit },
            crc: 0,
        };
        p.crc = p.compute_crc();
        p
    }

    /// 序列化各字段(除 crc 外)，计算 CRC。校验失败返回 false。
    pub fn verify(&self) -> bool {
        self.magic == MAGIC && self.compute_crc() == self.crc
    }

    fn compute_crc(&self) -> u16 {
        // CRC-16/CCITT-FALSE, poly=0x1021, init=0xFFFF
        let mut crc: u16 = 0xFFFF;
        for &b in [
            self.magic,
            (self.seq & 0xFF) as u8,
            (self.seq >> 8) as u8,
            (self.axis_x & 0xFF) as u8,
            (self.axis_x >> 8) as u8,
            (self.axis_y & 0xFF) as u8,
            (self.axis_y >> 8) as u8,
            (self.buttons & 0xFF) as u8,
            (self.buttons >> 8) as u8,
            self.speed_limit,
        ]
        .iter()
        {
            crc ^= (b as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }
}

/// 车 -> 遥控器 遥测包 (作为 ack payload 回传)
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct TelemetryPacket {
    /// 电池电压(mV), 0 表示未知
    pub battery_mv: u16,
    /// 最后收到的控制包序列号
    pub last_seq: u16,
    /// 状态位图: bit0=链路正常 bit1=电机使能 bit2=低电压告警
    pub status: u8,
}

/// 按键位定义
pub mod button {
    pub const BTN_A: u16 = 1 << 0;
    pub const BTN_B: u16 = 1 << 1;
    pub const BTN_C: u16 = 1 << 2;
    pub const BTN_D: u16 = 1 << 3;
    pub const BTN_SHIFT: u16 = 1 << 4;
}

fn clamp_i16(v: i16) -> i16 {
    if v > 1000 { 1000 } else if v < -1000 { -1000 } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_roundtrip() {
        let p = ControlPacket::new(42, 300, -200, button::BTN_A, 75);
        assert!(p.verify());
    }

    #[test]
    fn detects_corruption() {
        let mut p = ControlPacket::new(1, 0, 0, 0, 50);
        p.axis_x = 999; // 篡改后 crc 不再匹配
        assert!(!p.verify());
    }
}
