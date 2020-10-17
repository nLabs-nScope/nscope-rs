/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;
use std::{fmt, thread};

pub mod analog_output;
mod commands;
pub mod power;

use crate::scope::commands::generate_packet;
use analog_output::AnalogOutput;
use commands::Command;
use power::PowerStatus;

/// Object for accessing an nScope
pub struct Nscope {
    pub vid: u16,
    pub pid: u16,
    power_status: Arc<RwLock<PowerStatus>>,
    analog_output: [AnalogOutput; 2],
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

            let power_status = Arc::new(RwLock::new(PowerStatus::default()));
            let power_status_clone = Arc::clone(&power_status);

            let analog_output = [AnalogOutput::default(); 2];

            Some(Nscope {
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                power_status,
                analog_output,
                command_tx,
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
        let mut incoming_usb_buffer = [0u8; 64];
        let mut request_id: u8 = 0;

        loop {
            // check for an incoming command
            {
                if let Ok(command) = command_rx.try_recv() {
                    match command {
                        Command::Quit => break,
                        _ => {
                            request_id += 1;
                            if request_id == 0 {
                                request_id += 1
                            }
                            hid_device
                                .write(&generate_packet(request_id, command))
                                .unwrap()
                        }
                    };
                } else if hid_device.write(&commands::NULL_REQ).is_err() {
                    break;
                }
            }

            if hid_device.read(&mut incoming_usb_buffer[..]).is_err() {
                break;
            }

            let response = StatusResponse::new(&incoming_usb_buffer);

            // update the power status
            {
                let mut writer = power_status.write().unwrap();
                writer.state = response.power_state;
                writer.usage = response.power_usage as f64 * 5.0 / 255.0;
            }
        }
    }

    // Todo: come up with a better way of determining this
    pub fn is_connected(&self) -> bool {
        Arc::strong_count(&self.power_status) > 1
    }
}

/// When an Nscope goes out of scope, we need to exit the IO loop
impl Drop for Nscope {
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
struct StatusResponse {
    fw_version: u8,
    power_state: power::PowerState,
    power_usage: u8,
    request_id: u8,
}

impl StatusResponse {
    pub(crate) fn new(buf: &[u8]) -> Self {
        StatusResponse {
            fw_version: buf[0] & 0x3F,
            power_state: power::PowerState::from((buf[0] & 0xC0) >> 6),
            power_usage: buf[1],
            request_id: buf[2],
        }
    }
}
