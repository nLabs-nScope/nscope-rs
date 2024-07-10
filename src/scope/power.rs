/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://nscope.org
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nScope API
 *
 **************************************************************************************************/

use std::io;
use pyo3::{pyclass, pymethods};
use super::Nscope;

/// Information about the power supply status of nScope
#[derive(Debug, Copy, Clone)]
#[pyclass]
pub struct PowerStatus {
    #[pyo3(get)]
    pub state: PowerState,
    #[pyo3(get)]
    pub usage: f64,
}

impl Default for PowerStatus {
    fn default() -> Self {
        PowerStatus {
            state: PowerState::Unknown,
            usage: 0.0,
        }
    }
}

#[pymethods]
impl PowerStatus {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

/// Possible states of the nScope power supply
#[derive(Debug, PartialEq, Copy, Clone)]
#[pyclass]
pub enum PowerState {
    PowerOff,
    PowerOn,
    Shorted,
    Overcurrent,
    Startup,
    Unknown,
}

impl From<u8> for PowerState {
    fn from(orig: u8) -> Self {
        match orig {
            0 => PowerState::PowerOff,
            1 => PowerState::PowerOn,
            2 => PowerState::Shorted,
            3 => PowerState::Overcurrent,
            4 => PowerState::Startup,
            _ => PowerState::Unknown,
        }
    }
}

impl Nscope {
    pub fn power_status(&self) -> Result<PowerStatus, io::Error> {
        if !self.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "nScope connection aborted",
            ));
        }
        Ok(*self.power_status.read().unwrap())
    }
}
