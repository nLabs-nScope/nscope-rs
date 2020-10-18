/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use enclose::enclose;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;
use std::{fmt, thread};

pub mod analog_output;
mod commands;
pub mod power;

use analog_output::AnalogOutput;
use commands::Command;
use power::PowerStatus;

/// Object for accessing an nScope
pub struct Nscope {
    pub vid: u16,
    pub pid: u16,
    power_status: Arc<RwLock<PowerStatus>>,
    analog_output: Arc<RwLock<[AnalogOutput; 2]>>,
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

            let analog_output = Arc::new(RwLock::new([AnalogOutput::default(); 2]));

            let join_handle = Some(thread::spawn(
                enclose!((power_status, analog_output) move || {
                    Nscope::run(hid_device, command_rx, power_status, analog_output)
                }),
            ));

            Some(Nscope {
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                power_status,
                analog_output,
                command_tx,
                join_handle,
            })
        } else {
            None
        }
    }

    fn run(
        hid_device: HidDevice,
        command_rx: Receiver<Command>,
        power_status: Arc<RwLock<PowerStatus>>,
        _analog_output: Arc<RwLock<[AnalogOutput; 2]>>,
    ) {
        let mut active_requests: Vec<(u8, Command)> = Vec::new();
        let mut incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut outgoing_usb_buffer: [u8; 65] = [0u8; 65];
        let mut request_id: u8 = 0;

        loop {
            // check for an incoming command
            // Do one of the following:
            // 1. Write a request
            // 2. Write a null packet to request an update on the power status

            if let Ok(mut command) = command_rx.try_recv() {
                if let Command::Quit = command {
                    break;
                }

                // Process the command
                // 1. fill the outgoing USB buffer
                // 2. increment the request id
                // 3. send the
                // 3. store whatever we want to send back

                command.process(&mut outgoing_usb_buffer);
                {
                    //TODO: make this block more concise
                    request_id += 1;
                    if request_id == 0 {
                        request_id += 1
                    }
                    outgoing_usb_buffer[2] = request_id;
                }
                hid_device.write(&outgoing_usb_buffer).unwrap();
                active_requests.push((request_id, command));
                println!("Sent request {}", request_id);
            } else {
                hid_device.write(&commands::NULL_REQ).unwrap();
            }

            // Read the incoming command and process it
            hid_device.read(&mut incoming_usb_buffer).unwrap();

            let response = StatusResponse::new(&incoming_usb_buffer);

            // update the power status
            {
                let mut writer = power_status.write().unwrap();
                writer.state = response.power_state;
                writer.usage = response.power_usage as f64 * 5.0 / 255.0;
            }

            // close out the request
            if response.request_id > 0 {
                if let Some(queue_index) = active_requests
                    .iter()
                    .position(|(id, _)| id == &response.request_id)
                {
                    println!("Finished request ID: {}", response.request_id);
                    let (_, command) = active_requests.remove(queue_index);
                    command.finish();
                } else {
                    eprintln!("Received response for request {}, but cannot find a record of that request", response.request_id);
                }
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
