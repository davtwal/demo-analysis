use pyo3::prelude::*;

use crate::types::demo::{DemoData, TickData};

const DEMO_ANALYSIS_PY: &str = "python/demoanalysis.py";

#[allow(dead_code)]
pub fn launch_demo_analysis(demodata: &DemoData) {
    Python::with_gil(move |py| -> PyResult<()> {
        let m = PyModule::from_code(py,
            std::fs::read_to_string(DEMO_ANALYSIS_PY).unwrap().as_str(),
            "demoanalysis.py",
            "demoanalysis"
        )?;

        let args = (demodata.clone(),);

        m.getattr("demo_analysis_main")?.call1(args)?;

        Ok(())
    }).unwrap();
}

#[allow(dead_code)]
pub fn launch_tick_analysis(tickdata: &TickData) {
    Python::with_gil(move |py| -> PyResult<()> {
        let m = PyModule::from_code(py,
            std::fs::read_to_string(DEMO_ANALYSIS_PY).unwrap().as_str(),
            "demoanalysis.py",
            "demoanalysis"
        )?;

        let args = (tickdata.clone(),);

        m.getattr("tick_analysis_main")?.call1(args)?;

        Ok(())
    }).unwrap();
}

#[allow(dead_code)]
pub fn launch_get_player_groupings(tickdata: &TickData) {
    Python::with_gil(move |py| -> PyResult<()> {
        let m = PyModule::from_code(py,
            std::fs::read_to_string(DEMO_ANALYSIS_PY).unwrap().as_str(),
            "demoanalysis.py",
            "demoanalysis"
        )?;

        let args = (tickdata.clone(),);

        m.getattr("get_player_groupings")?.call1(args)?;

        Ok(())
    }).unwrap();
}