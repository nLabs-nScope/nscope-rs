use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;

#[pyclass]
struct LabBench(crate::LabBench);

#[pyclass]
struct Nscope(crate::Nscope);

#[pymethods]
impl LabBench {
    #[new]
    fn new() -> PyResult<Self> {
        match crate::LabBench::new() {
            Ok(bench) => Ok(LabBench(bench)),
            Err(_) => Err(PyRuntimeError::new_err("Cannot create LabBench")),
        }
    }

    fn open_first_available(&self) -> PyResult<Nscope> {
        let bench = &self.0;
        match bench.open_first_available(true) {
            Ok(scope) => Ok(Nscope(scope)),
            Err(err) => Err(PyRuntimeError::new_err(err)),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn nscope(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LabBench>()?;
    Ok(())
}
