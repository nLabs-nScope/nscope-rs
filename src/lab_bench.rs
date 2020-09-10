use hidapi::HidApi;
use hidapi::DeviceInfo;
use crate::Nscope;
use std::fmt;

pub struct LabBench {
    hid_api: HidApi
}

pub struct NscopeInfo<'api> {
    available: bool,
    info: DeviceInfo,
    hid_api: &'api HidApi,
}

impl LabBench {
    pub fn new() -> Option<LabBench> {
        if let Ok(hid_api) = HidApi::new() {
            Some(LabBench{hid_api})
        } else {
            None
        }

    }

    pub fn refresh(&mut self) {
        self.hid_api.refresh_devices().expect("poop");
    }

    /// Returns iterator containing information about attached HID devices.
    pub fn list(&self) -> impl Iterator<Item=NscopeInfo> + '_  {
        self.hid_api.device_list().filter_map(move |d| NscopeInfo::new(d.clone(), &self.hid_api) )
    }
}


impl<'api> NscopeInfo<'api> {
    fn new(info: DeviceInfo, hid_api: &'api HidApi) -> Option<NscopeInfo> {
        if info.vendor_id() == 0x04D8 && info.product_id() == 0xF3F6 {
            let available = match info.open_device(hid_api) {
                Ok(_) => true,
                Err(_) => false,
            };
            Some(NscopeInfo { available, info, hid_api})
        } else {
            None
        }

    }

    pub fn open(&self) -> Option<Nscope> {
        Nscope::new(&self.info, self.hid_api)
    }
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LabBench: {:#?}", self.list().collect::<Vec<NscopeInfo>>()
        )
    }
}

impl fmt::Debug for NscopeInfo<'_> {
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
