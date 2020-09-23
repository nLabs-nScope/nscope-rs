/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use super::responses;

pub struct NscopeData {
    pub power_state: responses::PowerState,
    pub power_usage: f32,
}

impl Default for NscopeData {
    fn default() -> Self {
        Self::new()
    }
}

impl NscopeData {
    pub fn new() -> Self {
        NscopeData {
            power_state: responses::PowerState::Unknown,
            power_usage: 0.0,
        }
    }
}
