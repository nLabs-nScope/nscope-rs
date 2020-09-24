/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

/// Storage class for the nScope power status
#[derive(Debug, Copy, Clone)]
pub struct PowerStatus {
    pub state: PowerState,
    pub usage: f32,
}

impl PowerStatus {
    pub(super) fn new() -> Self {
        PowerStatus {
            state: PowerState::Unknown,
            usage: 0.0,
        }
    }
}

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
            _ => PowerState::Unknown,
        }
    }
}
