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
use std::str::FromStr;
use super::commands::Command;
use super::Nscope;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use crate::scope::commands::ScopeCommand;
use crate::scope::NscopeState;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AnalogWaveType {
    Sine = 0,
    Triangle = 1,
}

impl FromStr for AnalogWaveType {
    type Err = ();
    fn from_str(input: &str) -> Result<AnalogWaveType, Self::Err> {
        match input {
            "Sine" => Ok(AnalogWaveType::Sine),
            "Triangle" => Ok(AnalogWaveType::Triangle),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AnalogSignalPolarity {
    Unipolar,
    Bipolar,
}

impl FromStr for AnalogSignalPolarity {
    type Err = ();
    fn from_str(input: &str) -> Result<AnalogSignalPolarity, Self::Err> {
        match input {
            "Unipolar" => Ok(AnalogSignalPolarity::Unipolar),
            "Bipolar" => Ok(AnalogSignalPolarity::Bipolar),
            _ => Err(()),
        }
    }
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
    pub fn get_ax(&self, channel: usize) -> AnalogOutput {
        let state = self.state.read().unwrap();
        state.analog_output[channel]
    }

    pub(crate) fn set_ax(&self, channel: usize, ax: AnalogOutput) -> Receiver<AnalogOutput> {
        // Create a method for the backend to communicate back to us what we want
        let (tx, rx) = mpsc::channel::<AnalogOutput>();

        // Create the command to set an analog output
        let command = Command::SetAnalogOutput(AxRequest{
            channel,
            ax,
            sender: tx,
        });

        // Send the command to the backend
        self.command_tx.send(command).unwrap();
        rx
    }

    pub fn set_ax_on(&self, channel: usize, on: bool) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.get_ax(channel);
        requested_ax.is_on = on;

        let rx = self.set_ax(channel, requested_ax);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_ax_frequency_hz(&self, channel: usize, freq: f64) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.get_ax(channel);
        requested_ax.frequency = freq;

        let rx = self.set_ax(channel, requested_ax);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_ax_amplitude(&self, channel: usize, amplitude: f64) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.get_ax(channel);
        requested_ax.amplitude = amplitude;

        let rx = self.set_ax(channel, requested_ax);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_ax_wave_type(&self, channel: usize, wave_type: AnalogWaveType) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.get_ax(channel);
        requested_ax.wave_type = wave_type;

        let rx = self.set_ax(channel, requested_ax);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }

    pub fn set_ax_polarity(&self, channel: usize, polarity: AnalogSignalPolarity) -> AnalogOutput {
        // Get the current state of the analog output
        let mut requested_ax = self.get_ax(channel);
        requested_ax.polarity = polarity;

        let rx = self.set_ax(channel, requested_ax);

        // Wait for the backend to receive a response and return the result
        rx.recv().unwrap()
    }
}

#[derive(Debug)]
pub(super) struct AxRequest {
    channel: usize,
    ax: AnalogOutput,
    sender: Sender<AnalogOutput>,
}

impl ScopeCommand for AxRequest {
    fn fill_tx_buffer(&self, usb_buf: &mut [u8; 65]) -> Result<(), Box<dyn Error>> {
        usb_buf[1] = 0x02;

        let i_ch = 3 + 10 * self.channel;
        if self.ax.is_on {
            usb_buf[i_ch] = self.ax.wave_type as u8;
            usb_buf[i_ch] |= 0x80;

            let scaled_frequency = self.ax.frequency * 2.0_f64.powi(28) / 4000000.0;
            let freq_register: u32 = scaled_frequency as u32;

            usb_buf[i_ch + 1] = (freq_register & 0x00FF) as u8;
            usb_buf[i_ch + 2] = ((freq_register & 0x3F00) >> 8) as u8;
            usb_buf[i_ch + 3] = (freq_register >> 14 & 0x00FF) as u8;
            usb_buf[i_ch + 4] = ((freq_register >> 14 & 0x3F00) >> 8) as u8;

            if self.ax.amplitude < 0.0 {
                usb_buf[i_ch] |= 0x2;
            }
            let rf = 49900.0;
            let vin = 0.6;
            let rm = 75.0;
            let rv = 100000.0 / 257.0;

            let gain: u8 = match self.ax.polarity {
                AnalogSignalPolarity::Unipolar => ((vin * rf / self.ax.amplitude.abs() - rm) / rv) as u8,
                AnalogSignalPolarity::Bipolar => {
                    ((vin * rf / 2.0 / self.ax.amplitude.abs() - rm) / rv) as u8
                }
            };

            let offset: u8 = ((rm + rv * (gain as f64)) / (rm + rv * (gain as f64) + rf)
                * self.ax.amplitude.abs()
                * 255.0
                / 3.05) as u8;

            usb_buf[i_ch + 5] = gain;
            usb_buf[i_ch + 6] = offset;
        } else {
            usb_buf[i_ch] = 0xFF;
        }
        Ok(())
    }

    fn handle_rx(self, _usb_buf: &[u8; 64], scope_state: &Arc<RwLock<NscopeState>>) -> Option<Self> {
        let mut state = scope_state.write().unwrap();
        state.analog_output[self.channel] = self.ax;
        self.sender.send(self.ax).unwrap();
        None
    }
}