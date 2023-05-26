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
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};

use log::trace;

use super::AnalogInput;
use super::Command;
use super::commands::ScopeCommand;
use super::Nscope;
use super::Trigger;

#[derive(Debug, Default, Clone)]
pub struct Sample {
    pub time_since_start: f64,
    pub data: [Option<f64>; Sample::num_channels() as usize],
}

impl Sample {
    pub const fn num_channels() -> u32 { 4 }

    pub fn clear(&mut self) {
        self.data = [None; Sample::num_channels() as usize];
    }
}

#[derive(Debug)]
pub(super) struct DataRequest {
    pub channels: [AnalogInput; 4],
    pub sample_rate_hz: f64,
    pub remaining_samples: Arc<RwLock<usize>>,
    pub trigger: Trigger,
    pub sender: Sender<Sample>,
    pub stop_recv: Receiver<()>,
}

#[derive(Debug)]
pub struct RequestHandle {
    pub receiver: Receiver<Sample>,
    samples_remaining: Arc<RwLock<usize>>,
    stop_send: Sender<()>,
}

#[derive(Debug)]
pub(super) struct StopRequest {}

impl Nscope {
    pub fn request(&self, sample_rate_hz: f64, number_of_samples: usize, trigger: Option<Trigger>) -> RequestHandle {
        let (tx, rx) = mpsc::channel::<Sample>();
        let (stop_send, stop_recv) = mpsc::channel::<()>();

        let remaining_samples = Arc::new(RwLock::new(number_of_samples));
        let command = Command::RequestData(DataRequest {
            channels: [self.ch1, self.ch2, self.ch3, self.ch4],
            sample_rate_hz,
            remaining_samples: remaining_samples.clone(),
            trigger: trigger.unwrap_or(Trigger::default()),
            sender: tx,
            stop_recv,
        });

        self.command_tx.send(command).unwrap();

        RequestHandle {
            receiver: rx,
            samples_remaining: remaining_samples,
            stop_send,
        }
    }
}

impl RequestHandle {
    pub fn remaining_samples(&self) -> usize {
        *self.samples_remaining.read().unwrap()
    }

    pub fn stop(&self) {
        self.stop_send.send(()).ok();
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

        let total_samples = *self.remaining_samples.read().unwrap();
        if samples_between_records < 250 && total_samples * num_channels_on > 3200 {
            return Err("Data not recordable".into());
        }


        usb_buf[3..=6].copy_from_slice(&samples_between_records.to_le_bytes());
        usb_buf[7..=10].copy_from_slice(&total_samples.to_le_bytes());
        trace!("Requesting {} samples with {} samples between records", total_samples, samples_between_records);

        if self.trigger.is_enabled {
            usb_buf[11] = (self.trigger.source_channel | (self.trigger.trigger_type.value() << 2)) as u8;

            if !(0..4usize).contains(&self.trigger.source_channel) {
                return Err("Invalid trigger channel".into());
            }
            let trigger_channel = self.channels[self.trigger.source_channel];
            let trigger_level = trigger_channel.measurement_from_voltage(self.trigger.trigger_level);
            if !(105..3990).contains(&trigger_level) {
                return Err("Trigger level is outside operating range of the channel".into());
            }
            let trigger_level = trigger_level as u16;
            usb_buf[11] |= ((trigger_level & 0x000F) << 4) as u8;
            usb_buf[12] = ((trigger_level & 0x0FF0) >> 4) as u8;

            let trigger_delay: u16 = match num_channels_on {
                1 => { 4 * self.trigger.trigger_delay_us / samples_between_records }
                2 => { 2 * self.trigger.trigger_delay_us / samples_between_records }
                3..=4 => { self.trigger.trigger_delay_us / samples_between_records }
                _ => { 1 }
            } as u16;
            usb_buf[13..=14].copy_from_slice(&trigger_delay.to_le_bytes());
        } else {
            usb_buf[11..=14].fill(0);
        }


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

    fn handle_rx(&self, usb_buf: &[u8; 64]) {
        let number_received_samples = usb_buf[3] as usize;

        {
            let mut remaining_samples = self.remaining_samples.write().unwrap();
            *remaining_samples -= number_received_samples;
            trace!("Received {} samples, {} samples remaining", number_received_samples, remaining_samples);
        }


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

                    trace!("Ch{}: ADCData: {} Vi: {}", i+1, adc_data, ch.voltage_from_measurement(adc_data));
                    sample.data[i] = Some(ch.voltage_from_measurement(adc_data));
                    total_parsed_readings += 1;
                }
            }

            self.sender.send(sample).unwrap();
        }
    }

    fn is_finished(&self) -> bool {
        *self.remaining_samples.read().unwrap() == 0
    }
}

impl ScopeCommand for StopRequest {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x05;
        Ok(())
    }
    fn handle_rx(&self, _usb_buf: &[u8; 64]) {}
    fn is_finished(&self) -> bool { true }
}