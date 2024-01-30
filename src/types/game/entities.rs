use itertools::Itertools;
/// entities.rs
/// 
/// Essentially re-creates the types from tf_demo_parser's analysers
/// in a way that they can be interfaced through python. Most of this code
/// is basically copied.

use pyo3::prelude::*;

use num_enum::{TryFromPrimitive, IntoPrimitive};

use super::super::{EntityId, UserId};
use super::super::math::Vector;

// this function is used in game::mod.rs but rust_analyzer thinks not
#[allow(dead_code)]
pub fn register_with(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    let ent_mod = PyModule::new(py, "entities")?;
    
    module.add_class::<World>()?;
    module.add_class::<UserInfo>()?;
    module.add_class::<PlayerState>()?;
    ent_mod.add_class::<Player>()?;
    ent_mod.add_class::<Sentry>()?;
    ent_mod.add_class::<Dispenser>()?;
    ent_mod.add_class::<Teleporter>()?;

    module.add_submodule(ent_mod)?;
    Ok(())
}



/////////////////////////////////////////////
/// WORLD
/// /////////////////////////////////////////

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
/// PLAYER
/// /////////////////////////////////////////

use super::{Class, Team, ClassList};

use tf_demo_parser::demo::parser::analyser::UserInfo as TFUinf;
//use tf_demo_parser::demo::parser::gamestateanalyser::{Player as TFPlayer};

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub classes: ClassList,
    pub name: String,
    pub user_id: u16,
    pub steam_id: String,
    pub entity_id: u32,
    pub team: Team,
}

// #[pymethods]
// impl UserInfo {
//     #[getter]
//     fn user_id(&self) -> u16 {
//         u16::from(self.user_id)
//     }

//     #[getter]
//     fn entity_id(&self) -> u32 {
//         u32::from(self.entity_id)
//     }
// }

impl From<TFUinf> for UserInfo {
    fn from(value: TFUinf) -> Self {
        UserInfo {
            classes: ClassList::from(value.classes),
            name: value.name,
            user_id: u16::from(value.user_id),
            steam_id: value.steam_id,
            entity_id: u32::from(value.entity_id),
            team: Team::from(value.team),
        }
    }
}

impl From<tf_demo_parser::demo::data::UserInfo> for UserInfo {
    fn from(info: tf_demo_parser::demo::data::UserInfo) -> Self {
        UserInfo {
            classes: ClassList::default(),
            name: info.player_info.name,
            user_id: u16::from(info.player_info.user_id),
            steam_id: info.player_info.steam_id,
            entity_id: u32::from(info.entity_id),
            team: Team::default(),
        }
    }
}

impl PartialEq for UserInfo {
    fn eq(&self, other: &UserInfo) -> bool {
        self.classes == other.classes
            && self.name == other.name
            && self.user_id == other.user_id
            && self.steam_id == other.steam_id
            && self.team == other.team
    }
}

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, Default)]
#[repr(u8)]
pub enum PlayerState {
    #[default]
    Alive = 0,
    Dying = 1,
    Death = 2,
    Respawnable = 3,
}

#[pymethods]
impl PlayerState {
    #[new]
    pub fn new(number: u8) -> Self {
        match number {
            1 => PlayerState::Dying,
            2 => PlayerState::Death,
            3 => PlayerState::Respawnable,
            _ => PlayerState::Alive,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Player {
    pub(in super::super) entity: u32,
    #[pyo3(get)]
    pub position: Vector,
    #[pyo3(get)]
    pub health: u16,
    #[pyo3(get)]
    pub max_health: u16,
    #[pyo3(get, name="player_class")]
    pub class: Class,
    #[pyo3(get)]
    pub team: Team,
    #[pyo3(get)]
    pub view_angle: f32,
    #[pyo3(get)]
    pub pitch_angle: f32,
    #[pyo3(get)]
    pub state: PlayerState,
    #[pyo3(get)]
    pub info: Option<UserInfo>,
    #[pyo3(get)]
    pub charge: u8,
    #[pyo3(get)]
    pub simtime: u16,
    #[pyo3(get)]
    pub ping: u16,
    #[pyo3(get)]
    pub in_pvs: bool,
}

use ordered_float::OrderedFloat;

impl Player {
    pub fn closest_to_xy<'a>(&self, player_list: Vec<&'a Player>) -> &'a Player {
        player_list.iter().min_by_key(|p| {
            if self.entity == p.entity {
                OrderedFloat(f32::MAX)
            } else {
                OrderedFloat(self.distance_from_xy(p))
            }
        }).unwrap()
    }
}

#[pymethods]
impl Player {
    #[pyo3(name="closest_to")]
    pub fn py_closest_to_xy(&self, player_list: Vec<Player>) -> Player {
        self.closest_to_xy(player_list.iter().map(|p| p).collect_vec()).clone()
    }

    pub fn distance_from(&self, other: &Player) -> f32 {
        self.position.dist_to(&other.position)
    }

    pub fn distance_from_xy(&self, other: &Player) -> f32 {
        self.position.xy().dist_to(&other.position.xy())
    }

    pub fn height_diff(&self, other: &Player) -> f32 {
        self.position.z - other.position.z
    }

    pub fn is_alive(&self) -> bool {
        self.state == PlayerState::Alive
    }
}

/////////////////////////////////////////////
/// BUILDINGS
/// /////////////////////////////////////////

#[pyclass(get_all)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sentry {
    pub entity: u32, // eid
    pub builder: u16, //uid
    pub position: Vector,
    pub level: u8,
    pub max_health: u16,
    pub health: u16,
    pub building: bool,
    pub sapped: bool,
    pub team: Team,
    pub angle: f32,
    pub player_controlled: bool,
    pub auto_aim_target: u16, //uid
    pub shells: u16,
    pub rockets: u16,
    pub is_mini: bool,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dispenser {
    pub entity: u32,        // entity id
    pub builder: u16,       // user id
    pub position: Vector,
    pub level: u8,
    pub max_health: u16,
    pub health: u16,
    pub building: bool,
    pub sapped: bool,
    pub team: Team,
    pub angle: f32,
    pub healing: Vec<u16>,  // user ids
    pub metal: u16,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Teleporter {
    pub entity: u32,        // entity id
    pub builder: u16,       // user id
    pub position: Vector,
    pub level: u8,
    pub max_health: u16,
    pub health: u16,
    pub building: bool,
    pub sapped: bool,
    pub team: Team,
    pub angle: f32,
    pub is_entrance: bool,
    pub other_end: u32,     // entity id
    pub recharge_time: f32,
    pub recharge_duration: f32,
    pub times_used: u16,
    pub yaw_to_exit: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Building {
    Sentry(Sentry),
    Dispenser(Dispenser),
    Teleporter(Teleporter),
}

impl Building {
    pub fn new(entity_id: EntityId, class: BuildingClass) -> Building {
        match class {
            BuildingClass::Sentry => Building::Sentry(Sentry {
                entity: u32::from(entity_id),
                ..Sentry::default()
            }),
            BuildingClass::Dispenser => Building::Dispenser(Dispenser {
                entity: u32::from(entity_id),
                ..Dispenser::default()
            }),
            BuildingClass::Teleporter => Building::Teleporter(Teleporter {
                entity: u32::from(entity_id),
                ..Teleporter::default()
            }),
        }
    }

    pub fn entity_id(&self) -> EntityId {
        match self {
            Building::Sentry(Sentry { entity, .. })
            | Building::Dispenser(Dispenser { entity, .. })
            | Building::Teleporter(Teleporter { entity, .. }) => EntityId::from(*entity),
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Building::Sentry(Sentry { level, .. })
            | Building::Dispenser(Dispenser { level, .. })
            | Building::Teleporter(Teleporter { level, .. }) => *level,
        }
    }

    pub fn position(&self) -> Vector {
        match self {
            Building::Sentry(Sentry { position, .. })
            | Building::Dispenser(Dispenser { position, .. })
            | Building::Teleporter(Teleporter { position, .. }) => *position,
        }
    }

    pub fn builder(&self) -> UserId {
        match self {
            Building::Sentry(Sentry { builder, .. })
            | Building::Dispenser(Dispenser { builder, .. })
            | Building::Teleporter(Teleporter { builder, .. }) => UserId::from(*builder),
        }
    }

    pub fn angle(&self) -> f32 {
        match self {
            Building::Sentry(Sentry { angle, .. })
            | Building::Dispenser(Dispenser { angle, .. })
            | Building::Teleporter(Teleporter { angle, .. }) => *angle,
        }
    }

    pub fn max_health(&self) -> u16 {
        match self {
            Building::Sentry(Sentry { max_health, .. })
            | Building::Dispenser(Dispenser { max_health, .. })
            | Building::Teleporter(Teleporter { max_health, .. }) => *max_health,
        }
    }

    pub fn health(&self) -> u16 {
        match self {
            Building::Sentry(Sentry { health, .. })
            | Building::Dispenser(Dispenser { health, .. })
            | Building::Teleporter(Teleporter { health, .. }) => *health,
        }
    }

    pub fn sapped(&self) -> bool {
        match self {
            Building::Sentry(Sentry { sapped, .. })
            | Building::Dispenser(Dispenser { sapped, .. })
            | Building::Teleporter(Teleporter { sapped, .. }) => *sapped,
        }
    }

    pub fn team(&self) -> Team {
        match self {
            Building::Sentry(Sentry { team, .. })
            | Building::Dispenser(Dispenser { team, .. })
            | Building::Teleporter(Teleporter { team, .. }) => *team,
        }
    }

    pub fn class(&self) -> BuildingClass {
        match self {
            Building::Sentry(_) => BuildingClass::Sentry,
            Building::Dispenser(_) => BuildingClass::Sentry,
            Building::Teleporter(_) => BuildingClass::Teleporter,
        }
    }
}

pub enum BuildingClass {
    Sentry,
    Dispenser,
    Teleporter,
}

/////////////////////////////////////////////
/// PROJECTILES
/// /////////////////////////////////////////

// TODO
#[pyclass]
#[derive(Default, Copy, Clone, Debug)]
pub enum ProjectileType {
    #[default]
    Unknown,
    Rocket,
    GrenadePipe,
    StickyBomb,
    CrossbowBolt,
    HuntsmanArrow,
    BallOrnament,
    Flare,
    BisonBolt,
}

// SendProps for:
/*
    CBaseProjectile:
        - m_hOriginalLauncher probably don't care
        CTFBaseRocket:
            - m_vInitialVelocty
            - m_vecOrigin

 */
#[pyclass]
#[derive(Default, Copy, Clone, Debug)]
pub struct Projectile {
    pub(in super::super) entity: u32,
    pub shooter: u16,
    pub projectile_type: ProjectileType,
    pub position: Vector,
}

impl Projectile {
    pub fn new(entity_id: u32, class: ProjectileType) -> Self {
        Projectile {
            entity: entity_id,
            projectile_type: class,
            ..Default::default()
        }
    }
}


