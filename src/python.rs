mod bench;
mod scope;
mod analog_output;
mod pulse_output;

use pyo3::prelude::*;
use crate::{AnalogSignalPolarity, AnalogWaveType, PowerStatus, PowerState};

#[pyclass]
struct LabBench;

#[pyclass]
struct Nlab(crate::Nlab);

/// A Python module implemented in Rust.
#[pymodule]
fn nlabapi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    m.add_class::<Nlab>()?;
    m.add_class::<AnalogWaveType>()?;
    m.add_class::<AnalogSignalPolarity>()?;
    m.add_class::<PowerState>()?;
    m.add_class::<PowerStatus>()?;
    Ok(())
}