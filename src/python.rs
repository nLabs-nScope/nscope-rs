use pyo3::exceptions::*;
use pyo3::prelude::*;

#[pyclass]
struct LabBench(crate::LabBench);

#[pymethods]
impl LabBench {
    #[new]
    fn new() -> PyResult<Self> {
        match crate::LabBench::new() {
            Ok(bench) => Ok(LabBench(bench)),
            Err(_) => Err(PyRuntimeError::new_err("Cannot create LabBench")),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn nscope(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    Ok(())
}
