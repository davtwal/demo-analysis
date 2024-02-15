//! The library that Python gets access to.
//! 
//! [`demo_analysis_lib`]` is run during `maturin develop` to add the classes
//! necessary to `demo_analysis_lib.pyd`.

mod types;
mod parsing;
mod datatransmit;
mod app;

use std::path::PathBuf;
use pyo3::prelude::*;
use types::demo::DemoData;
//use crate::types::{math, demo, game};

/// Adds all of the types to the python library.
#[pymodule]
fn tf2dal(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn load_demo(fname: PathBuf) -> PyResult<DemoData> {
        Ok(crate::app::do_parses(vec![fname])?
            .get(0).ok_or(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "could not find demo file")
            )?.to_owned().1 // Could potentially avoid the .to_owned here?
        )
    }
    
    #[pyfn(m)]
    fn load_demo_rounds(fname: PathBuf) -> PyResult<Vec<DemoData>> {
        let data = load_demo(fname)?;

        let mut retvec = Vec::new();
        for round in &data.rounds {
            retvec.push(data.round_data(round).into())
        }

        Ok(retvec)
    }

    let loader_fns = vec![
        types::demo::get_submod,
        types::entities::get_submod,
        types::events::get_submod,
        types::game::get_submod,
        types::math::get_submod,
    ];

    for func in loader_fns {
        m.add_submodule(func(py)?)?;
    }

    Ok(())
}
