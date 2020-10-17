/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use super::commands::Command;
use super::Nscope;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AnalogWaveType {
    Sine = 0,
    Triangle = 1,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AnalogSignalPolarity {
    Unipolar,
    Bipolar,
}

/// Interface to an analog output
#[derive(Debug, Copy, Clone)]
pub struct AnalogOutput {
    pub is_on: bool,
    pub frequency: f64,
    pub amplitude: f64,
    pub wave_type: AnalogWaveType,
    pub polarity: AnalogSignalPolarity,
}

impl Default for AnalogOutput {
    fn default() -> Self {
        AnalogOutput {
            is_on: false,
            frequency: 1.0,
            amplitude: 1.0,
            wave_type: AnalogWaveType::Sine,
            polarity: AnalogSignalPolarity::Unipolar,
        }
    }
}

impl Nscope {
    pub fn set_ax_on(&mut self, on: bool) {
        self.analog_output[0].is_on = on;
        self.command_tx
            .send(Command::SetAnalogOutput {
                channel: 0,
                ax: self.analog_output[0],
            })
            .unwrap();
    }
}
