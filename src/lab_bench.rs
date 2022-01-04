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
use hidapi::DeviceInfo;
use hidapi::HidApi;
use std::{fmt, io};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct LabBench {
    hid_devices: Vec<DeviceInfo>,
    hid_api: Arc<RwLock<HidApi>>,
}

pub struct NscopeLink {
    available: bool,
    info: DeviceInfo,
    hid_api: Arc<RwLock<HidApi>>,
}

impl LabBench {
    pub fn new() -> Result<LabBench, Box<dyn Error>> {
        let hid_api = HidApi::new()?;
        Ok(LabBench {
            hid_devices: hid_api.device_list().cloned().collect(),
            hid_api: Arc::new(RwLock::new(hid_api)),
        })
    }

    pub fn refresh(&mut self) {
        let mut api = self.hid_api.write().unwrap();
        api.refresh_devices().expect("failed to refresh");
        self.hid_devices = api.device_list().cloned().collect();
    }

    /// Returns iterator containing information about attached nScopes
    pub fn list(&self) -> impl Iterator<Item=NscopeLink> + '_ {
        self.hid_devices
            .iter()
            .filter_map(move |d| NscopeLink::new(d.clone(), Arc::clone(&self.hid_api)))
    }

    pub fn open_all_available(self) -> Vec<Nscope> {
        self.list().filter_map(|nsl| nsl.open().ok()).collect()
    }

    pub fn open_first_available(self) -> Result<Nscope, io::Error> {

        // Default error is that we found zero nScopes
        let mut err = io::Error::new(io::ErrorKind::NotFound, "Cannot find any nScopes");

        for nsl in self.list() {
            if let Ok(nscope) = nsl.open() {
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
    fn new(info: DeviceInfo, hid_api: Arc<RwLock<HidApi>>) -> Option<Self> {
        if info.vendor_id() == 0x04D8 && info.product_id() == 0xF3F6 {
            let api = hid_api.read().ok()?;
            let available = info.open_device(&api).is_ok();
            Some(NscopeLink {
                available,
                info,
                hid_api: Arc::clone(&hid_api),
            })
        } else {
            None
        }
    }

    pub fn open(&self) -> Result<Nscope, Box<dyn Error>> {
        let api = self.hid_api.read().unwrap();
        Nscope::new(&self.info, &api)
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
        write!(
            f,
            "VID: 0x{:04X}, PID: 0x{:04X}, Available: {}",
            self.info.vendor_id(),
            self.info.product_id(),
            self.available
        )
    }
}
