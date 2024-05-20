mod bench;
mod scope;

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;

#[pyclass]
struct LabBench(crate::LabBench);

#[pyclass]
struct Nscope(crate::Nscope);

/// A Python module implemented in Rust.
#[pymodule]
fn nscope(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    m.add_class::<Nscope>()?;
    Ok(())
}