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
mod responses;

use hidapi::{HidDevice, HidApi, DeviceInfo};
use std::{fmt, thread};
use std::thread::JoinHandle;
use std::sync::{mpsc, RwLock, Arc};
use std::sync::mpsc::{Receiver, Sender};
use commands::Command;


pub struct NscopeData {
    pub power_state: responses::PowerState,
    pub power_usage: f32,
}

impl NscopeData {
    fn new() -> NscopeData {
        NscopeData {
            power_state: responses::PowerState::Unknown,
            power_usage: 0.0,
        }
    }
}

pub struct Nscope {
    vid: u16,
    pid: u16,
    pub data: Arc<RwLock<NscopeData>>,
    command_tx: Sender<Command>,
    join_handle: Option<JoinHandle<()>>,
}

impl fmt::Debug for Nscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VID: 0x{:04X}, PID: 0x{:04X}",
            self.vid,
            self.pid,
        )
    }
}

/// Create a new Nscope object
impl Nscope {
    pub(crate) fn new(dev: &DeviceInfo, hid_api: &HidApi) -> Option<Nscope> {

        // Open the hid_device
        if let Ok(hid_device) = dev.open_device(hid_api) {

            // If we're able to open it
            let (command_tx, command_rx) = mpsc::channel();

            let data = Arc::new(RwLock::new(NscopeData::new()));
            let data2 = Arc::clone(&data);

            Some(Nscope {
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                data,
                command_tx,
                join_handle: Some(thread::spawn(move || {
                    Nscope::run(hid_device, command_rx, data2)
                })),
            })
        } else {
            None
        }
    }

    fn run(hid_device: HidDevice, command_rx: Receiver<Command>, data: Arc<RwLock<NscopeData>>) {

        let mut buf = [0u8; 64];

        loop {
            // check for an incoming command
            {
                if let Ok(command) = command_rx.try_recv() {
                    match command {
                        Command::Quit => break
                    }
                } else {
                    hid_device.write(&commands::NULL_REQ).unwrap();
                }
            }

            hid_device.read(&mut buf[..]).expect("Cannot Read from device");

            let response = responses::StatusResponse::new(&buf);
            {
                let mut data_writer = data.write().unwrap();
                data_writer.power_state = response.power_state;
                data_writer.power_usage = response.power_usage as f32 * 5.0 / 255.0;
            }
        }
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