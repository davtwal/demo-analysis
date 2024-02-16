
use itertools::Itertools;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;

/// This function is used by lib.rs, but the IDE thinks it's unused.
#[allow(dead_code)]
pub(crate) fn get_submod(py: Python<'_>) -> PyResult<&PyModule> {
    let module = PyModule::new(py, "game")?;
    module.add_class::<World>()?;
    module.add_class::<Class>()?;
    module.add_class::<Team>()?;
    module.add_class::<ClassList>()?;
    module.add_class::<ClassListIter>()?;
    module.add_class::<Round>()?;

    Ok(module)
}

/////////////////////////////////////////////
/// WORLD
/// /////////////////////////////////////////

use super::math::Vector;

/// Defines the boundaries of the world as given by the demofile.
/// You can expect bound_min.x < bound_max.x, and so on.
#[pyclass(get_all)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct World {
    /// The minimum.
    pub bound_min: Vector,
    
    /// The maximum.
    pub bound_max: Vector,
}

impl World {
    pub fn adjoin_bounds(&self, other: &World) -> Self {
        World {
            bound_max: Vector {
                x: f32::max(self.bound_max.x, other.bound_max.x),
                y: f32::max(self.bound_max.y, other.bound_max.y),
                z: f32::max(self.bound_max.z, other.bound_max.z),
            },
            bound_min: Vector {
                x: f32::min(self.bound_min.x, other.bound_min.x),
                y: f32::min(self.bound_min.y, other.bound_min.y),
                z: f32::min(self.bound_min.z, other.bound_min.z),
            }
        }
    }

    pub fn stretch_to_include(&mut self, point: Vector) {
        *self = self.adjoin_bounds(&World{bound_min: point, bound_max: point});
    }
}


/////////////////////////////////////////////
/// Class
/// /////////////////////////////////////////

use tf_demo_parser::demo::parser::analyser::{Class as TFClass, ClassList as TFCList};

// missing: serde, fromstr
/// Representation of each class in the game as an enum.
use std::hash::Hash;
#[pyclass]
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive, Default)]
#[repr(u8)]
pub enum Class {
    #[default]
    Other = 0,
    Scout = 1,
    Sniper = 2,
    Soldier = 3,
    Demoman = 4,
    Medic = 5,
    Heavy = 6,
    Pyro = 7,
    Spy = 8,
    Engineer = 9,
}

impl From<TFClass> for Class {
    fn from(value: TFClass) -> Self {
        Class::new(value as u8)
    }
}

impl Class {
    pub fn new<U>(number: U) -> Self
    where
        u8: TryFrom<U>,
    {
        Class::try_from(u8::try_from(number).unwrap_or_default()).unwrap_or_default()
    }
}

use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

#[pymethods]
impl Class{
    #[new]
    fn new_py(n: u8) -> Self {
        Self::new(n)
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, comp: CompareOp) -> Self {
        match comp {
            CompareOp::Eq => *{if self == other {self} else {other}},
            CompareOp::Ge => *{if self >= other {self} else {other}},
            CompareOp::Gt => *{if self >  other {self} else {other}},
            CompareOp::Le => *{if self <= other {self} else {other}},
            CompareOp::Lt => *{if self <  other {self} else {other}},
            CompareOp::Ne => *{if self != other {self} else {other}},
        }
    }
}

/////////////////////////////////////////////
/// Class List
/// /////////////////////////////////////////

#[pyclass(sequence)]
#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub struct ClassList([u8; 10]);

impl Index<Class> for ClassList {
    type Output = u8;

    #[cfg_attr(feature = "no-panic", no_panic::no_panic)]
    fn index(&self, class: Class) -> &Self::Output {
        &self.0[class as u8 as usize]
    }
}

impl IndexMut<Class> for ClassList {
    #[cfg_attr(feature = "no-panic", no_panic::no_panic)]
    fn index_mut(&mut self, class: Class) -> &mut Self::Output {
        &mut self.0[class as u8 as usize]
    }
}

impl From<HashMap<Class, u8>> for ClassList {
    fn from(map: HashMap<Class, u8>) -> Self {
        let mut classes = ClassList::default();

        for (class, count) in map.into_iter() {
            classes[class] = count;
        }

        classes
    }
}

impl From<TFCList> for ClassList {
    fn from(value: TFCList) -> Self {
        let mut cl = ClassList::default();
        for c in 0..10 {
            cl[Class::new(c)] = value[TFClass::new(c)]
        }
        cl
    }
}

impl ClassList {
    /// Get an iterator for all classes played and the number of spawn on the class
    pub fn iter(&self) -> impl Iterator<Item = (Class, u8)> + '_ {
        self.0
            .iter()
            .copied()
            .enumerate()
            .map(|(class, count)| (Class::new(class), count))
            .filter(|(_, count)| *count > 0)
    }

    /// Get an iterator for all classes played and the number of spawn on the class, sorted by the number of spawns
    pub fn sorted(&self) -> impl Iterator<Item = (Class, u8)> {
        let mut classes = self.iter().collect::<Vec<(Class, u8)>>();
        classes.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        classes.into_iter()
    }
}

#[pymethods]
impl ClassList
{
    fn __len__(&self) -> usize {
        let mut count = 0;
        for i in 0..10 {
            if self.0[i] > 0 {
                count += 1;
            }
        }
        count
    }

    fn __contains__(&self, item: Class) -> bool {
        self[item] > 0
    }

    // TODO: __get__

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<ClassListIter>> {
        let list: Vec<(Class, u8)> = slf.iter().collect_vec();
        let iter = ClassListIter {
            inlist: list,
            cur_index: 0
        };
        Py::new(slf.py(), iter)
    }
}

#[pyclass]
pub struct ClassListIter {
    inlist: Vec<(Class, u8)>,
    cur_index: usize,
}

#[pymethods]
impl ClassListIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(Class, u8)> {
        slf.cur_index += 1;
        slf.inlist.get(slf.cur_index).copied()
    }
}

/////////////////////////////////////////////
/// Team
/// /////////////////////////////////////////

#[pyclass]
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive, Default)]
#[repr(u8)]
pub enum Team {
    #[default]
    Other = 0,
    Spectator = 1,
    Red = 2,
    Blue = 3,
}

use tf_demo_parser::demo::parser::analyser::Team as TFTeam;
impl From<TFTeam> for Team {
    fn from(value: TFTeam) -> Self {
        match value {
            TFTeam::Other => Team::Other,
            TFTeam::Spectator => Team::Spectator,
            TFTeam::Red => Team::Red,
            TFTeam::Blue => Team::Blue
        }
    }
}

impl Team {
    pub fn new<U>(number: U) -> Self
    where
        u8: TryFrom<U>,
    {
        Team::try_from(u8::try_from(number).unwrap_or_default()).unwrap_or_default()
    }
}

#[pymethods]
impl Team {
    #[new]
    fn new_py(n: u8) -> Self {
        Self::new(n)
    }

    pub fn is_player(&self) -> bool {
        *self == Team::Red || *self == Team::Blue
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, comp: CompareOp) -> Self {
        match comp {
            CompareOp::Eq => *{if self == other {self} else {other}},
            CompareOp::Ge => *{if self >= other {self} else {other}},
            CompareOp::Gt => *{if self >  other {self} else {other}},
            CompareOp::Le => *{if self <= other {self} else {other}},
            CompareOp::Lt => *{if self <  other {self} else {other}},
            CompareOp::Ne => *{if self != other {self} else {other}},
        }
    }
}

/////////////////////////////////////////////
/// Round
/// /////////////////////////////////////////

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Round {
    pub start_tick: u32,
    pub end_tick: u32,
    pub winner: Team,
}

#[pymethods]
impl Round {
    pub fn is_tie(&self) -> bool {
        match self.winner {
            Team::Blue | Team::Red => true,
            _ => false
        }
    }
}