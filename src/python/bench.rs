use pyo3::exceptions::*;
use pyo3::prelude::*;
use crate::{LabBench, python};

#[pymethods]
impl python::LabBench {
    #[staticmethod]
    fn open_first_available() -> PyResult<python::Nlab> {
        if let Ok(bench) = LabBench::new() {
            return match bench.open_first_available(true) {
                Ok(scope) => Ok(python::Nlab(scope)),
                Err(err) => Err(PyRuntimeError::new_err(err)),
            };
        }
        Err(PyRuntimeError::new_err("Cannot create LabBench"))
    }

    #[staticmethod]
    fn list_all_nlabs() {
        if let Ok(bench) = LabBench::new() {
            for nlab_link in bench.list() {
                println!("{:?}", nlab_link);
            }
        } else {
            println!("Cannot create LabBench");
        }
    }
}
