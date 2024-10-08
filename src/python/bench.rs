use pyo3::exceptions::*;
use pyo3::prelude::*;
use crate::{LabBench, python};

#[pymethods]
impl python::LabBench {
    #[staticmethod]
    fn open_first_available() -> PyResult<python::Nscope> {
        if let Ok(bench) = LabBench::new() {
            return match bench.open_first_available(true) {
                Ok(scope) => Ok(python::Nscope(scope)),
                Err(err) => Err(PyRuntimeError::new_err(err)),
            };
        }
        Err(PyRuntimeError::new_err("Cannot create LabBench"))
    }

    #[staticmethod]
    fn list_all_nscopes() {
        if let Ok(bench) = LabBench::new() {
            for nscope_link in bench.list() {
                println!("{:?}", nscope_link);
            }
        } else {
            println!("Cannot create LabBench");
        }
    }
}
