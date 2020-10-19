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
use std::sync::mpsc;

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
    pub fn get_ax(&self) -> AnalogOutput {
        self.analog_output.read().unwrap()[0]
    }

    pub fn set_ax_on(&self, on: bool) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.analog_output.read().unwrap()[0];
        requested_ax.is_on = on;

        // Create a method for the backend to communicate back to us what we want
        let (tx, rx) = mpsc::channel::<AnalogOutput>();

        // Send the command to the backend
        self.command_tx
            .send(Command::SetAnalogOutput {
                channel: 0,
                ax: requested_ax,
                actual: None,
                sender: tx,
            })
            .unwrap();

        // Wait for the backend to receive a response and return the result
        let actual_ax = rx.recv().unwrap();
        let mut writer = self.analog_output.write().unwrap();
        writer[0] = actual_ax;
        actual_ax
    }
}

pub(crate) fn update_analog_output(
    usb_buf: &mut [u8; 65],
    _: usize,
    ax: AnalogOutput,
) -> Option<AnalogOutput> {
    usb_buf[1] = 0x02;

    if ax.is_on {
        usb_buf[3] = ax.wave_type as u8;
        usb_buf[3] |= 0x80;

        let scaled_frequency = ax.frequency * 2.0_f64.powi(28) / 4000000.0;
        let freq_register: u32 = scaled_frequency as u32;

        usb_buf[4] = (freq_register & 0x00FF) as u8;
        usb_buf[5] = ((freq_register & 0x3F00) >> 8) as u8;
        usb_buf[6] = (freq_register >> 14 & 0x00FF) as u8;
        usb_buf[7] = ((freq_register >> 14 & 0x3F00) >> 8) as u8;

        if ax.amplitude < 0.0 {
            usb_buf[3] |= 0x2;
        }
        let rf = 49900.0;
        let vin = 0.6;
        let rm = 75.0;
        let rv = 100000.0 / 257.0;

        let gain: u8 = match ax.polarity {
            AnalogSignalPolarity::Unipolar => ((vin * rf / ax.amplitude.abs() - rm) / rv) as u8,
            AnalogSignalPolarity::Bipolar => {
                ((vin * rf / 2.0 / ax.amplitude.abs() - rm) / rv) as u8
            }
        };

        let offset: u8 = ((rm + rv * (gain as f64)) / (rm + rv * (gain as f64) + rf)
            * ax.amplitude.abs()
            * 255.0
            / 3.05) as u8;

        usb_buf[8] = gain;
        usb_buf[9] = offset;
    } else {
        usb_buf[3] = 0xFF;
    }
    Some(ax)
}
