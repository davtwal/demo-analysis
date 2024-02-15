use pyo3::prelude::*;

use super::{DemoTick, game::Team};

use tf_demo_parser::demo::gameevent_gen::{
    PlayerDeathEvent,
    PlayerChargeDeployedEvent,
    TeamPlayPointCapturedEvent
};

// this function is used in game::mod.rs but rust_analyzer thinks not
#[allow(dead_code)]
pub(crate) fn get_submod(
    py: Python<'_>,
) -> PyResult<&PyModule> {
    let module = PyModule::new(py, "events")?;
    module.add_class::<Kill>()?;
    module.add_class::<Capture>()?;
    module.add_class::<Ubercharge>()?;
    Ok(module)
}

// Information surrounding a PlayerDeath event
#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct Kill {
    pub dead_id: u16,        // UserID who died
    pub dead_entity: u32,  // EntityID of the player who died
    pub attacker_id: u16,    // UserID of the attacker
    pub inflictor_id: u32, // EntityID of the inflictor.
                                // Notably may not be the entityID of the attacker.
    #[pyo3(get)]
    pub weapon: String,         // Weapon type

    #[pyo3(get)]
    pub weapon_id: u16,         // Weapon ID? not sure

    pub assister: Option<u16>,       // UserID that assisted in the kill

    #[pyo3(get)]
    pub dead_rocketjumping: bool,// If the player that died was rocket jumping

    pub tick: DemoTick,
}

#[pymethods]
impl Kill {
    #[getter]
    fn dead_id(&self) -> PyResult<u16> {
        Ok(u16::from(self.dead_id))
    }
    
    #[getter]
    fn dead_entity(&self) -> PyResult<u32> {
        Ok(u32::from(self.dead_entity))
    }

    #[getter]
    fn attacker_id(&self) -> PyResult<u16> {
        Ok(u16::from(self.attacker_id))
    }
    
    #[getter]
    fn inflictor_id(&self) -> PyResult<u32> {
        Ok(u32::from(self.inflictor_id))
    }

    #[getter]
    fn assister_id(&self) -> PyResult<Option<u16>> {
        Ok(match self.assister {
            Some(uid) => Some(u16::from(uid)),
            None => None            
        })
    }

    #[getter]
    fn tick(&self) -> PyResult<u32> {
        Ok(u32::from(self.tick))
    }
}

impl Kill {
    pub fn from_event(tick: DemoTick, death: &PlayerDeathEvent) -> Self {
        let assister = if death.assister < (16 * 1024) {
            Some(death.assister)
        } else {
            None
        };
        Kill {
            dead_id: death.user_id,
            dead_entity: death.victim_ent_index,
            attacker_id: death.attacker,
            inflictor_id: death.inflictor_ent_index,
            weapon: death.weapon.to_string(),
            weapon_id: death.weapon_id,
            assister,
            dead_rocketjumping: death.rocket_jump,
            tick
        }
    }
}

// capture

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct Capture {
    pub cp_index: u8,
    pub cp_name: String,
    pub team: Team,
    // user ids
    pub cappers: Vec<u16>,
    pub tick: u32
}

impl Capture {
    pub fn from_event(tick: DemoTick, capture: &TeamPlayPointCapturedEvent) -> Self {
        let cappers: Vec<u16> = capture.cappers
            .as_bytes()
            .iter()
            .cloned()
            .map(|x| x as u16)
            .collect();

        Capture {
            cp_index: capture.cp,
            cp_name: capture.cp_name.to_string(),
            team: Team::new(capture.team),
            cappers,
            tick: u32::from(tick)
        }
    }
}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct Ubercharge {
    pub medic_id: u16,
    pub ubered_id: u16,
    pub tick: u32,
}

impl Ubercharge {
    pub fn from_event(tick: DemoTick, charge: &PlayerChargeDeployedEvent) -> Self {
        Ubercharge {
            medic_id: charge.user_id,
            ubered_id: charge.target_id,
            tick: u32::from(tick),
        }
    }
}