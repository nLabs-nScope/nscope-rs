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
use log::trace;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;
use std::{fmt, thread};

mod commands;
pub mod analog_output;
pub mod pulse_output;
pub mod power;

use analog_output::AnalogOutput;
use commands::Command;
use power::PowerStatus;
use crate::scope::pulse_output::PulseOutput;

struct NscopeState {
    fw_version: Option<u8>,
    power_status: PowerStatus,
    analog_output: [AnalogOutput; 2],
    pulse_output: [PulseOutput; 2],
}

/// Object for accessing an nScope
pub struct Nscope {
    pub vid: u16,
    pub pid: u16,
    state: Arc<RwLock<NscopeState>>,
    command_tx: Sender<Command>,
    join_handle: Option<JoinHandle<()>>,
}

impl fmt::Debug for Nscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VID: 0x{:04X}, PID: 0x{:04X}", self.vid, self.pid, )
    }
}

impl Nscope {
    /// Create a new Nscope object
    pub(crate) fn new(dev: &DeviceInfo, hid_api: &HidApi) -> Result<Self, Box<dyn Error>> {
        // Open the hid_device
        let hid_device = dev.open_device(hid_api)?;

        // Create communication channels to scope
        let (command_tx, command_rx) = mpsc::channel::<Command>();

        let scope_state = Arc::new(RwLock::new(NscopeState {
            fw_version: None,
            power_status: PowerStatus::default(),
            analog_output: [AnalogOutput::default(); 2],
            pulse_output: [PulseOutput::default(); 2],
        }));

        let join_handle = Some(thread::spawn(enclose!((scope_state) move || {
            Nscope::run(hid_device, command_rx, scope_state)
        })));

        Ok(Nscope {
            vid: dev.vendor_id(),
            pid: dev.product_id(),
            state: scope_state,
            command_tx,
            join_handle,
        })
    }

    fn run(
        hid_device: HidDevice,
        command_rx: Receiver<Command>,
        scope_state: Arc<RwLock<NscopeState>>,
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

                let result = command.fill_tx_buffer(&mut outgoing_usb_buffer);
                if result.is_err() {
                    eprintln!("{:?}", result);
                }
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
                trace!("Sent request {}", request_id);
            } else {
                hid_device.write(&commands::NULL_REQ).unwrap();
            }
            // DO NOT USE UNWRAP
            // Read the incoming command and process it
            hid_device.read(&mut incoming_usb_buffer).unwrap();

            let response = StatusResponse::new(&incoming_usb_buffer);

            // update the scope power status
            {
                let mut state = scope_state.write().unwrap();
                state.fw_version = Some(response.fw_version);
                state.power_status.state = response.power_state;
                state.power_status.usage = response.power_usage as f64 * 5.0 / 255.0;
            }

            // close out the request
            if response.request_id > 0 {
                if let Some(queue_index) = active_requests
                    .iter()
                    .position(|(id, _)| id == &response.request_id)
                {
                    trace!("Finished request ID: {}", response.request_id);
                    let (_, command) = active_requests.remove(queue_index);
                    command.finish(&scope_state);
                } else {
                    eprintln!("Received response for request {}, but cannot find a record of that request", response.request_id);
                }
            }
        }
    }

    // Todo: come up with a better way of determining this
    pub fn is_connected(&self) -> bool {
        Arc::strong_count(&self.state) > 1
    }

    pub fn fw_version(&self) -> Result<u8, Box<dyn Error>> {
        let state = &self.state.read().unwrap();
        state.fw_version.ok_or_else(|| "Cannot read FW version".into())
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
