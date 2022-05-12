/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use super::data_requests::DataRequest;

use std::error::Error;
use super::analog_output::AxRequest;
use super::pulse_output::PxRequest;
use super::NscopeState;
use log::debug;
use std::sync::{Arc, RwLock};


pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

pub(super) trait ScopeCommand: Sized {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>>;
    fn handle_rx(self, usb_buf: &[u8; 64], scope_state: &Arc<RwLock<NscopeState>>) -> Option<Self>;
}

// Check out initialization
// INITIALIZATION_WITHOUT_POWER = 0x06, -- tbd
// INITIALIZATION_WITH_POWER = 0x07, -- tbd

// Build scope interface
// STOP_REQUEST = 0x05, -- next up
// SCOPE_SWEEP_REQUEST = 0x08, -- next up

// Build out featureset
// PWM_DUTY_REQUEST = 0x00, -- not for 1.0
// FINITE_DATA_REQUEST = 0x03, -- not for 1.0
// RESET_TO_BOOTLOADER = 0x10 -- not for 1.0


#[derive(Debug)]
pub(super) enum Command {
    Quit,
    SetAnalogOutput(AxRequest),
    SetPulseOutput(PxRequest),
    RequestData(DataRequest),
}

impl Command {
    pub(super) fn fill_tx_buffer(&mut self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        debug!("Processed command: {:?}", self);
        match self {
            Command::Quit => { Ok(()) }
            Command::SetAnalogOutput(cmd) => { cmd.fill_tx_buffer(usb_buf) }
            Command::SetPulseOutput(cmd) => { cmd.fill_tx_buffer(usb_buf) }
            Command::RequestData(cmd) => { cmd.fill_tx_buffer(usb_buf) }
        }
    }

    pub(super) fn finish(self, buffer: &[u8; 64], scope_state: &Arc<RwLock<NscopeState>>) -> Option<Self> {
        match self {
            Command::Quit => { None }
            Command::SetAnalogOutput(cmd) => { cmd.handle_rx(buffer, scope_state).map(Command::SetAnalogOutput) }
            Command::SetPulseOutput(cmd) => { cmd.handle_rx(buffer, scope_state).map(Command::SetPulseOutput) }
            Command::RequestData(cmd) => { cmd.handle_rx(buffer, scope_state).map(Command::RequestData) }
        }
    }
}
