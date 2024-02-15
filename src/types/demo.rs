
use std::collections::HashMap;

use itertools::Itertools;
use pyo3::prelude::*;

use super::{DemoTick, EntityId};
use super::entities::{
    Player, Sentry, Dispenser, Teleporter, Building, Projectile,
    Medigun, ProjectileType, BuildingClass
};
use super::events::{Kill, Capture};
use super::game::{Round, World};

/// This function is used by lib.rs, but the IDE thinks it's unused.
#[allow(dead_code)]
pub(crate) fn get_submod<'a>(
    py: Python<'a>
) -> PyResult<&'a PyModule> {
    let module = PyModule::new(py, "demo")?;

    module.add_class::<TickData>()?;
    module.add_class::<DemoData>()?;

    Ok(module)
}

#[pyclass]
#[derive(Default, Debug, Clone)]
pub struct TickData {
    // Formerly gamestate internals
    #[pyo3(get)]
    pub players: Vec<Player>,
    pub projectiles: HashMap<u32, Projectile>,
    pub buildings: HashMap<u32, Building>,
    pub mediguns: HashMap<u32, Medigun>,
    pub tick: DemoTick,

    #[pyo3(get)]
    pub tick_delta: f32,    // This is the amount of time (in seconds) that passed after the previous tick.
}

impl TickData {
    pub fn get_or_create_player<T: Copy>(&mut self, entity_id: T) -> &mut Player where u32: From<T>{
        let index = match self
            .players
            .iter()
            .enumerate()
            .find(|(_index, player)| player.entity == u32::from(entity_id))
            .map(|(index, _)| index)
        {
            Some(index) => index,
            None => {
                let index = self.players.len();
                self.players.push(Player {
                    entity: u32::from(entity_id),
                    ..Player::default()
                });
                index
            }
        };

        &mut self.players[index]
    }

    pub fn get_or_create_projectile<T: Copy>(
        &mut self,
        entity_id: T,
        class: ProjectileType
    ) 
        -> &mut Projectile where u32: From<T>
    {
        self.projectiles
            .entry(u32::from(entity_id))
            .or_insert_with(|| Projectile::new(entity_id.into(), class))
    }

    pub fn get_or_create_building<T: Copy>(
        &mut self,
        entity_id: T,
        class: BuildingClass,
    ) -> &mut Building where u32: From<T>, EntityId: From<T> {
        self.buildings
            .entry(u32::from(entity_id))
            .or_insert_with(|| Building::new(entity_id.into(), class))
    }

    pub fn remove_building<T: Copy>(&mut self, entity_id: T) where u32: From<T> {
        self.buildings.remove(&u32::from(entity_id));
    }


    pub fn get_or_create_medigun<T: Copy>(
        &mut self,
        entity_id: T
    ) -> &mut Medigun where u32: From<T> {
        self.mediguns
            .entry(u32::from(entity_id))
            .or_insert_with(|| Medigun::new(u32::from(entity_id)))
    }

    pub fn get_player_by_userid(&self, user_id: u16) -> Option<&Player> {
        self.players
            .iter()
            .filter(|p| if let Some(_) = p.info {true} else {false})
            .find(|p| p.info.as_ref().unwrap().user_id == user_id)
    }

    pub fn mut_player_by_userid(&mut self, user_id: u16) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .filter(|p| if let Some(_) = p.info {true} else {false})
            .find(|p| p.info.as_ref().unwrap().user_id == user_id)
    }

    pub fn get_player_by_entityid(&self, entity_id: u32) -> Option<&Player> {
        self.players
            .iter()
            .filter(|p| if let Some(_) = p.info {true} else {false})
            .find(|p| p.entity == entity_id)
    }

    pub fn mut_player_by_entityid(&mut self, entity_id: u32) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .filter(|p| if let Some(_) = p.info {true} else {false})
            .find(|p| p.entity == entity_id)
    }
}

#[pymethods]
impl TickData {
    #[getter]
    fn sentries(&self) -> HashMap<u32, Sentry> {
        let mut hash = HashMap::<u32, Sentry>::default();
        for (eid, bld) in &self.buildings {
            match bld {
                Building::Sentry(bld) => {hash.insert(u32::from(*eid), bld.clone());},
                _ => {}
            }
        }
        hash
    }

    #[getter]
    fn dispensers(&self) -> HashMap<u32, Dispenser> {
        let mut hash = HashMap::<u32, Dispenser>::default();
        for (eid, bld) in &self.buildings {
            match bld {
                Building::Dispenser(bld) => {hash.insert(u32::from(*eid), bld.clone());},
                _ => {}
            }
        }
        hash
    }

    #[getter]
    fn teleporters(&self) -> HashMap<u32, Teleporter> {
        let mut hash = HashMap::<u32, Teleporter>::default();
        for (eid, bld) in &self.buildings {
            match bld {
                Building::Teleporter(bld) => {hash.insert(u32::from(*eid), bld.clone());},
                _ => {}
            }
        }
        hash
    }

    #[getter]
    fn tick(&self) -> u32 {
        u32::from(self.tick)
    }

    #[getter]
    pub fn mediguns(&self) -> Vec<Medigun> {
        self.mediguns.values().cloned().collect_vec()
    }

    #[pyo3(name="get_player_by_entityid")]
    pub fn py_get_player_by_entityid(&self, entity_id: u32) -> Option<Player> {
        self.players.iter().find(|p| p.entity == entity_id).cloned()
    }

    #[pyo3(name="get_player_by_userid")]
    pub fn py_get_player_by_userid(&self, user_id: u16) -> Option<Player> {
        self.players
            .iter()
            .filter(|p| if let Some(_) = p.info {true} else {false})
            .find(|p| p.info.as_ref().unwrap().user_id == user_id).cloned()
    }
}

use std::path::PathBuf;

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct DemoData {
    // General information about the demo

    /// The path to the demofile that this data is for.
    pub demo_filename: PathBuf,

    /// The name of the map. Should end in .bsp.
    pub map_name: String,

    /// Duration of the recording, in seconds.
    pub duration: f32,

    /// Basic information for each round that occurred.
    pub rounds: Vec<Round>,
    
    /// Every single kill that happened in the game.
    pub kills: Vec<Kill>,

    pub point_captures: Vec<Capture>,

    /// TODO: Point captures, blocks/defends, ubers
    /// time spend on each class, etc.
    /// TODO: world

    /// The minimum and maximum X, Y, and Z values players ever had positions.
    pub player_reach_bounds: World,

    /// Tick data. Key: DemoTick as u32, Value: TickGameState
    pub tick_states: HashMap<u32, TickData>,
}

pub struct DemoDataSlice<'a> {
    pub demo_filename: &'a PathBuf,
    pub map_name: &'a String,
    pub duration: f32,
    pub rounds: Vec<&'a Round>,
    pub kills: Vec<&'a Kill>,
    pub point_captures: Vec<&'a Capture>,
    pub player_reach_bounds: World,
    pub tick_states: HashMap<u32, &'a TickData>,
}

impl DemoData {
    pub fn round_data<'a>(&'a self, round: &'a Round) -> DemoDataSlice<'a> {
        let tick_states = self.tick_states
            .iter()
            .filter(|(tick, _)| **tick >= round.start_tick && **tick <= round.end_tick)
            .map(|(tickr, stater)| (*tickr, stater))
            .collect::<HashMap<u32, &TickData>>();

        let kills = self.kills
            .iter()
            .filter(|kill| u32::from(kill.tick) >= round.start_tick && u32::from(kill.tick) <= round.end_tick)
            .collect();

        let point_captures = self.point_captures
            .iter()
            .filter(|cap| cap.tick >= round.start_tick && cap.tick <= round.end_tick)
            .collect();

        DemoDataSlice {
            demo_filename: &self.demo_filename,
            map_name: &self.map_name,
            duration: self.duration,
            rounds: vec![&round],
            kills,
            point_captures,
            player_reach_bounds: self.player_reach_bounds.clone(),
            tick_states
        }
    }
}

impl From<DemoDataSlice<'_>> for DemoData {
    fn from(value: DemoDataSlice<'_>) -> Self {
        DemoData {
            demo_filename: value.demo_filename.clone(),
            map_name: value.map_name.clone(),
            duration: value.duration,
            rounds: value.rounds.iter().map(|r| (*r).clone()).collect_vec(),
            kills: value.kills.iter().map(|r| (*r).clone()).collect_vec(),
            point_captures: value.point_captures.clone().into_iter().cloned().collect(),
            player_reach_bounds: value.player_reach_bounds,
            tick_states: value.tick_states.iter().map(|(t,s)| (*t, (*s).clone())).collect()
        }
    }
}

#[pymethods]
impl DemoData {
    /// View data that has been limited to a single round.
    /// This DOES copy the data in the parse, so be wary.
    #[pyo3(name = "round_data")]
    fn py_round_data(&self, round: &Round) -> DemoData {
        self.round_data(round).into()
    }
}

// Post-game summary of a player
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct PlayerSummary {
    pub kills: u32,
    pub assists: u32,
    pub deaths: u32,
    pub buildings_destroyed: u32,
    pub captures: u32,
    pub defenses: u32,
    //pub 
    pub user_id: u16,
}