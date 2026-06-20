use protocol::TelemetryPacket;

use crate::TELEM_SIGNAL;

/// 尝试从 nRF24L01 的 ACK payload 中读取遥测包。
/// 调用前需确保 nrf 可用且已开启 ack_payloads。
pub fn try_read_ack(nrf: &mut rf24::radio::RF24<impl embedded_hal::spi::SpiDevice, impl embedded_hal::digital::OutputPin, impl embedded_hal::delay::DelayNs>) -> Option<TelemetryPacket> {
    use rf24::radio::prelude::EsbFifo;
    use rf24::radio::prelude::EsbRadio;

    if nrf.available().ok()? {
        let mut buf = [0u8; 32];
        let len = nrf.read(&mut buf, None).ok()?;
        if len > 0 {
            if let Ok(pkt) = postcard::from_bytes::<TelemetryPacket>(&buf[..len as usize]) {
                Some(pkt)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
