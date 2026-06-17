use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::sync::Arc;
use tokio::runtime::Runtime;

use arkhe_kernel::testing::TestOrchestrator;

#[pyclass]
pub struct PyTestOrchestrator {
    inner: Arc<TestOrchestrator>,
    rt: Runtime,
}

#[pymethods]
impl PyTestOrchestrator {
    #[new]
    fn new(
        _spawner: PyObject,
        _attestation_manager: PyObject,
        _store: PyObject,
        _signer: PyObject,
    ) -> PyResult<Self> {
        Err(PyRuntimeError::new_err("Not fully implemented yet for PyO3 due to mocked deps"))
    }
}

#[pymodule]
fn cathedral_arkhe(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTestOrchestrator>()?;
    Ok(())
}
