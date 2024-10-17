/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use std::{fmt, thread};
use std::convert::TryInto;
use std::error::Error;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

use hidapi::HidDevice;
use log::info;

use analog_input::AnalogInput;
use analog_output::AnalogOutput;
use commands::Command;
use power::PowerStatus;
use pulse_output::PulseOutput;
use trigger::Trigger;
use crate::lab_bench::NlabDevice;

mod commands;
pub mod analog_input;
pub mod analog_output;
pub mod pulse_output;
pub mod trigger;
pub mod power;
pub mod data_requests;
mod run_loops;

enum NlabHandle {
    NlabLegacy(HidDevice),
    Nlab(rusb::DeviceHandle<rusb::GlobalContext>),
}

/// Primary interface to the nLab, used to set outputs,
/// trigger sweeps of input data on scope channels, and monitor power state
pub struct Nlab {
    pub a1: AnalogOutput,
    pub a2: AnalogOutput,
    pub p1: PulseOutput,
    pub p2: PulseOutput,

    pub ch1: AnalogInput,
    pub ch2: AnalogInput,
    pub ch3: AnalogInput,
    pub ch4: AnalogInput,

    fw_version: Arc<RwLock<Option<u16>>>,
    power_status: Arc<RwLock<PowerStatus>>,
    command_tx: Sender<Command>,
    join_handle: Option<JoinHandle<()>>,
}

impl fmt::Debug for Nlab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nLab v1 [ connected: {} ]", self.is_connected())
    }
}

impl Nlab {
    /// Create a new Nlab object
    pub(crate) fn new(dev: &NlabDevice, power_on: bool) -> Result<Self, Box<dyn Error>> {
        let device_handle: NlabHandle = match dev {
            NlabDevice::HidApiDevice { device, api } => {
                let api = api.read().unwrap();
                NlabHandle::NlabLegacy(device.open_device(&api)?)
            }
            NlabDevice::RusbDevice(device) => {
                NlabHandle::Nlab(device.open()?)
            }
        };

        // Create communication channels to scope
        let (command_tx, command_rx) = mpsc::channel::<Command>();

        let fw_version = Arc::new(RwLock::new(None));
        let power_status = Arc::new(RwLock::new(PowerStatus::default()));

        let backend_command_tx = command_tx.clone();
        let backend_fw_version = fw_version.clone();
        let backend_power_status = power_status.clone();

        // Create the communication thread
        let communication_thread = thread::Builder::new().name("Communication Thread".to_string());

        let mut is_legacy = false;
        let join_handle = match device_handle {
            NlabHandle::NlabLegacy(hid_device) => {
                is_legacy = true;
                communication_thread.spawn(move || {
                    Nlab::run_v1(hid_device, backend_command_tx, command_rx, backend_fw_version, backend_power_status);
                }).ok()
            }
            NlabHandle::Nlab(usb_device) => {
                usb_device.claim_interface(0)?;
                communication_thread.spawn(move || {
                    Nlab::run_v2(usb_device, backend_command_tx, command_rx, backend_fw_version, backend_power_status);
                }).ok()
            }
        };

        let scope = Nlab {
            a1: AnalogOutput::create(command_tx.clone(), 0),
            a2: AnalogOutput::create(command_tx.clone(), 1),
            p1: PulseOutput::create(command_tx.clone(), 0),
            p2: PulseOutput::create(command_tx.clone(), 1),
            ch1: AnalogInput::create(is_legacy),
            ch2: AnalogInput::create(is_legacy),
            ch3: AnalogInput::create(is_legacy),
            ch4: AnalogInput::create(is_legacy),
            fw_version,
            power_status,
            command_tx,
            join_handle,
        };

        // Send the initialization command
        let (init_tx, init_rx) = mpsc::channel::<()>();
        if scope.command_tx.send(Command::Initialize(power_on, init_tx)).is_ok() {
            if is_legacy {
                info!("Connected to nLab legacy firmware v{}", scope.version().unwrap() as f64 / 10.0);
                return Ok(scope);
            }
            // Wait for the response from the backend
            if init_rx.recv_timeout(Duration::from_secs(5)).is_ok() {
                info!("Connected to nLab firmware 0x{:04X}", scope.version().unwrap());
                return Ok(scope);
            }
        }
        Err("Cannot initialize scope".into())
    }

    pub fn is_connected(&self) -> bool {
        match &self.join_handle {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    pub fn close(&mut self) {
        let _ = self.command_tx.send(Command::Quit);
        // Wait for the loop to end
        if self.join_handle.is_some() {
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    #[deprecated(since = "1.1.0", note = "Please use `version` instead")]
    pub fn fw_version(&self) -> Result<u8, Box<dyn Error>> {
        if let Some(full_version) = *self.fw_version.read().unwrap() {
            if (full_version & 0xFF00) != 0 {
                return Err("Connected to nLab v2 or newer, use scope.version() to read".into());
            }
            return Ok(full_version as u8);
        }
        Err("Cannot read nLab version".into())
    }

    pub fn version(&self) -> Result<u16, Box<dyn Error>> {
        self.fw_version.read().unwrap().ok_or_else(|| "Cannot read nLab version".into())
    }

    pub fn analog_output(&self, channel: usize) -> Option<&AnalogOutput> {
        match channel {
            1 => Some(&self.a1),
            2 => Some(&self.a2),
            _ => None,
        }
    }

    pub fn pulse_output(&self, channel: usize) -> Option<&PulseOutput> {
        match channel {
            1 => Some(&self.p1),
            2 => Some(&self.p2),
            _ => None,
        }
    }

    pub fn channel(&self, channel: usize) -> Option<&AnalogInput> {
        match channel {
            1 => Some(&self.ch1),
            2 => Some(&self.ch2),
            3 => Some(&self.ch3),
            4 => Some(&self.ch4),
            _ => None,
        }
    }
}

/// When an Nlab goes out of scope, we need to exit the IO loop
impl Drop for Nlab {
    fn drop(&mut self) {
        // Send a quit command to the IO loop
        let _ = self.command_tx.send(Command::Quit);

        // Wait for the loop to end
        if self.join_handle.is_some() {
            self.join_handle.take().unwrap().join().unwrap()
        }
    }
}

#[derive(Debug)]
struct StatusResponseLegacy {
    fw_version: u8,
    power_state: power::PowerState,
    power_usage: u8,
    request_id: u8,
}

impl StatusResponseLegacy {
    pub(crate) fn new(buf: &[u8]) -> Self {
        StatusResponseLegacy {
            fw_version: buf[0] & 0x3F,
            power_state: power::PowerState::from((buf[0] & 0xC0) >> 6),
            power_usage: buf[1],
            request_id: buf[2],
        }
    }
}

#[derive(Debug)]
struct StatusResponse {
    fw_version: u16,
    power_state: power::PowerState,
    power_usage: f32,
    request_id: u8,
}

impl StatusResponse {
    pub(crate) fn new(buf: &[u8]) -> Self {
        StatusResponse {
            request_id: buf[0],
            fw_version: u16::from_le_bytes(buf[1..3].try_into().unwrap()),
            power_state: power::PowerState::from(buf[3]),
            power_usage: f32::from_le_bytes(buf[4..8].try_into().unwrap()),
        }
    }
}