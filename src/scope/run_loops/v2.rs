use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use log::{error, trace};
use rusb::DeviceHandle;
use crate::PowerStatus;
use crate::scope::commands::Command;

impl crate::Nscope {
    pub(crate) fn run_v2(
        usb_device: DeviceHandle<rusb::GlobalContext>,
        command_tx: Sender<Command>,
        command_rx: Receiver<Command>,
        fw_version: Arc<RwLock<Option<u8>>>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut active_requests_map: HashMap<u8, Command> = HashMap::new();
        let incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut request_id: u8 = 0;

        'communication: loop {
            if let Ok(command) = command_rx.try_recv() {
                if let Command::Quit = &command {
                    break 'communication;
                }

                request_id = request_id.wrapping_add(1);
                if request_id == 0 {
                    request_id += 1
                }

                active_requests_map.insert(request_id, command);
                trace!("Sent request {}", request_id);
            }

            // If we have an active request with this ID
            if let Some(command) = active_requests_map.get(&request_id)
            {
                // Handle the incoming usb packet
                command.handle_rx(&incoming_usb_buffer);

                // If the command has finished it's work
                if command.is_finished() {

                    active_requests_map.remove(&request_id);

                    // Set the active data request as none if we just finished it
                    // active_data_request = active_data_request.filter(|&id| id != response.request_id);

                    trace!("Finished request ID: {}", request_id);
                } else {
                    trace!("Received request ID: {}", request_id);
                }
            } else {
                error!("Received response for request {}, but cannot find a record of that request", request_id);
            }
        }
    }
}