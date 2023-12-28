/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::error::Error;
use std::fmt;

use crate::firmware::FIRMWARE;

pub struct NscopeDFU {
    device: rusb::Device<rusb::GlobalContext>,
}


impl NscopeDFU {
    pub(crate) fn new(device: &rusb::Device<rusb::GlobalContext>) -> Option<Self> {
        if let Ok(device_desc) = device.device_descriptor() {
            if device_desc.vendor_id() == 0x0483 && device_desc.product_id() == 0xA4AB {
                return Some(NscopeDFU { device: device.clone() });
            }
        }
        None
    }

    pub fn update(&self) -> Result<(), Box<dyn Error>> {
        let mut dfu = dfu_libusb::DfuLibusb::from_usb_device(
            self.device.clone(),
            self.device.open()?,
            0, 0)?;

        dfu.override_address(0x08008000);
        dfu.download_from_slice(FIRMWARE)?;
        dfu.detach()?;
        println!("Resetting device");
        dfu.usb_reset()?;

        Ok(())
    }
}

impl fmt::Debug for NscopeDFU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nScope in DFU mode: {:?}", &self.device)
    }
}