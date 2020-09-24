/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

mod commands;
mod power;

use commands::Command;

use crate::nscope::power::PowerStatus;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;
use std::{fmt, thread};

pub struct Nscope {
    pub vid: u16,
    pub pid: u16,
    power_status: Arc<RwLock<PowerStatus>>,

    command_tx: Sender<Command>,
    join_handle: Option<JoinHandle<()>>,
}

impl fmt::Debug for Nscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VID: 0x{:04X}, PID: 0x{:04X}", self.vid, self.pid,)
    }
}

/// Create a new Nscope object
impl Nscope {
    pub(crate) fn new(dev: &DeviceInfo, hid_api: &HidApi) -> Option<Self> {
        // Open the hid_device
        if let Ok(hid_device) = dev.open_device(hid_api) {
            // If we're able to open it
            let (command_tx, command_rx) = mpsc::channel::<Command>();

            let power_status = Arc::new(RwLock::new(PowerStatus::new()));
            let power_status_clone = Arc::clone(&power_status);
            // let data = Arc::new(RwLock::new(NscopeData::new()));
            // let data_clone = Arc::clone(&data);

            Some(Nscope {
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                command_tx,
                power_status,
                join_handle: Some(thread::spawn(move || {
                    Nscope::run(hid_device, command_rx, power_status_clone)
                })),
            })
        } else {
            None
        }
    }

    fn run(
        hid_device: HidDevice,
        command_rx: Receiver<Command>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut buf = [0u8; 64];

        loop {
            // check for an incoming command
            {
                if let Ok(command) = command_rx.try_recv() {
                    match command {
                        Command::Quit => break,
                    }
                } else {
                    hid_device.write(&commands::NULL_REQ).unwrap();
                }
            }

            hid_device
                .read(&mut buf[..])
                .expect("Cannot Read from device");

            let response = StatusResponse::new(&buf);

            // update the power status
            {
                let mut writer = power_status.write().unwrap();
                writer.state = response.power_state;
                writer.usage = response.power_usage as f32 * 5.0 / 255.0;
            }
        }
    }

    pub fn power_usage(&self) -> f32 {
        self.power_status.read().unwrap().usage
    }

    pub fn power_state(&self) -> power::State {
        self.power_status.read().unwrap().state
    }
}

/// When an Nscope goes out of scope, we need to exit the IO loop
impl Drop for Nscope {
    fn drop(&mut self) {
        // Send a quit command to the IO loop
        self.command_tx.send(Command::Quit).unwrap();

        // Wait for the loop to end
        if self.join_handle.is_some() {
            self.join_handle.take().unwrap().join().unwrap()
        }
    }
}

#[derive(Debug)]
struct StatusResponse {
    fw_version: u8,
    power_state: power::State,
    power_usage: u8,
    request_id: u8,
}

impl StatusResponse {
    pub(crate) fn new(buf: &[u8]) -> Self {
        StatusResponse {
            fw_version: buf[0] & 0x3F,
            power_state: power::State::from((buf[0] & 0xC0) >> 6),
            power_usage: buf[1],
            request_id: buf[2],
        }
    }
}
