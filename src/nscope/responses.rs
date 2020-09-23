#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PowerState {
    PowerOff,
    PowerOn,
    Shorted,
    Overcurrent,
    Unknown,
}

impl From<u8> for PowerState {
    fn from(orig: u8) -> Self {
        match orig {
            0 => PowerState::PowerOff,
            1 => PowerState::PowerOn,
            2 => PowerState::Shorted,
            3 => PowerState::Overcurrent,
            _ => PowerState::Unknown
        }
    }
}

#[derive(Debug)]
pub(super) struct StatusResponse {
    fw_version: u8,
    pub(super) power_state: PowerState,
    pub(super) power_usage: u8,
    request_id: u8,
}

impl StatusResponse {
    pub(crate) fn new(buf: &[u8]) -> StatusResponse {
        StatusResponse {
            fw_version: buf[0] & 0x3F,
            power_state: PowerState::from((buf[0] & 0xC0) >> 6),
            power_usage: buf[1],
            request_id: buf[2],
        }
    }
}