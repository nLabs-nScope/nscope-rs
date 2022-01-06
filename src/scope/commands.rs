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
use super::analog_output::{update_analog_output, AnalogOutput};
use super::NscopeState;
use log::debug;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use crate::scope::pulse_output::{PulseOutput, update_pulse_output};

pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

#[derive(Debug)]
pub enum Command {
    Quit,
    SetAnalogOutput {
        channel: usize,
        ax: AnalogOutput,
        sender: Sender<AnalogOutput>,
    },
    SetPulseOutput {
        channel: usize,
        px: PulseOutput,
        sender: Sender<PulseOutput>,
    },
}

impl Command {
    pub fn fill_tx_buffer(&mut self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        debug!("Processed command: {:?}", self);
        match self {
            Command::Quit => {Ok(())}
            Command::SetAnalogOutput { channel, ax, .. } => {
                update_analog_output(usb_buf, channel, ax)
            }
            Command::SetPulseOutput { channel, px, .. } => {
                update_pulse_output(usb_buf, channel, px)
            }
        }
    }

    pub(super) fn finish(self, scope_state: &Arc<RwLock<NscopeState>>) {
        match self {
            Command::Quit => {}
            Command::SetAnalogOutput { channel, ax, sender, } => {
                let mut state = scope_state.write().unwrap();
                state.analog_output[channel] = ax;
                sender.send(ax).unwrap();
            }
            Command::SetPulseOutput { channel, px, sender } => {
                let mut state = scope_state.write().unwrap();
                state.pulse_output[channel] = px;
                sender.send(px).unwrap();
            }
        };
    }
}
