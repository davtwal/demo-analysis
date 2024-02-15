use itertools::Itertools;
/// entities.rs
/// 
/// Essentially re-creates the types from tf_demo_parser's analysers
/// in a way that they can be interfaced through python. Most of this code
/// is basically copied.

use pyo3::prelude::*;

use num_enum::{TryFromPrimitive, IntoPrimitive};

use super::{EntityId, UserId};
use super::math::Vector;

/// This function is used by lib.rs, but the IDE thinks it's unused.
#[allow(dead_code)]
pub(crate) fn get_submod<'a>(
    py: Python<'a>,
) -> PyResult<&'a PyModule> {
    let module = PyModule::new(py, "entities")?;
    module.add_class::<UserInfo>()?;
    module.add_class::<PlayerState>()?;
    module.add_class::<Player>()?;
    module.add_class::<Sentry>()?;
    module.add_class::<Dispenser>()?;
    module.add_class::<Teleporter>()?;
    module.add_class::<Medigun>()?;
    Ok(module)
}

/////////////////////////////////////////////
/// PLAYER
/// /////////////////////////////////////////

use super::game::{Class, Team, ClassList};

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
#[derive(Debug, Clone, Default)]
pub struct Player {
    pub(crate) entity: u32,
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
    pub time_since_last_hurt: f32,

    pub class_info: Option<ClassInfo>,

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
    #[getter]
    fn scout_info(&self) -> Option<ScoutInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Scout(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn soldier_info(&self) -> Option<SoldierInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Soldier(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn pyro_info(&self) -> Option<PyroInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Pyro(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn demoman_info(&self) -> Option<DemomanInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Demoman(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn heavy_info(&self) -> Option<HeavyInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Heavy(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn engineer_info(&self) -> Option<EngineerInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Engineer(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn medic_info(&self) -> Option<MedicInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Medic(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn sniper_info(&self) -> Option<SniperInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Sniper(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

    #[getter]
    fn spy_info(&self) -> Option<SpyInfo> {
        match &self.class_info {
            Some(inf) => match inf {
                ClassInfo::Spy(inf) => Some(inf.clone()),
                _ => None
            }, None => None
        }
    }

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

    pub fn critheal_percent(&self) -> f32 {
        ((self.time_since_last_hurt - 10.0) / 5.0).clamp(0.0, 1.0)
    }
}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Medigun {
    pub entity_id: u32,
    pub owner: u32,
    pub charge: f32,
    pub heal_target: u32,
    pub is_healing: bool,
    pub is_holstered: bool,
}

impl Medigun {
    pub fn new(entity_id: u32) -> Self {
        Medigun {
            entity_id: entity_id,
            ..Default::default()
        }
    }
}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct ScoutInfo {}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct SoldierInfo{}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct PyroInfo{}

#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct DemomanInfo{}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct HeavyInfo{}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct EngineerInfo{}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct MedicInfo {
    pub is_healing: bool,
    pub heal_target: u32,
    pub last_heal_target: u32
}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct SniperInfo{}
#[pyclass(get_all)]
#[derive(Default, Debug, Clone)]
pub struct SpyInfo{}

#[derive(Debug, Clone)]
pub enum ClassInfo {
    Scout(ScoutInfo),
    Soldier(SoldierInfo),
    Pyro(PyroInfo),
    Demoman(DemomanInfo),
    Heavy(HeavyInfo),
    Engineer(EngineerInfo),
    Medic(MedicInfo),
    Sniper(SniperInfo),
    Spy(SpyInfo),
}

impl ClassInfo {
    pub fn from(class: Class) -> Result<Self, &'static str> {
        use Class::*;
        match class {
            Scout => Ok(ClassInfo::Scout(ScoutInfo::default())),
            Soldier => Ok(ClassInfo::Soldier(SoldierInfo::default())),
            Pyro => Ok(ClassInfo::Pyro(PyroInfo::default())),
            Demoman => Ok(ClassInfo::Demoman(DemomanInfo::default())),
            Heavy => Ok(ClassInfo::Heavy(HeavyInfo::default())),
            Engineer => Ok(ClassInfo::Engineer(EngineerInfo::default())),
            Medic => Ok(ClassInfo::Medic(MedicInfo::default())),
            Sniper => Ok(ClassInfo::Sniper(SniperInfo::default())),
            Spy => Ok(ClassInfo::Spy(SpyInfo::default())),
            _ => Err("no")
        }
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
    #[allow(dead_code)]
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


