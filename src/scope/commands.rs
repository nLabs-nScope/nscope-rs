/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use super::analog_output::{AnalogOutput, AnalogSignalPolarity};

pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

#[derive(Copy, Clone)]
pub enum Command {
    Quit,
    SetAnalogOutput { channel: i8, ax: AnalogOutput },
}

pub(super) fn generate_packet(request_id: u8, cmd: Command) -> [u8; 65] {
    let mut buf: [u8; 65] = [0; 65];
    match cmd {
        Command::SetAnalogOutput { channel: _, ax } => {
            buf[0] = 0x00;
            buf[1] = 0x02;
            buf[2] = request_id;

            if ax.is_on {
                buf[3] = ax.wave_type as u8;
                buf[3] |= 0x80;

                let scaled_frequency = ax.frequency * 2.0_f64.powi(28) / 4000000.0;
                let freq_register: u32 = scaled_frequency as u32;

                buf[4] = (freq_register & 0x00FF) as u8;
                buf[5] = ((freq_register & 0x3F00) >> 8) as u8;
                buf[6] = (freq_register >> 14 & 0x00FF) as u8;
                buf[7] = ((freq_register >> 14 & 0x3F00) >> 8) as u8;

                if ax.amplitude < 0.0 {
                    buf[3] |= 0x2;
                }
                let rf = 49900.0;
                let vin = 0.6;
                let rm = 75.0;
                let rv = 100000.0 / 257.0;

                let gain: u8 = match ax.polarity {
                    AnalogSignalPolarity::Unipolar => {
                        ((vin * rf / ax.amplitude.abs() - rm) / rv) as u8
                    }
                    AnalogSignalPolarity::Bipolar => {
                        ((vin * rf / 2.0 / ax.amplitude.abs() - rm) / rv) as u8
                    }
                };

                let offset: u8 = ((rm + rv * (gain as f64)) / (rm + rv * (gain as f64) + rf)
                    * ax.amplitude.abs()
                    * 255.0
                    / 3.05) as u8;

                buf[8] = gain;
                buf[9] = offset;
            } else {
                buf[3] = 0xFF;
            }
        }
        _ => {}
    };
    buf
}
