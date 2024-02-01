
use std::collections::HashMap;

use itertools::Itertools;
use pyo3::prelude::*;

use crate::types::game::entities::ProjectileType;

use super::{DemoTick, EntityId};
use super::game::entities::{Player, Sentry, Dispenser, Teleporter, Building, Projectile};
use super::game::events::Kill;
use super::game::Round;

#[pyclass]
#[derive(Default, Debug, Clone)]
pub struct TickData {
    // Formerly gamestate internals
    #[pyo3(get)]
    pub players: Vec<Player>,
    pub projectiles: HashMap<u32, Projectile>,
    pub buildings: HashMap<u32, Building>,
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
        class: super::game::entities::BuildingClass,
    ) -> &mut Building where u32: From<T>, EntityId: From<T> {
        self.buildings
            .entry(u32::from(entity_id))
            .or_insert_with(|| Building::new(entity_id.into(), class))
    }

    pub fn remove_building<T: Copy>(&mut self, entity_id: T) where u32: From<T> {
        self.buildings.remove(&u32::from(entity_id));
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

    fn get_player_by_entityid(&self, entity_id: u32) -> Option<Player> {
        self.players.iter().find(|p| p.entity == entity_id).cloned()
    }

    fn get_player_by_userid(&self, user_id: u16) -> Option<Player> {
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

    /// TODO: Point captures, blocks/defends, ubers
    /// time spend on each class, etc.
    /// TODO: world

    /// Tick data. Key: DemoTick as u32, Value: TickGameState
    pub tick_states: HashMap<u32, TickData>,
}

pub struct DemoDataSlice<'a> {
    pub demo_filename: &'a PathBuf,
    pub map_name: &'a String,
    pub duration: f32,
    pub rounds: Vec<&'a Round>,
    pub kills: Vec<&'a Kill>,
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

        DemoDataSlice {
            demo_filename: &self.demo_filename,
            map_name: &self.map_name,
            duration: self.duration,
            rounds: vec![&round],
            kills,
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