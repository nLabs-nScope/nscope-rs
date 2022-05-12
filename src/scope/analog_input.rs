/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

mod voltages;

#[derive(Debug, Copy, Clone)]
pub struct AnalogInput {
    pub is_on: bool,
    pub(crate) gain_setting: u8,
    pub(crate) offset_setting: u8,
}

impl Default for AnalogInput {
    fn default() -> Self {
        let mut analog_input = AnalogInput {
            is_on: true,
            gain_setting: 0,
            offset_setting: 0
        };
        analog_input.set_range(-5.0, 5.0);
        analog_input
    }
}

