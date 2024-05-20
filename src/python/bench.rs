use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;
use crate::python;

#[pymethods]
impl python::LabBench {
    #[new]
    fn new() -> PyResult<Self> {
        match crate::LabBench::new() {
            Ok(bench) => Ok(python::LabBench(bench)),
            Err(_) => Err(PyRuntimeError::new_err("Cannot create LabBench")),
        }
    }

    fn open_first_available(&self) -> PyResult<python::Nscope> {
        let bench = &self.0;
        match bench.open_first_available(true) {
            Ok(scope) => Ok(python::Nscope(scope)),
            Err(err) => Err(PyRuntimeError::new_err(err)),
        }
    }

    fn list_all_nscopes(&mut self) {
        let bench: &mut crate::LabBench = &mut self.0;
        bench.refresh();
        for nscope_link in bench.list() {
            println!("{:?}", nscope_link);
        }
    }
}
