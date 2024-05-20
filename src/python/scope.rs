use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::*;
use crate::python;

#[pymethods]
impl python::Nscope {
    fn is_connected(&self) -> bool {
        let scope: &crate::Nscope = &self.0;
        scope.is_connected()
    }
}
