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
use std::time::Duration;
use crate::firmware::{FIRMWARE, FIRMWARE_VERSION};

#[derive(Clone)]
pub(crate) struct HidDevice(hidapi::DeviceInfo);

pub(crate) enum NscopeDevice {
    HidApiDevice { device: HidDevice, api: Arc<RwLock<hidapi::HidApi>> },
    RusbDevice(rusb::Device<rusb::GlobalContext>),
}

impl PartialEq<Self> for HidDevice {
    fn eq(&self, other: &Self) -> bool {
        if self.vendor_id() != other.vendor_id() {
            return false;
        }
        if self.product_id() != other.product_id() {
            return false;
        }
        if self.0.path() != other.0.path() {
            return false;
        }
        true
    }
}

impl HidDevice {
    pub(crate) fn vendor_id(&self) -> u16 { self.0.vendor_id() }
    pub(crate) fn product_id(&self) -> u16 { self.0.product_id() }
    pub(crate) fn open_device(&self, api: &hidapi::HidApi) -> hidapi::HidResult<hidapi::HidDevice> { self.0.open_device(api) }
}


/// A representation of all the nScopes plugged into a computer
pub struct LabBench {
    hid_api: Arc<RwLock<hidapi::HidApi>>,
    hid_devices: Vec<HidDevice>,
    rusb_devices: Vec<rusb::Device<rusb::GlobalContext>>,
}

/// A detected link between the computer and an nScope, used to open and retrieve an nScope
pub struct NscopeLink {
    pub available: bool,
    pub in_dfu: bool,
    pub needs_update: bool,
    device: NscopeDevice,
}


impl LabBench {
    /// Creates a new lab bench, searching the computer for nScope links
    pub fn new() -> Result<LabBench, Box<dyn Error>> {
        let hid_api = hidapi::HidApi::new()?;
        Ok(LabBench {
            hid_devices: hid_api.device_list().cloned().map(HidDevice).collect(),
            hid_api: Arc::new(RwLock::new(hid_api)),
            rusb_devices: rusb::devices().unwrap().iter().collect(),
        })
    }

    /// Refreshes the list of nScope Links
    pub fn refresh(&mut self) {
        let mut api = self.hid_api.write().unwrap();
        api.refresh_devices().expect("failed to refresh");
        self.hid_devices = api.device_list().cloned().map(HidDevice).collect();
        self.rusb_devices = rusb::devices().unwrap().iter().collect();
    }

    /// Returns iterator containing information about detected nScopes plugged into the computer
    pub fn list(&self) -> impl Iterator<Item=NscopeLink> + '_ {
        let v1_nscopes = self.hid_devices
            .iter()
            .filter_map(move |d| NscopeLink::new(
                NscopeDevice::HidApiDevice {
                    device: d.clone(),
                    api: Arc::clone(&self.hid_api),
                }
            ));

        let v2_nscopes = self.rusb_devices
            .iter()
            .filter_map(move |d| NscopeLink::new(
                NscopeDevice::RusbDevice(d.clone())
            ));

        v2_nscopes.chain(v1_nscopes)
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

    /// Returns the first nScope that is in DFU mode
    pub fn get_first_in_dfu(&self) -> Option<NscopeLink> {
        for nsl in self.list() {
            if nsl.in_dfu {
                return Some(nsl)
            }
        }
        None
    }

    /// Returns the first nScope that is available and needs an update
    pub fn get_first_needing_update(&self) -> Option<NscopeLink> {
        for nsl in self.list() {
            if nsl.needs_update && nsl.available {
                return Some(nsl)
            }
        }
        None
    }
}

impl NscopeLink {
    fn new(device: NscopeDevice) -> Option<Self> {
        match device {
            NscopeDevice::HidApiDevice { device: info, api } => { NscopeLink::from_hid_device(info, api) }
            NscopeDevice::RusbDevice(device) => { NscopeLink::from_rusb_device(device) }
        }
    }

    fn from_hid_device(info: HidDevice, api: Arc<RwLock<hidapi::HidApi>>) -> Option<Self> {
        if info.vendor_id() == 0x04D8 && info.product_id() == 0xF3F6 {
            let hid_api = api.read().ok()?;
            let available = info.open_device(&hid_api).is_ok();
            return Some(NscopeLink {
                available,
                in_dfu: false,
                needs_update: false,
                device: NscopeDevice::HidApiDevice { device: info.clone(), api: Arc::clone(&api) },
            });
        }
        None
    }

    fn from_rusb_device(device: rusb::Device<rusb::GlobalContext>) -> Option<Self> {
        if let Ok(device_desc) = device.device_descriptor() {
            let vendor_id = device_desc.vendor_id();
            let product_id = device_desc.product_id();
            let firmware_version = device_desc.device_version();

            if vendor_id == 0x0483 && product_id == 0xA4AA {
                let mut available = false;
                if let Ok(mut dev) = device.open() {
                    if let Ok(()) = dev.claim_interface(0) {
                        available = true;
                    }
                }
                return Some(NscopeLink {
                    available,
                    in_dfu: false,
                    needs_update: firmware_version != rusb::Version::from_bcd(FIRMWARE_VERSION),
                    device: NscopeDevice::RusbDevice(device),
                });
            } else if device_desc.vendor_id() == 0x0483 && device_desc.product_id() == 0xA4AB {
                return Some(NscopeLink {
                    available: false,
                    in_dfu: true,
                    needs_update: false,
                    device: NscopeDevice::RusbDevice(device),
                });
            }
        }
        None
    }


    ///
    /// Takes an NscopeLink and checks to ensure the device is still connected.
    ///
    /// Returns a validated NscopeLink if the device is still connected, otherwise returns None
    ///
    pub fn validate(self) -> Option<Self> {
        match self.device {
            NscopeDevice::HidApiDevice { device: info, api } => {
                let detected_devices: Option<Vec<HidDevice>> = match api.write() {
                    Ok(mut hid_api) => {
                        if hid_api.refresh_devices().is_ok() {
                            Some(hid_api.device_list().cloned().map(HidDevice).collect())
                        } else {
                            None
                        }
                    }
                    Err(_) => { None }
                };

                if let Some(device_list) = detected_devices {
                    for device in device_list {
                        if device == info {
                            return NscopeLink::from_hid_device(device, api);
                        }
                    }
                }
                None
            }
            NscopeDevice::RusbDevice(existing_device) => {
                if let Ok(devices) = rusb::devices() {
                    for detected_device in devices.iter() {
                        if existing_device == detected_device {
                            return NscopeLink::from_rusb_device(detected_device);
                        }
                    }
                }
                None
            }
        }
    }


    /// Opens and returns the nScope at the link
    ///
    /// Fails if the nScope is in DFU mode or needs an update
    pub fn open(&self, power_on: bool) -> Result<Nscope, Box<dyn Error>> {
        if self.in_dfu {
            return Err("nScope is in DFU mode".into());
        }
        if self.needs_update {
            return Err("nScope needs a firmware update".into());
        }
        Nscope::new(&self.device, power_on)
    }

    /// Update the nScope at the link
    ///
    /// Fails if the nScope is not in DFU mode
    pub fn update(&self) -> Result<(), Box<dyn Error>> {
        if !self.in_dfu {
            return Err("nScope is not in DFU mode".into());
        }

        match &self.device {
            NscopeDevice::HidApiDevice { .. } => {
                return Err("Cannot update nScope v1".into());
            }
            NscopeDevice::RusbDevice(device) => {
                let mut dfu = dfu_libusb::DfuLibusb::from_usb_device(
                    device.clone(),
                    device.open()?,
                    0, 0)?;
                dfu.override_address(0x08010000);
                dfu.download_from_slice(FIRMWARE)?;
            }
        };
        Ok(())
    }

    /// Requests the nScope to jump to DFU mode
    ///
    /// Fails if the nScope is in DFU mode or is unavailable
    pub fn request_dfu(&self) -> Result<(), Box<dyn Error>> {
        if self.in_dfu {
            return Err("nScope is already in DFU mode".into());
        }
        match &self.device {
            NscopeDevice::HidApiDevice { .. } => {
                return Err("Unsupported for nScope v1".into());
            }
            NscopeDevice::RusbDevice(device) => {
                let out_buffer = [0u8, 6u8];
                let mut device_handle = device.open()?;
                device_handle.claim_interface(0)?;
                device_handle.write_bulk(0x01, &out_buffer, Duration::from_millis(100))?;
            }
        };
        Ok(())
    }
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.list().collect::<Vec<NscopeLink>>())
    }
}

impl fmt::Debug for NscopeLink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let device_name = match &self.device {
            NscopeDevice::HidApiDevice { .. } => { "nScope v1" }
            NscopeDevice::RusbDevice(_) => { "nScope v2" }
        };
        if self.in_dfu {
            write!(
                f,
                "Link to {} [ in DFU mode ]",
                device_name,
            )
        } else if self.needs_update {
            write!(
                f,
                "Link to {} [ REQUIRES FIRMWARE UPDATE ] [ available: {} ]",
                device_name,
                self.available,
            )
        } else {
            write!(
                f,
                "Link to {} [ available: {} ]",
                device_name,
                self.available,
            )
        }
    }
}
