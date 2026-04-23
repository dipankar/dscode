use dscode_core::{AppDirectories, TextBuffer};
use pyo3::prelude::*;

/// A rope-based text buffer for efficient editing operations.
///
/// Parameters
/// ----------
/// text : str
///     Initial content of the buffer.
///
/// Returns
/// -------
/// TextBuffer
///     A new text buffer instance.
#[pyclass]
struct PyTextBuffer {
    inner: TextBuffer,
}

#[pymethods]
impl PyTextBuffer {
    #[new]
    #[pyo3(signature = (text=""))]
    fn new(text: &str) -> Self {
        Self {
            inner: TextBuffer::new(text),
        }
    }

    /// Get the full text content of the buffer.
    fn get_text(&self) -> String {
        self.inner.get_text()
    }

    /// Get the number of characters in the buffer.
    fn len_chars(&self) -> usize {
        self.inner.len_chars()
    }
}

/// Platform-aware directory resolution for DSCode application directories.
#[pyclass]
struct PyAppDirectories {
    inner: AppDirectories,
}

#[pymethods]
impl PyAppDirectories {
    /// Resolve directories from user configuration and environment overrides.
    #[new]
    fn new() -> PyResult<Self> {
        let dirs =
            AppDirectories::resolve().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
        Ok(Self { inner: dirs })
    }

    /// Get the extensions directory path.
    #[getter]
    fn extensions_dir(&self) -> String {
        self.inner.extensions_dir.to_string_lossy().to_string()
    }

    /// Get the storage directory path.
    #[getter]
    fn storage_dir(&self) -> String {
        self.inner.storage_dir.to_string_lossy().to_string()
    }

    /// Get the logs directory path.
    #[getter]
    fn logs_dir(&self) -> String {
        self.inner.logs_dir.to_string_lossy().to_string()
    }
}

/// Python bindings for dscode-core.
///
/// Provides TextBuffer and AppDirectories for Python consumers.
#[pymodule]
fn dscode_core_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTextBuffer>()?;
    m.add_class::<PyAppDirectories>()?;
    Ok(())
}
