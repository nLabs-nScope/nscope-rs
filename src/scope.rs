/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::{fmt, thread};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::{error, trace};

use analog_input::AnalogInput;
use analog_output::AnalogOutput;
use commands::Command;
use power::PowerStatus;
use pulse_output::PulseOutput;
use trigger::Trigger;
use crate::scope::data_requests::StopRequest;

mod commands;
pub mod analog_input;
pub mod analog_output;
pub mod pulse_output;
pub mod trigger;
pub mod power;
pub mod data_requests;

/// Object for accessing an nScope
pub struct Nscope {
    pub a1: AnalogOutput,
    pub a2: AnalogOutput,
    pub p1: PulseOutput,
    pub p2: PulseOutput,

    pub ch1: AnalogInput,
    pub ch2: AnalogInput,
    pub ch3: AnalogInput,
    pub ch4: AnalogInput,

    pub trigger: Trigger,

    vid: u16,
    pid: u16,

    fw_version: Arc<RwLock<Option<u8>>>,
    power_status: Arc<RwLock<PowerStatus>>,
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

        let fw_version = Arc::new(RwLock::new(None));
        let power_status = Arc::new(RwLock::new(PowerStatus::default()));

        let backend_command_tx = command_tx.clone();
        let backend_fw_version = fw_version.clone();
        let backend_power_status = power_status.clone();

        // Create the communication thread
        let communication_thread = thread::Builder::new().name("Communication Thread".to_string());
        let join_handle = communication_thread.spawn(move || {
            Nscope::run(hid_device, backend_command_tx, command_rx, backend_fw_version, backend_power_status);
        }).ok();

        let scope = Nscope {
            a1: AnalogOutput::create(command_tx.clone(), 0),
            a2: AnalogOutput::create(command_tx.clone(), 1),
            p1: PulseOutput::create(command_tx.clone(), 0),
            p2: PulseOutput::create(command_tx.clone(), 1),
            ch1: AnalogInput::default(),
            ch2: AnalogInput::default(),
            ch3: AnalogInput::default(),
            ch4: AnalogInput::default(),
            trigger: Trigger::default(),
            vid: dev.vendor_id(),
            pid: dev.product_id(),
            fw_version,
            power_status,
            command_tx,
            join_handle,
        };

        Ok(scope)
    }

    fn run(
        hid_device: HidDevice,
        command_tx: Sender<Command>,
        command_rx: Receiver<Command>,
        fw_version: Arc<RwLock<Option<u8>>>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut active_requests_map: HashMap<u8, Command> = HashMap::new();
        let mut active_data_request: Option<u8> = None;
        let mut incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut outgoing_usb_buffer: [u8; 65] = [0u8; 65];
        let mut request_id: u8 = 0;

        'communication: loop {
            // Check first to see if we have a cancelled active request
            if let Some(id) = &active_data_request {
                // We have an active request id
                if let Command::RequestData(rq) = active_requests_map.get(id).unwrap() {
                    // we get the active request
                    if let Ok(()) = rq.stop_recv.try_recv() {
                        // We have recieved a stop signal
                        command_tx.send(Command::StopData(StopRequest{})).unwrap();
                    }
                }
            }


            // check for an incoming command from the user
            // Do one of the following:
            // 1. Write a request to do the command
            // 2. Write a null packet to request an update on the power status

            if let Ok(mut command) = command_rx.try_recv() {
                if let Command::Quit = &command {
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
                    request_id = request_id.wrapping_add(1);
                    if request_id == 0 {
                        request_id += 1
                    }
                    outgoing_usb_buffer[2] = request_id;
                }
                if hid_device.write(&outgoing_usb_buffer).is_err() {
                    eprintln!("USB write error, ending nScope connection");
                    break 'communication;
                }

                if let Command::RequestData(_) = &command {
                    active_data_request = Some(request_id);
                }
                active_requests_map.insert(request_id, command);
                trace!("Sent request {}", request_id);

            } else if hid_device.write(&commands::NULL_REQ).is_err() {
                eprintln!("USB write error, ending nScope connection");
                break 'communication;
            }

            // Read the incoming command and process it
            if hid_device.read(&mut incoming_usb_buffer).is_err() {
                eprintln!("USB read error, ending nScope connection");
                break 'communication;
            }

            let response = StatusResponse::new(&incoming_usb_buffer);

            *fw_version.write().unwrap() = Some(response.fw_version);
            power_status.write().unwrap().state = response.power_state;
            power_status.write().unwrap().usage = response.power_usage as f64 * 5.0 / 255.0;

            // close out request if it's open
            if response.request_id > 0 {

                // If we have an active request with this ID
                if let Some(command) = active_requests_map.get(&response.request_id)
                {
                    // Handle the incoming usb packet
                    command.handle_rx(&incoming_usb_buffer);

                    // If the command has finished it's work
                    if command.is_finished() {

                        // Set the active data request as none if we just finished it
                        active_data_request = active_data_request.filter(|&id| id != response.request_id);

                        // Remove this request from the active map
                        if let Some(Command::StopData(_)) = active_requests_map.remove(&response.request_id) {
                            // If we received the ACK on a stop command, check if we have an active id
                            if let Some(active_id) = &active_data_request {
                                // Look up that ID, remove the command from the active map
                                if let Some(Command::RequestData(rq)) = active_requests_map.remove(active_id) {
                                    // If that command is a request data command
                                    *rq.remaining_samples.write().unwrap() = 0;
                                }
                                active_data_request = None;
                            }
                        }

                        trace!("Finished request ID: {}, ADRQ: {:?}", response.request_id, active_data_request);
                    } else {
                        trace!("Received request ID: {}, ADRQ: {:?}", response.request_id, active_data_request);
                    }
                } else {
                    error!("Received response for request {}, but cannot find a record of that request", response.request_id);
                }
            }
        }
    }

    // Todo: come up with a better way of determining this
    pub fn is_connected(&self) -> bool {
        Arc::strong_count(&self.fw_version) > 1
    }


    pub fn fw_version(&self) -> Result<u8, Box<dyn Error>> {
        self.fw_version.read().unwrap().ok_or_else(|| "Cannot read FW version".into())
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
