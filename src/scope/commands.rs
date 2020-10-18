/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use super::analog_output::{update_analog_output, AnalogOutput};
use std::sync::mpsc::Sender;

pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

#[derive(Debug)]
pub enum Command {
    Quit,
    SetAnalogOutput {
        channel: usize,
        ax: AnalogOutput,
        actual: Option<AnalogOutput>,
        sender: Sender<AnalogOutput>,
    },
}

impl Command {
    pub fn process(&mut self, usb_buf: &mut [u8; 65]) {
        println!("Processed command: {:?}", self);
        match self {
            Command::Quit => {}
            Command::SetAnalogOutput {
                channel,
                ax,
                actual,
                sender: _,
            } => {
                *actual = update_analog_output(usb_buf, *channel, *ax);
            }
        };
    }

    pub fn finish(&self) {
        match self {
            Command::Quit => {}
            Command::SetAnalogOutput {
                channel,
                ax: _,
                actual,
                sender,
            } => {
                sender.send(actual.unwrap()).unwrap();
            }
        };
    }
}
