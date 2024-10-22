use pyo3::exceptions::*;
use pyo3::prelude::*;

use crate::{PowerStatus, python, Sample};

#[pymethods]
impl python::Nlab {
    fn is_connected(&self) -> bool {
        let scope: &crate::Nlab = &self.0;
        scope.is_connected()
    }

    fn close(&mut self) {
        let scope: &mut crate::Nlab = &mut self.0;
        scope.close()
    }

    fn version(&self) -> PyResult<u16> {
        let scope: &crate::Nlab = &self.0;
        match scope.version() {
            Ok(version) => Ok(version),
            Err(_) => Err(PyRuntimeError::new_err("Cannot read nScope version")),
        }
    }

    fn power_status(&self) -> PyResult<PowerStatus> {
        let scope: &crate::Nlab = &self.0;
        match scope.power_status() {
            Ok(status) => Ok(status),
            Err(error) => Err(PyRuntimeError::new_err(error)),
        }
    }

    fn read_all_channels(&mut self, sample_rate: f64, number_of_samples: u32) -> PyResult<Vec<Vec<Option<f64>>>> {
        let scope: &mut crate::Nlab = &mut self.0;
        scope.ch1.turn_on();
        scope.ch2.turn_on();
        scope.ch3.turn_on();
        scope.ch4.turn_on();
        let sweep_handle = scope.request(sample_rate, number_of_samples, None);

        let mut return_data: Vec<Vec<Option<f64>>> = Vec::new();

        for _ in 0..Sample::num_channels() {
            return_data.push(Vec::new());
        }

        for sample in sweep_handle.receiver {
            for (ch, &data) in sample.data.iter().enumerate() {
                return_data[ch].push(data);
            }
        }
        Ok(return_data)
    }
}
