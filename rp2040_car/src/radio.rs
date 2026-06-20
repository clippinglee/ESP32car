//! nRF24L01 无线接收 (PRX 角色) — 协议层
//!
//! 车控端常听模式: 收到原始载荷 -> 解析校验 -> 通过 channel 送给控制任务。
//! 失控保护由控制循环根据"最后一次有效包时间"判断, 见 main.rs。
//!
//! 注: rf24-rs 驱动的硬件层实例化在 main.rs 中完成。本模块只负责
//!     把字节流解析为 ControlPacket, 以及提供便于驱动配置的常量。

use protocol::{ControlPacket, MAX_PAYLOAD};

/// 无线链路解析错误
#[derive(defmt::Format)]
pub enum RadioError {
    /// 载荷长度超过 nRF24L01 单包上限
    BadLen,
    /// 魔数/CRC 校验失败(噪声包或损坏)
    BadPacket,
    /// postcard 反序列化失败
    Decode,
}

/// 把一个原始载荷解析为 ControlPacket。
/// 会校验长度、魔数、CRC。
pub fn parse_payload(buf: &[u8]) -> Result<ControlPacket, RadioError> {
    if buf.is_empty() || buf.len() > MAX_PAYLOAD {
        return Err(RadioError::BadLen);
    }
    match postcard::from_bytes::<ControlPacket>(buf) {
        Ok(pkt) if pkt.verify() => Ok(pkt),
        Ok(_) => Err(RadioError::BadPacket),
        Err(_) => Err(RadioError::Decode),
    }
}
