/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use crate::scope::Nscope;
use std::{fmt, io};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub(crate) enum NscopeDevice {
    HidApiDevice { info: hidapi::DeviceInfo, api: Arc<RwLock<hidapi::HidApi>> },
    RusbDevice(rusb::Device<rusb::GlobalContext>),
}

/// A representation of all the nScopes plugged into a computer
pub struct LabBench {
    hid_api: Arc<RwLock<hidapi::HidApi>>,
    hid_devices: Vec<hidapi::DeviceInfo>,
    rusb_devices: Vec<rusb::Device<rusb::GlobalContext>>,
}

/// A detected link between the computer and an nScope, used to open and retrieve an nScope
pub struct NscopeLink {
    available: bool,
    device: NscopeDevice,
}


impl LabBench {
    /// Creates a new lab bench, searching the computer for nScope links
    pub fn new() -> Result<LabBench, Box<dyn Error>> {
        let hid_api = hidapi::HidApi::new()?;
        Ok(LabBench {
            hid_devices: hid_api.device_list().cloned().collect(),
            hid_api: Arc::new(RwLock::new(hid_api)),
            rusb_devices: rusb::devices().unwrap().iter().collect(),
        })
    }

    /// Refreshes the list of nScope Links
    pub fn refresh(&mut self) {
        let mut api = self.hid_api.write().unwrap();
        api.refresh_devices().expect("failed to refresh");
        self.hid_devices = api.device_list().cloned().collect();
        self.rusb_devices = rusb::devices().unwrap().iter().collect();
    }

    /// Returns iterator containing information about detected nScopes plugged into the computer
    pub fn list(&self) -> impl Iterator<Item=NscopeLink> + '_ {
        let v1_nscopes = self.hid_devices
            .iter()
            .filter_map(move |d| NscopeLink::new(
                NscopeDevice::HidApiDevice {
                    info: d.clone(),
                    api: Arc::clone(&self.hid_api),
                }
            ));

        let v2_nscopes = self.rusb_devices
            .iter()
            .filter_map(move |d| NscopeLink::new(
                NscopeDevice::RusbDevice(d.clone())
            ));

        v1_nscopes.chain(v2_nscopes)
    }

    /// Returns a vector containing all nScopes that are available
    pub fn open_all_available(&self) -> Vec<Nscope> {
        self.list().filter_map(|nsl| nsl.open(false).ok()).collect()
    }

    /// Returns the first available nScope
    pub fn open_first_available(&self, power_on: bool) -> Result<Nscope, io::Error> {

        // Default error is that we found zero nScopes
        let mut err = io::Error::new(io::ErrorKind::NotFound, "Cannot find any nScopes");


        for nsl in self.list() {
            if let Ok(nscope) = nsl.open(power_on) {
                // return the first open nScope
                return Ok(nscope);
            }
            // If we've gotten here, then the error is that we cannot open an nScope
            err = io::Error::new(io::ErrorKind::ConnectionRefused, "Cannot connect to any nScopes");
        }
        Err(err)
    }
}

impl NscopeLink {
    fn new(device: NscopeDevice) -> Option<Self> {
        match device {
            NscopeDevice::HidApiDevice { info, api } => {
                if info.vendor_id() == 0x04D8 && info.product_id() == 0xF3F6 {
                    let hid_api = api.read().ok()?;
                    let available = info.open_device(&hid_api).is_ok();
                    return Some(NscopeLink {
                        available,
                        device: NscopeDevice::HidApiDevice { info: info.clone(), api: Arc::clone(&api) },
                    });
                }
                None
            }
            NscopeDevice::RusbDevice(device) => {
                if let Ok(device_desc) = device.device_descriptor() {
                    if device_desc.vendor_id() == 0xCAFE && device_desc.product_id() == 0x1234 {
                        let available = device.open().is_ok();
                        return Some(NscopeLink {
                            available,
                            device: NscopeDevice::RusbDevice(device),
                        });
                    }
                }
                None
            }
        }
    }

    /// Opens and returns the nScope at the link
    pub fn open(&self, power_on: bool) -> Result<Nscope, Box<dyn Error>> {
        Nscope::new(&self.device, power_on)
    }
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LabBench: {:#?}",
            self.list().collect::<Vec<NscopeLink>>()
        )
    }
}

impl fmt::Debug for NscopeLink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let device_name = match &self.device {
            NscopeDevice::HidApiDevice { .. } => { "nScope v1" }
            NscopeDevice::RusbDevice(_) => { "nScope v2" }
        };
        write!(
            f,
            "Link to {} [ available: {} ]",
            device_name,
            self.available
        )
    }
}
