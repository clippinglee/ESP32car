use protocol::{ControlPacket, MAX_PAYLOAD};

pub enum RadioError {
    BadLen,
    BadPacket,
    Decode,
}

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

pub struct BlockingDelay;

impl embedded_hal::delay::DelayNs for BlockingDelay {
    fn delay_ns(&mut self, ns: u32) {
        let cycles = (ns as u64 * 125_000_000) / 1_000_000_000;
        cortex_m::asm::delay(cycles as u32);
    }
}
