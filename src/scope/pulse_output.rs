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
use std::sync::{mpsc, RwLock};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::scope::commands::{Command, ScopeCommand};

#[derive(Debug, Copy, Clone)]
enum PulsePreScale {
    One,
    Eight,
    SixtyFour,
    TwoFiftySix,
}

impl PulsePreScale {
    fn value(&self) -> u64 {
        match *self {
            PulsePreScale::One => 1,
            PulsePreScale::Eight => 8,
            PulsePreScale::SixtyFour => 64,
            PulsePreScale::TwoFiftySix => 256,
        }
    }
    fn register(&self) -> u8 {
        match *self {
            PulsePreScale::One => 0,
            PulsePreScale::Eight => 1,
            PulsePreScale::SixtyFour => 2,
            PulsePreScale::TwoFiftySix => 3,
        }
    }
}


/// Interface to a pulse output
#[derive(Debug, Copy, Clone)]
struct PulseOutputState {
    pub is_on: bool,
    pub frequency: f64,
    pub duty: f64,
}

impl PulseOutputState {
    pub fn period(&self) -> Duration {
        let period = 1.0 / self.frequency;
        Duration::from_secs_f64(period)
    }
    pub fn pulse_width(&self) -> Duration {
        let period = self.period();
        period.mul_f64(self.duty)
    }
}

#[derive(Debug)]
pub struct PulseOutput {
    pub channel: usize,
    command_tx: Sender<Command>,
    state: RwLock<PulseOutputState>,
}


impl PulseOutput {
    pub(super) fn create(cmd_tx: Sender<Command>, px_channel: usize) -> Self {
        PulseOutput {
            command_tx: cmd_tx,
            channel: px_channel,
            state: RwLock::new(PulseOutputState {
                is_on: false,
                frequency: 1.0,
                duty: 0.5,
            }),
        }
    }

    fn set(&self, px_state: PulseOutputState) {
        // Create a method for the backend to communicate back to us what we want
        let (tx, rx) = mpsc::channel::<PulseOutputState>();

        // Create the command to set an analog output
        let command = Command::SetPulseOutput(PxRequest {
            channel: self.channel,
            px_state,
            sender: tx,

        });

        // Send the command to the backend
        self.command_tx.send(command).unwrap();

        // Wait for the response from the backend
        let response_state = rx.recv().unwrap();
        // Write the response state
        *self.state.write().unwrap() = response_state;
    }

    pub fn is_on(&self) -> bool {
        self.state.read().unwrap().is_on
    }
    pub fn frequency(&self) -> f64 {
        self.state.read().unwrap().frequency
    }
    pub fn duty(&self) -> f64 {
        self.state.read().unwrap().duty
    }
    pub fn period(&self) -> Duration {
        self.state.read().unwrap().period()
    }
    pub fn pulse_width(&self) -> Duration {
        self.state.read().unwrap().pulse_width()
    }

    pub fn turn_on(&self) {
        let mut state = *self.state.read().unwrap();
        state.is_on = true;
        self.set(state)
    }
    pub fn turn_off(&self) {
        let mut state = *self.state.read().unwrap();
        state.is_on = false;
        self.set(state)
    }

    pub fn set_frequency(&self, desired_hz: f64) {
        let mut state = *self.state.read().unwrap();
        state.frequency = desired_hz;
        self.set(state)
    }

    pub fn set_duty(&self, desired_percentage: f64) {
        let mut state = *self.state.read().unwrap();
        state.duty = desired_percentage;
        self.set(state)
    }
}

fn get_registers(pulse_output: &PulseOutputState) -> Result<(u8, u32, u32), Box<dyn Error>> {

    // The period and duty registers are an integeter number of 16 MHz clock cycles
    let period = (pulse_output.period().as_nanos() * 16 / 1000) as u64;
    let duty = (pulse_output.pulse_width().as_nanos() * 16 / 1000) as u64;

    let prescale = if period < 4u64 {
        return Err("Desired pulse length is too short".into());
    } else if period <= u16::MAX as u64 {
        PulsePreScale::One
    } else if period <= u16::MAX as u64 * PulsePreScale::Eight.value() {
        PulsePreScale::Eight
    } else if period <= u16::MAX as u64 * PulsePreScale::SixtyFour.value() {
        PulsePreScale::SixtyFour
    } else if period <= u16::MAX as u64 * PulsePreScale::TwoFiftySix.value() {
        PulsePreScale::TwoFiftySix
    } else {
        return Err("Desired pulse length is too long".into());
    };

    let period_register = (period / (prescale.value())) as u32;
    let duty_register = (duty / (prescale.value())) as u32;

    Ok((prescale.register(), period_register, duty_register))
}

#[derive(Debug)]
pub(super) struct PxRequest {
    channel: usize,
    px_state: PulseOutputState,
    sender: Sender<PulseOutputState>,
}

impl ScopeCommand for PxRequest {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x01;

        let i_ch = 3 + 10 * self.channel;
        let (prescale, period, duty) = get_registers(&self.px_state)?;

        if self.px_state.is_on {
            usb_buf[i_ch] = 0x80 | prescale;
            usb_buf[i_ch + 1..=i_ch + 4].copy_from_slice(&period.to_le_bytes());
            usb_buf[i_ch + 5..=i_ch + 8].copy_from_slice(&duty.to_le_bytes());
        } else {
            usb_buf[i_ch] = 0xFF;
        }

        Ok(())
    }

    fn handle_rx(&self, _usb_buf: &[u8; 64]) {
        self.sender.send(self.px_state).unwrap();
    }

    fn is_finished(&self) -> bool {
        true
    }
}