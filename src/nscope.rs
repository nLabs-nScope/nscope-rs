use crate::NscopeError;
use crate::NscopeError::{BenchError, UnknownError};
use hidapi::DeviceInfo;
use hidapi::HidDevice;
use std::ffi;
use std::fmt;

pub struct Nscope {
    pub(super) name: String,
    path: ffi::CString,
    vid: u16,
    pid: u16,
    hid_device: Option<HidDevice>,
}

impl fmt::Debug for Nscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Name: {:?}, ", self.name)?;
        write!(f, "VID: 0x{:04X}, PID: 0x{:04X}, ", self.vid, self.pid)?;
        write!(f, "Open: {}", self.hid_device.is_some())
    }
}

impl Nscope {
    pub fn new(d: &DeviceInfo) -> Nscope {
        let path = ffi::CString::from(d.path());
        let vid = d.vendor_id();
        let pid = d.product_id();
        Nscope {
            name: String::from(""),
            path,
            vid,
            pid,
            hid_device: None,
        }
    }
    pub fn open(&mut self, api: &hidapi::HidApi) -> Result<(), NscopeError> {
        if self.hid_device.is_some() {
            return Err(BenchError {
                message: "nScope is already open".to_string(),
            });
        }
        match api.open_path(self.path.as_c_str()) {
            Ok(device) => {
                self.hid_device = Some(device);
                Ok(())
            }
            Err(e) => Err(UnknownError {
                message: e.to_string(),
            }),
        }
    }
}
