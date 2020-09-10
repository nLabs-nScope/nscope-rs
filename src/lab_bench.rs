/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use hidapi::HidApi;
use hidapi::DeviceInfo;
use crate::Nscope;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

pub struct LabBench {
    hid_devices: Vec<DeviceInfo>,
    hid_api: Rc<RefCell<HidApi>>,
}

pub struct NscopeLink {
    available: bool,
    info: DeviceInfo,
    hid_api: Rc<RefCell<HidApi>>,
}

impl LabBench {
    pub fn new() -> Option<LabBench> {
        match HidApi::new() {
            Ok(hid_api) => Some(LabBench {
                hid_devices: hid_api.device_list().map(|d| d.clone()).collect(),
                hid_api: Rc::new(RefCell::new(hid_api)),
            }),
            Err(_) => None
        }
    }

    pub fn refresh(&mut self) {
        let mut api = self.hid_api.try_borrow_mut().unwrap();
        api.refresh_devices().expect("failed to refresh");
        self.hid_devices = api.device_list().map(|d| d.clone()).collect();
    }

    /// Returns iterator containing information about attached nScopes
    pub fn list(&self) -> impl Iterator<Item=NscopeLink> + '_ {
        self.hid_devices.iter().filter_map(move |d| NscopeLink::new(d.clone(), Rc::clone(&self.hid_api)))
    }
}


impl NscopeLink {
    fn new(info: DeviceInfo, hid_api: Rc<RefCell<HidApi>>) -> Option<NscopeLink> {
        if info.vendor_id() == 0x04D8 && info.product_id() == 0xF3F6 {
            let api = hid_api.try_borrow().unwrap();
            let available = match info.open_device(&api) {
                Ok(_) => true,
                Err(_) => false,
            };
            Some(NscopeLink { available, info, hid_api: Rc::clone(&hid_api) })
        } else {
            None
        }
    }

    pub fn open(&self) -> Option<Nscope> {
        Nscope::new(&self.info, &self.hid_api)
    }
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LabBench: {:#?}", self.list().collect::<Vec<NscopeLink>>()
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
