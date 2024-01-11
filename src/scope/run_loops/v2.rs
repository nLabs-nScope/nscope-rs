use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use log::{error, trace};
use rusb::DeviceHandle;
use crate::PowerStatus;
use crate::scope::commands::{Command, ScopeCommand};
use crate::scope::StatusResponse;

impl crate::Nscope {
    pub(crate) fn run_v2(
        usb_device: DeviceHandle<rusb::GlobalContext>,
        command_tx: Sender<Command>,
        command_rx: Receiver<Command>,
        fw_version: Arc<RwLock<Option<u8>>>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut active_requests_map: HashMap<u8, Command> = HashMap::new();
        let mut active_data_request: Option<Command> = None;
        let mut incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut outgoing_usb_buffer: [u8; 64] = [0u8; 64];
        let mut _incoming_channel_buffers: [[u8; 64]; 4] = [[0u8; 64]; 4];
        let mut request_id: u8 = 0;

        'communication: loop {
            // Check first to see if we have a cancelled active request
            if let Some(Command::RequestData(rq)) = &active_data_request {
                // Get the active request if we have one, check to see if we have a received a stop
                if let Ok(()) = rq.stop_recv.try_recv() {
                    command_tx.send(Command::StopData).unwrap();
                }
            }

            if let Ok(command) = command_rx.try_recv() {
                // If we have a command from the front-end, assign a new requestID
                // and send the request out to the usb control line

                outgoing_usb_buffer.fill(0);
                request_id = request_id.wrapping_add(1);
                if request_id == 0 {
                    request_id += 1
                }

                outgoing_usb_buffer[0] = request_id;
                outgoing_usb_buffer[1] = command.id_byte();
                trace!("Sent request {}: command: {}", request_id, command.id_byte());

                // Fill the outgoing buffer with whatever we need
                match &command {
                    Command::Quit => { break 'communication; }
                    Command::Initialize(power_on) => {
                        outgoing_usb_buffer[2] = *power_on as u8;
                        active_requests_map.insert(request_id, command);
                    }
                    Command::SetAnalogOutput(cmd) => {
                        cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                            .expect("Invalid parameters given to AxRequest");
                        active_requests_map.insert(request_id, command);
                    }
                    Command::SetPulseOutput(cmd) => {
                        cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                            .expect("Invalid parameters given to PxRequest");
                        active_requests_map.insert(request_id, command);
                    }
                    Command::RequestData(cmd) => {
                        cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                            .expect("Invalid parameters given to DataRequest");
                        active_data_request = Some(command);
                    }
                    Command::StopData => {
                        active_requests_map.insert(request_id, command);
                    }
                };

                if let Err(error) = usb_device.write_bulk(0x01,
                                                          &outgoing_usb_buffer,
                                                          Duration::from_millis(100))
                {
                    error!("USB write error: {:?}", error);
                    break 'communication;
                }
            }

            match usb_device.read_bulk(0x81,
                                       &mut incoming_usb_buffer,
                                       Duration::from_millis(1))
            {
                Err(rusb::Error::Timeout) => {}
                Ok(_) => {
                    let response = StatusResponse::new(&incoming_usb_buffer);

                    *fw_version.write().unwrap() = Some((response.fw_version & 0xFF) as u8);
                    power_status.write().unwrap().state = response.power_state;
                    power_status.write().unwrap().usage = response.power_usage as f64 / 1000.0 * 5.0;

                    if response.request_id == 0 {
                        trace!("Received a status update from nScope");
                    } else if let Some(command) = active_requests_map.get(&response.request_id) {
                        // If we have an active request with this ID

                        // Handle the incoming usb packet
                        command.handle_rx(&incoming_usb_buffer);

                        // If the command has finished it's work
                        if command.is_finished() {
                            active_requests_map.remove(&request_id);


                            trace!("Finished request ID: {}", request_id);
                        } else {
                            trace!("Received request ID: {}", request_id);
                        }
                    } else {
                        error!("Received response for request {}, but cannot find a record of that request", request_id);
                    }
                }
                Err(error) => {
                    error!("USB read error: {:?}", error);
                    break 'communication;
                }
            }

            // TODO: Read the four input channels and put them into a sample
        }
    }
}