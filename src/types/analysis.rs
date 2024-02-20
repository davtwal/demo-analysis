//! As Python is slow, this section is specifically for doing
//! analysis that would require large amounts of computing power.

use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::HashMap;

use super::demo::DemoData;

#[allow(dead_code)]
pub(crate) fn get_submod<'a>(
    py: Python<'a>
) -> PyResult<&'a PyModule> {
    let module = PyModule::new(py, "analysis")?;

    module.add_wrapped(wrap_pyfunction!(get_intervals))?;

    Ok(module)
}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub start_tick: u32,
    pub tick_duration: u32,
}

#[pymethods]
impl Interval {
    pub fn tick_in_interval(&self, tick: u32) -> bool {
        tick >= self.start_tick && tick <= self.start_tick + self.tick_duration
    }
}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct PlayerIntervals {
    /// Intervals in which the player is being healed
    pub healed: Vec<Interval>,

    /// Intervals in which the player has critheals
    pub critheals: Vec<Interval>,

    /// Intervals in which the player is alive
    pub alive: Vec<Interval>,
}

#[pyfunction]
/// Compute intervals for each player on each team.
/// Intervals include: 
fn get_intervals(demo: DemoData) -> PyResult<()> {

    let ret = HashMap::<u16, Vec<Interval>>::new();

    for (tick, data) in demo.tick_states
                            .into_iter()
                            .sorted_by(|(tick1, _), (tick2, _)| Ord::cmp(tick1, tick2))
    {
        for player in data.players {

        }
    }

    Ok(())
}