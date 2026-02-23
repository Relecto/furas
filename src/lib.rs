use pyo3::prelude::*;
pub mod signature;
pub mod model;
pub mod utils;
mod math;

/// A Python module implemented in Rust.
#[pymodule]
mod furas {
    use pyo3::prelude::*;
    pub use crate::signature::*;

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}
