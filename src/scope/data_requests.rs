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

use crate::scope::NscopeState;
use super::commands::ScopeCommand;
use super::Command;
use super::Nscope;
use super::AnalogInput;
use super::Trigger;

#[derive(Debug)]
pub struct DataRequest {
    pub channels: [AnalogInput; 4],
    pub sample_rate_hz: f32,
    pub samples_to_record: u32,
    pub trigger: Trigger,
    pub sender: Sender<()>,

}


impl Nscope {
    pub fn request(&self) -> Receiver<()> {
        let (tx, rx) = mpsc::channel::<()>();

        let state = self.state.read().unwrap();

        let command = Command::RequestData(DataRequest {
            channels: state.analog_inputs,
            sample_rate_hz: 100000.0,
            samples_to_record: 800,
            trigger: state.trigger,
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

        if samples_between_records < 250 && self.samples_to_record * num_channels_on as u32 > 3200 {
            return Err("Data not recordable".into());
        }


        usb_buf[3..7].copy_from_slice(&samples_between_records.to_le_bytes());
        usb_buf[7..11].copy_from_slice(&self.samples_to_record.to_le_bytes());

        trace!("Requesting {} samples with {} samples between records", self.samples_to_record, samples_between_records);

        for (i, ch) in self.channels.iter().enumerate() {
            if ch.is_on {
                usb_buf[15+i] = ch.gain_setting;
                usb_buf[19+i] = ch.offset_setting;
            } else {
                usb_buf[15+i] = 0xFF;
            }
        }


        Ok(())
    }

    fn handle_rx(mut self, usb_buf: &[u8; 64], _scope_state: &Arc<RwLock<NscopeState>>) -> Option<Self> {

        self.samples_to_record -= usb_buf[3] as u32;
        trace!("Received {} samples, {} samples remaining", usb_buf[3], self.samples_to_record);

        if self.samples_to_record > 0 {
            return Some(self)
        }
        None
    }
}