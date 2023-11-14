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

type RusbDevice = rusb::Device<rusb::GlobalContext>;
type HidApiDevice = (hidapi::DeviceInfo, Arc<RwLock<hidapi::HidApi>>);

/// A representation of all the nScopes plugged into a computer
pub struct LabBench {
    hid_api: Arc<RwLock<hidapi::HidApi>>,
    hid_devices: Vec<hidapi::DeviceInfo>,
    rusb_devices: Vec<RusbDevice>,
}

/// A detected link between the computer and an nScope, used to open and retrieve an nScope
pub struct NscopeLink {
    available: bool,
    hid_info: Option<HidApiDevice>,
    rusb_info: Option<RusbDevice>,
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
        let v1_iterator = self.hid_devices
            .iter()
            .filter_map(
                move |d| NscopeLink::new(
                    Some( (d.clone(), Arc::clone(&self.hid_api))),
                    None
                )
            );

        let v2_iterator = self.rusb_devices
            .iter()
            .filter_map(move |d|
                NscopeLink::new(
                    None,
                    Some(d.clone())
                )
            );

        v1_iterator.chain(v2_iterator)
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
    fn new(hid_info: Option<HidApiDevice>, rusb_info: Option<RusbDevice>) -> Option<Self> {
        if let Some((hid_device, hid_api)) = hid_info {
            if hid_device.vendor_id() == 0x04D8 && hid_device.product_id() == 0xF3F6 {
                let api = hid_api.read().ok()?;
                let available = hid_device.open_device(&api).is_ok();
                return Some(NscopeLink {
                    available,
                    hid_info: Some((hid_device, Arc::clone(&hid_api))),
                    rusb_info: None,
                });
            }
            return None;
        }

        if let Some(device) = rusb_info {
            if let Ok(device_desc) = device.device_descriptor() {
                if device_desc.vendor_id() == 0xCAFE && device_desc.product_id() == 0x1234 {
                    let available = device.open().is_ok();
                    return Some (NscopeLink {
                        available,
                        hid_info: None,
                        rusb_info: Some(device)
                    })
                }
            }
            return None;
        }

        None
    }

    /// Opens and returns the nScope at the link
    pub fn open(&self, power_on: bool) -> Result<Nscope, Box<dyn Error>> {
        if let Some((hid_device, hid_api)) = &self.hid_info {
            let api = hid_api.read().unwrap();
            return Nscope::new(hid_device, &api, power_on);
        }

        Err("Cannot find valid information to open nScope".into())
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

        if self.hid_info.is_some() {
            return write!(
                f,
                "Link to nScope v1 [ available: {} ]",
                self.available
            )
        }
        if self.rusb_info.is_some() {
            return write!(
                f,
                "Link to nScope v2 [ available: {} ]",
                self.available
            )
        }
        write!(f, "Invalid nScope")
    }
}
