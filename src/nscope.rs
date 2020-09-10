/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use hidapi::{HidDevice, HidApi, DeviceInfo};
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Nscope {
    vid: u16,
    pid: u16,
    hid_device: Option<HidDevice>,
    hid_api: Rc<RefCell<HidApi>>,
}

impl fmt::Debug for Nscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VID: 0x{:04X}, PID: 0x{:04X}, Open: {}",
            self.vid,
            self.pid,
            self.hid_device.is_some(),
        )
    }
}

impl Nscope {
    pub(crate) fn new(dev: &DeviceInfo, hid_api: &Rc<RefCell<HidApi>>) -> Option<Nscope> {
        let api = hid_api.try_borrow().unwrap();
        if let Ok(hid_device) = dev.open_device(&api) {
            Some(Nscope{
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                hid_device: Some(hid_device),
                hid_api: Rc::clone(&hid_api),
            })
        } else {
            None
        }
    }
}