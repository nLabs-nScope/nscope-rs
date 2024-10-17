/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

mod voltages_legacy;
mod voltages;

use voltages_legacy::AnalogInterfaceLegacy;
use voltages::AnalogInterfaceModern;

#[derive(Debug, Copy, Clone)]
enum AnalogInterface {
    Legacy(AnalogInterfaceLegacy),
    Modern(AnalogInterfaceModern)
}

/// Interface to a single scope channel
#[derive(Debug, Copy, Clone)]
pub struct AnalogInput {
    pub(crate) is_on: bool,
    analog_interface: AnalogInterface,
}

impl AnalogInput {
    pub(crate) fn create(is_legacy: bool) -> Self {
        let mut analog_input = match is_legacy {
            true => AnalogInput {
                is_on: true,
                analog_interface: AnalogInterface::Legacy(
                    AnalogInterfaceLegacy {
                        gain_setting: 0,
                        offset_setting: 0,
                    }),
            },
            false => AnalogInput {
                is_on: true,
                analog_interface: AnalogInterface::Modern(
                    AnalogInterfaceModern {
                    }),
            }
        };
        analog_input.set_range(-5.0, 5.0);
        analog_input
    }
}

impl AnalogInput {
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    pub fn set_range(&mut self, vmin: f64, vmax: f64) {
        match self.analog_interface {
            AnalogInterface::Legacy(mut interface) => { interface.set_range(vmin, vmax) }
            AnalogInterface::Modern(mut interface) => { interface.set_range(vmin, vmax) }
        }
    }

    pub fn gain(&self) -> f64 {
        match self.analog_interface {
            AnalogInterface::Legacy(interface) => { interface.gain() }
            AnalogInterface::Modern(interface) => { interface.gain() }
        }
    }

    pub(crate) fn measurement_from_voltage(&self, voltage: f64) -> i16 {
        match self.analog_interface {
            AnalogInterface::Legacy(interface) => { interface.measurement_from_voltage(voltage) }
            AnalogInterface::Modern(interface) => { interface.measurement_from_voltage(voltage) }
        }
    }

    pub(crate) fn voltage_from_measurement(&self, adc_data: u16) -> f64 {
        match self.analog_interface {
            AnalogInterface::Legacy(interface) => { interface.voltage_from_measurement(adc_data) }
            AnalogInterface::Modern(interface) => { interface.voltage_from_measurement(adc_data) }
        }
    }

    pub(crate) fn gain_cmd(&self) -> u8 {
        match self.analog_interface {
            AnalogInterface::Legacy(interface) => { interface.gain_setting }
            AnalogInterface::Modern(_interface) => { 0 }
        }
    }

    pub(crate) fn offset_cmd(&self) -> u8 {
        match self.analog_interface {
            AnalogInterface::Legacy(interface) => { interface.offset_setting }
            AnalogInterface::Modern(_interface) => { 0 }
        }
    }
}

