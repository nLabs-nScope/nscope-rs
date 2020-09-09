use hidapi::DeviceInfo;
use hidapi::HidDevice;
use std::fmt;

pub struct Nscope {
    vid: u16,
    pid: u16,
    pub(crate) hid_device: Option<HidDevice>,
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
    pub fn operate(&mut self) {
        self.pid = 0xFFFF;
    }

    pub(crate) fn new(d: &DeviceInfo) -> Nscope {
        let vid = d.vendor_id();
        let pid = d.product_id();
        Nscope {
            vid,
            pid,
            hid_device: None,
        }
    }
}