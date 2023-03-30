/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use log::trace;

use super::commands::ScopeCommand;
use super::Command;
use super::Nscope;
use super::AnalogInput;
use super::Trigger;

#[derive(Debug, Default, Clone)]
pub struct Sample {
    pub time_since_start: f64,
    pub data: [Option<f64>; Sample::num_channels() as usize],
}

impl Sample {
    pub const fn num_channels() -> u32 {
        return 4;
    }
}

#[derive(Debug)]
pub(super) struct DataRequest {
    pub channels: [AnalogInput; 4],
    pub sample_rate_hz: f64,
    pub remaining_samples: u32,
    pub trigger: Trigger,
    pub sender: Sender<Sample>,
}


impl Nscope {
    pub fn request(&self, sample_rate_hz: f64, number_of_samples: u32) -> Receiver<Sample> {
        let (tx, rx) = mpsc::channel::<Sample>();

        let command = Command::RequestData(DataRequest {
            channels: [self.ch1, self.ch2, self.ch3, self.ch4],
            sample_rate_hz,
            remaining_samples: number_of_samples,
            trigger: self.trigger,
            sender: tx,
        });

        self.command_tx.send(command).unwrap();

        rx
    }
}

impl ScopeCommand for DataRequest {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x08;

        let num_channels_on = self.channels.iter().filter(|&ch| ch.is_on).count();


        let samples_between_records: u32 = match num_channels_on {
            0 => { return Err("No scope channels are on".into()); }
            1 => { (4_000_000.0 / self.sample_rate_hz) as u32 }
            2 => { (2_000_000.0 / self.sample_rate_hz) as u32 }
            3 | 4 => { (1_000_000.0 / self.sample_rate_hz) as u32 }
            _ => { return Err("Unexpected number of channels are on".into()); }
        };

        if samples_between_records < 250 && self.remaining_samples * num_channels_on as u32 > 3200 {
            return Err("Data not recordable".into());
        }


        usb_buf[3..7].copy_from_slice(&samples_between_records.to_le_bytes());
        usb_buf[7..11].copy_from_slice(&self.remaining_samples.to_le_bytes());

        trace!("Requesting {} samples with {} samples between records", self.remaining_samples, samples_between_records);

        for (i, ch) in self.channels.iter().enumerate() {
            if ch.is_on {
                usb_buf[15 + i] = ch.gain_setting;
                usb_buf[19 + i] = ch.offset_setting;
            } else {
                usb_buf[15 + i] = 0xFF;
            }
        }


        Ok(())
    }

    fn handle_rx(mut self, usb_buf: &[u8; 64]) -> Option<Self> {
        let number_received_samples = usb_buf[3] as u32;

        self.remaining_samples -= number_received_samples;
        trace!("Received {} samples, {} samples remaining", number_received_samples, self.remaining_samples);


        let mut total_parsed_readings: usize = 0;

        for _ in 0..number_received_samples {
            let mut sample = Sample {
                time_since_start: 0.0,
                data: [None; 4],
            };

            for (i, ch) in self.channels.iter().enumerate() {
                if ch.is_on {

                    let byte = 4 + total_parsed_readings / 2 * 3;

                    let adc_data = match total_parsed_readings & 1 {
                        0 => usb_buf[byte] as u16 | ((usb_buf[byte + 1] & 0xF) as u16) << 8,
                        1 => usb_buf[byte + 1] as u16 >> 4 | (usb_buf[byte + 2] as u16) << 4,
                        _ => panic!("Unexpected behavior of odd/even bitmask")
                    };

                    let measured_voltage = adc_data as f64 * 3.3 / 4095.0;
                    trace!("Ch{}: ADCData: {} Vm: {}, Vi: {}", i+1, adc_data, measured_voltage, ch.voltage_from_measurement(measured_voltage));
                    sample.data[i] = Some(ch.voltage_from_measurement(measured_voltage));
                    total_parsed_readings += 1;
                }
            }

            self.sender.send(sample).unwrap();
        }


        if self.remaining_samples > 0 {
            return Some(self);
        }

        None
    }
}