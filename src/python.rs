mod bench;
mod scope;
mod analog_output;
mod pulse_output;

use pyo3::prelude::*;
use crate::{AnalogSignalPolarity, AnalogWaveType};

#[pyclass]
struct LabBench(crate::LabBench);

#[pyclass]
struct Nscope(crate::Nscope);

/// A Python module implemented in Rust.
#[pymodule]
fn nscope(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    m.add_class::<Nscope>()?;
    m.add_class::<AnalogWaveType>()?;
    m.add_class::<AnalogSignalPolarity>()?;
    Ok(())
}