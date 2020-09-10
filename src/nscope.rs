use hidapi::{HidDevice, HidApi, DeviceInfo};
use std::fmt;

pub struct Nscope<'api> {
    vid: u16,
    pid: u16,
    hid_device: Option<HidDevice>,
    hid_api: &'api HidApi,
}

impl fmt::Debug for Nscope<'_> {
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

impl<'api> Nscope<'api> {
    pub(crate) fn new(dev: &DeviceInfo, hidapi: &'api HidApi) -> Option<Nscope<'api>> {
        if let Ok(hid_device) = dev.open_device(hidapi) {
            Some(Nscope{
                vid: dev.vendor_id(),
                pid: dev.product_id(),
                hid_device: Some(hid_device),
                hid_api: hidapi,
            })
        } else {
            None
        }
    }
}