/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

pub struct PowerStatus {
    pub state: State,
    pub usage: f32,
}

impl PowerStatus {
    pub(super) fn new() -> Self {
        PowerStatus {
            state: State::Unknown,
            usage: 0.0,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    PowerOff,
    PowerOn,
    Shorted,
    Overcurrent,
    Unknown,
}

impl From<u8> for State {
    fn from(orig: u8) -> Self {
        match orig {
            0 => State::PowerOff,
            1 => State::PowerOn,
            2 => State::Shorted,
            3 => State::Overcurrent,
            _ => State::Unknown,
        }
    }
}
