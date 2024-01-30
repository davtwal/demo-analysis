//! The library that Python gets access to.
//! 
//! [`demo_analysis_lib`]` is run during `maturin develop` to add the classes
//! necessary to `demo_analysis_lib.pyd`.

mod types;

use pyo3::prelude::*;
use crate::types::{math, demo, game};

/// Adds all of the types to the python library.
#[pymodule]
fn demo_analysis_lib(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<math::Vector>()?;
    m.add_class::<math::VectorXY>()?;
    m.add_class::<demo::DemoData>()?;
    m.add_class::<demo::TickData>()?;

    game::register_with(_py, m)?;
    Ok(())
}
