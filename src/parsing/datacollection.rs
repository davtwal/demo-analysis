//////////////////////////////
//! Author: David Walker
//! Date:   11.30.23
//! Purpose:    
//!     Contains the state and custom analyser used to collect the data about
//!     the game state. This doesn't do any analysis; this is the data collection
//! 
//! Usage:
//!  - Create a [`GameStateAnalyserPlus`] and hand it to a 
//!     [`DemoParser`](tf_demo_parser::demo::parser::DemoParser).
//! 
//! Structure:
//!  -  
//! 
//!  -  [`TickGameState`] contains all relevant information about a game tick,
//!     including player positioning.
//! 
//////////////////////////////

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use tf_demo_parser::{
    MessageType, ParserState, ReadResult, Stream,
    demo::{
        gamevent::GameEvent,

        gameevent_gen::ObjectDestroyedEvent,

        message::{
            Message,
            tempentities::EventInfo,
            packetentities::{
                PacketEntity,
                UpdateType,
            },
        },

        packet::{
            datatable::{
                ParseSendTable,
                ServerClass,
                ServerClassName,
            },

            message::MessagePacketMeta,
            stringtable::StringTableEntry,
        },

        parser::{
            handler::BorrowMessageHandler,
            MessageHandler,
        },

        sendprop::{
            SendProp,
            SendPropIdentifier as SPI,
            SendPropValue,
        },

        vector::{
            Vector as TFVec,
            VectorXY as TFVecXY,
        }
    }
};

use crate::types::DemoTick;
use crate::types::math::{Vector, VectorXY};
use crate::types::game::{World, Round, Team, Class};
use crate::types::events::{Capture, Kill, Ubercharge};
use crate::types::entities::*;
use crate::types::demo::TickData;

//use serde::{Serialize, Deserialize};

///////////////////////////////////////////////////
///////////////////////////////////////////////////
///////////////////////////////////////////////////
/// TICK GAME STATE
/// ///////////////////////////////////////////////

#[derive(Default, Debug, Clone)]
pub struct TickGameState {
    // Formerly gamestate internals
    // Kept track of on the regular.
    pub data: TickData,
    pub rounds: Vec<Round>,
    pub world: Option<World>,
    
    pub(crate) _cur_round_index: u32,

    // Past this, these are cleared at the start of each packet.
    // The parser should be collecting these as "new information"
    // every tick.
    pub kills: Vec<Kill>,
    pub captures: Vec<Capture>,
    pub ubercharges: Vec<Ubercharge>,
    pub players_hit: Vec<u32>, // by entity_id
}

impl TickGameState {
    pub fn start_round(&mut self, tick: DemoTick) {
        self.rounds.push(Round {
            start_tick: u32::from(tick),
            end_tick: 0,
            winner: Team::Other
        })
    }

    pub fn end_round(&mut self, tick: DemoTick, winner: Team) {
        if let Some(last) = &mut self.rounds.last_mut() {
            last.end_tick = u32::from(tick);
            last.winner = winner;
        }
    }
}

///////////////////////////////////////////////////////
/// GAME STATE ANALYSER: DEFINTION
/// ///////////////////////////////////////////////////
/// ///////////////////////////////////////////////////
/// ///////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct HandleMap {
    // used to convert handles (such as m_hOwner) to entity ids
    handle_to_ent: HashMap<u32, u32>,

    // weapon entity id -> hOwner
    // to get weapon owner identity do
    // outer_map.get(weapon_map.get(entity_id))
    entity_owners: HashMap<u32, u32>,
}

impl HandleMap {
    /// entity_id: The ID of the entity being registered.
    /// entity_handle: The handle (via m_hOuter) of the entity being registered.
    pub fn register_entity_handle(&mut self, entity_id: u32, entity_handle: u32) {
        self.handle_to_ent.insert(entity_handle, entity_id);
    }

    pub fn register_entity_owner(&mut self, entity_id: u32, owner_handle: u32) {
        self.entity_owners.insert(entity_id, owner_handle);
    }

    pub fn get_entity_owner_id(&self, entity_id: u32) -> Option<u32> {
        match self.entity_owners.get(&entity_id) {
            Some(owner_handle) => self.handle_to_ent.get(owner_handle).copied(),
            None => None
        }
    }

    pub fn get_entity_id(&self, entity_handle: u32) -> Option<u32> {
        self.handle_to_ent.get(&entity_handle).copied()
    }

    pub fn handle_prop<'a>(&mut self, entity: &PacketEntity, prop: &'a SendProp) -> Option<&'a SendProp> {
        match prop.identifier {
            SPI_BASEENTITY_OWNER
            | SPI_BASECOMBATWEP_OWNER
            => {
                self.register_entity_owner(
                    u32::from(entity.entity_index), 
                    get_prop_int_u32(&prop.value)
                );
            },

            SPI_ATTR_CONT_HOUTER
            | SPI_ATTR_MNGR_HOUTER
            | SPI_ATTR_LIST_HOUTER
            => {
                self.register_entity_handle(
                    u32::from(entity.entity_index), 
                    get_prop_int_u32(&prop.value)
                );
            }

            _ => return Some(prop)
        }
        None
    }
}

#[derive(Default, Debug)]
pub struct GameStateAnalyserPlus {
    pub state: TickGameState,
    tick: DemoTick,
    class_names: Vec<ServerClassName>,

    handle_map: HandleMap,
}

// The goal for message handler is to get out of message handler into impl as
// quickly as possible for organization purposes.
impl MessageHandler for GameStateAnalyserPlus {
    type Output = TickGameState;

    fn does_handle(message_type: MessageType) -> bool {
        matches!(
            message_type,
            MessageType::PacketEntities     // Packet of entities!
            | MessageType::GameEvent        // All the events!
            | MessageType::TempEntities     // Not actually handled right now
        )
    }

    // Legit just copied
    fn handle_string_entry(
        &mut self,
        table: &str,
        index: usize,
        entry: &StringTableEntry<'_>,
        _parser_state: &ParserState,
    ) {
        if table == "userinfo" {
            let _ = self.parse_user_info(
                index,
                entry.text.as_ref().map(|s| s.as_ref()),
                entry.extra_data.as_ref().map(|data| data.data.clone()),
            );
        }
    }

    // Legit just copied
    fn handle_data_tables(
        &mut self,
        _parse_tables: &[ParseSendTable],
        server_classes: &[ServerClass],
        _parser_state: &ParserState,
    ) {
        self.class_names = server_classes
            .iter()
            .map(|class| &class.name)
            .cloned()
            .collect();
    }


    fn handle_packet_meta(
        &mut self,
        tick: DemoTick,
        meta: &MessagePacketMeta,
        parser_state: &ParserState,
    ) {
        self.handle_tick_end(tick, parser_state);
        self.handle_tick_start(tick, meta, parser_state);
    }


    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }

    fn handle_message(&mut self, message: &Message<'_>, tick: DemoTick, parser_state: &ParserState) {    
        match message {
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity, parser_state);
                }
            }

            Message::TempEntities(message) => self.handle_temp_entity(&message.events),

            Message::GameEvent(message) => self.handle_event(&message.event, tick),
            _ => {}
        }
    }
}

const CLASSNAME_PLAYER: &str = "CTFPlayer";
const CLASSNAME_PLAYER_RESOURCE: &str = "CTFPlayerResource";
const CLASSNAME_WEP_MEDIGUN: &str = "CWeaponMedigun";
const CLASSNAME_SENTRY: &str = "CObjectSentrygun";
const CLASSNAME_DISPENSER: &str = "CObjectDispenser";
const CLASSNAME_TELEPORTER: &str = "CObjectTeleporter";

const SPI_BASEENTITY_OWNER: SPI = SPI::new("DT_BaseEntity", "m_hOwnerEntity");
const SPI_BASECOMBATWEP_OWNER: SPI = SPI::new("DT_BaseCombatWeapon", "m_hOwner");
const SPI_ATTR_MNGR_HOUTER: SPI = SPI::new("DT_AttributeManager", "m_hOuter");
const SPI_ATTR_CONT_HOUTER: SPI = SPI::new("DT_AttributeContainer", "m_hOuter");
const SPI_ATTR_LIST_HOUTER: SPI = SPI::new("DT_AttributeList", "m_hOuter");

const SPI_MEDIGUN_HEALTARG: SPI = SPI::new("DT_WeaponMedigun", "m_hHealingTarget");
const SPI_MEDIGUN_HEALING: SPI = SPI::new("DT_WeaponMedigun", "m_bHealing");
const SPI_MEDIGUN_HOLSTERED: SPI = SPI::new("DT_WeaponMedigun", "m_bHolstered");
const SPI_MEDIGUN_CHARGE_LOCAL: SPI = SPI::new("DT_LocalTFWeaponMedigunData", "m_flChargeLevel");
const SPI_MEDIGUN_CHARGE_NONLOCAL: SPI = SPI::new("DT_TFWeaponMedigunDataNonLocal", "m_flChargeLevel");

//const SPI_ROCKET_ORIGIN: SPI = SPI::new("DT_TFBaseRocket", "m_vecOrigin");
//const SPI_ROCKET_INIT_VEL : SPI = SPI::new("DT_TFBaseRocket", "m_vInitialVelocity");
//const SPI_ROCKET_ANG_ROTATION : SPI = SPI::new("DT_TFBaseRocket", "m_vInitialVelocity");

fn get_prop_int_u32(prop: &SendPropValue) -> u32 {
    i64::try_from(prop).unwrap_or_default() as u32
}

fn get_prop_bool(prop: &SendPropValue) -> bool {
    i64::try_from(prop).unwrap_or_default() != 0
}

impl GameStateAnalyserPlus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_tick_end(&mut self, _next_tick: DemoTick, _parser_state: &ParserState) {
        // noop atm
    }

    pub fn handle_tick_start(&mut self,
        tick: DemoTick,
        _meta: &MessagePacketMeta,
        parser_state: &ParserState,
    ) {
        for player in &mut self.state.data.players {
            player.time_since_last_hurt += parser_state.demo_meta.interval_per_tick;
        }

        self.state.data.tick = tick;
        self.state.data.tick_delta = parser_state.demo_meta.interval_per_tick;
        self.tick = tick;

        // Clear data that is captured per-tick
        self.state.kills.clear();
        self.state.captures.clear();
        self.state.ubercharges.clear();
    }

    pub fn handle_temp_entity(&mut self, _events: &Vec<EventInfo>) {
    }

    ////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////
    /// EVENT HANDLING
    ////////////////////////////////////////////////////////////////////////////

    pub fn handle_event(&mut self, event: &GameEvent, tick: DemoTick) {
        const WIN_REASON_TIME_LIMIT: u8 = 6;

        match event {
            // Round/point related
            GameEvent::RoundStart(_) => {
                self.state.data.buildings.clear();
            }
            GameEvent::TeamPlayRoundStart(_) => {
                self.state.data.buildings.clear();
                self.state.start_round(self.tick);
            }
            GameEvent::TeamPlayRoundWin(event) => {
                let victor: Team = if event.win_reason != WIN_REASON_TIME_LIMIT {
                    Team::new(event.team)
                } else {
                    Team::Other
                };
                self.state.end_round(self.tick, victor)
            }
            GameEvent::TeamPlayRoundStalemate(_) => {
                self.state.end_round(self.tick, Team::Other)
            }

            GameEvent::TeamPlayPointCaptured(event) => {
                self.state.captures.push(Capture::from_event(tick, &event));
            }

            // Player related
            GameEvent::PlayerDeath(event) => {
                self.state.kills.push(Kill::from_event(tick, event));
            }
            
            GameEvent::PlayerHurt(event) => {
                if let Some(player) = self.state.data.mut_player_by_userid(event.user_id) {
                    player.time_since_last_hurt = 0.0;
                }
            },
            GameEvent::PlayerHealed(event) => {
                // get players
                if let Some(patient) = self.state.data.get_player_by_userid(event.patient) {
                    if let Some(healer) = self.state.data.get_player_by_userid(event.healer) {
                        if patient.team != healer.team || healer.class != Class::Medic {
                            println!("NOTE: weird player healed: pat/heal team {:?} {:?}, heal class {:?}",
                                    patient.team, healer.team, healer.class);
                        }
                        else {
                            
                        }
                    }
                }
                else {
                    println!("player healed with invalid patient");
                }
            }
            GameEvent::PlayerChargeDeployed(event) => {
                self.state.ubercharges.push(Ubercharge::from_event(tick, event))
            }


            // Object / building
            GameEvent::ObjectDestroyed(ObjectDestroyedEvent{index, ..}) => {
                self.state.data.remove_building(*index as u32);
            },

            _ => {}
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////
    /// ENTITY HANDLING
    ////////////////////////////////////////////////////////////////////////////
    /*
    PROJECTILES FROM WEAPONS TO CARE ABOUT IN SIXES:
	- tf_projectile_arrow : Huntsman arrows
	- tf_projectile_ball_ornament : Wrap assassin ball
	- tf_projectile_energy_ring : Righteous Bison bolt
	- tf_projectile_flare : Flare bolt
	- tf_projectile_healing_bolt : Crossbow bolt
	- tf_projectile_pipe : Grenade
	- tf_projectile_pipe_remote : Stickies
	- tf_projectile_rocket : Rocket from soldier. ~110 hammer units/sec? */

    pub fn handle_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let class_name: &str = self
            .class_names
            .get(usize::from(entity.server_class))
            .map(|class_name| class_name.as_str())
            .unwrap_or("");

        match class_name {
            CLASSNAME_PLAYER => self.handle_player_entity(entity, parser_state),
            CLASSNAME_PLAYER_RESOURCE => self.handle_player_resource(entity, parser_state),
            "CWorld" => self.handle_world_entity(entity, parser_state),
            CLASSNAME_SENTRY => self.handle_sentry_entity(entity, parser_state),
            CLASSNAME_DISPENSER => self.handle_dispenser_entity(entity, parser_state),
            CLASSNAME_TELEPORTER => self.handle_teleporter_entity(entity, parser_state),
            
            // Weapons
            "CTFScattergun" 
            => self.handle_weapon(entity, parser_state, Class::Scout),

            "CTFRocketLauncher"
            | "CTFRocketLauncher_AirStrike"
            | "CTFRocketLauncher_DirectHit"
            => self.handle_weapon(entity, parser_state, Class::Soldier),

            "CTFFlamethrower"
            => self.handle_weapon(entity, parser_state, Class::Pyro),

            "CTFGrenadeLauncher"
            | "CTFPipebombLauncher"
            => self.handle_weapon(entity, parser_state, Class::Demoman),

            "CTFMinigun"
            => self.handle_weapon(entity, parser_state, Class::Heavy),

            "CTFWrench"
            => self.handle_weapon(entity, parser_state, Class::Engineer),

            CLASSNAME_WEP_MEDIGUN => self.handle_medic_weapon(entity, parser_state),

            "CTFSniperRifle"
            | "CTFCompoundBow"
            => self.handle_weapon(entity, parser_state, Class::Sniper),

            "CTFRevolver"
            => self.handle_weapon(entity, parser_state, Class::Spy),

            // Projectiles
            "CTFProjectile_Rocket" => self.handle_rocket(entity, parser_state, ProjectileType::Rocket),
            _ => {
                //println!("[{}] Handling Entity: {:?}{:?}", self.tick, entity.update_type, class_name);
            }
        }
    }

    pub fn handle_medic_weapon(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let medigun = self.state.data.get_or_create_medigun(entity.entity_index);

        let mut new_target_handle: Option<u32> = None;
        let mut new_charge: Option<f32> = None;
        let mut heal_change: Option<bool> = None;
        let mut holster_change: Option<bool> = None;
        for prop in entity.props(parser_state) {
            if let Some(prop) = self.handle_map.handle_prop(entity, &prop) {
                match prop.identifier {
                    SPI_MEDIGUN_HEALTARG => {
                        new_target_handle = Some(get_prop_int_u32(&prop.value));
                    },

                    SPI_MEDIGUN_CHARGE_LOCAL
                    | SPI_MEDIGUN_CHARGE_NONLOCAL => {
                        new_charge = Some(f32::try_from(&prop.value).unwrap_or_default());
                    },

                    SPI_MEDIGUN_HEALING => {
                        heal_change = Some(get_prop_bool(&prop.value));
                    }

                    SPI_MEDIGUN_HOLSTERED => {
                        holster_change = Some(get_prop_bool(&prop.value));
                    }
                    _ => {}
                }
            }
        }

        // must do the get stuff after prop parse as the owner/etc could have been set during
        // the prop parse
        if let Some(medic_id) = self.handle_map.get_entity_owner_id(u32::from(entity.entity_index)) {
            medigun.owner = medic_id;
            if let Some(target_handle) = new_target_handle {
                if let Some(target_id) = self.handle_map.get_entity_id(target_handle) {
                    medigun.heal_target = target_id;
                }
            }

            if let Some(charge) = new_charge {
                medigun.charge = charge;
            }

            if let Some(is_healing) = heal_change {
                medigun.is_healing = is_healing;
            }

            if let Some(is_holstered) = holster_change {
                medigun.is_holstered = is_holstered;
            }
            
        }
    }

    pub fn handle_weapon(&mut self, entity: &PacketEntity, parser_state: &ParserState, _class: Class) {
        if entity.update_type == UpdateType::Enter {
            for _prop in entity.props(parser_state) {
                
            }
        }
    }

    pub fn handle_rocket(&mut self, entity: &PacketEntity, parser_state: &ParserState, projectile_type: ProjectileType) {
        self.handle_projectile(entity, parser_state);
        
        let _projectile = self.state.data.get_or_create_projectile(entity.entity_index, projectile_type);

        //const H_

        

        //const 
    }

    pub fn handle_projectile(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        //if entity.update_type == UpdateType::Enter {return;}

        //self.state.projectile_update_this_tick.push(entity.clone())
        //let projectile = self.state.data.get_or_create_projectile(entity.entity_index);

        

        //projectile.in_pvs = entity.in_pvs;

        for prop in entity.props(parser_state) {
            match prop.identifier {
                _ => {}
            }
        }
    }

    pub fn handle_player_resource(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        for prop in entity.props(parser_state) {
            if let Some((table_name, prop_name)) = prop.identifier.names() {
                if let Ok(player_id) = u32::from_str(prop_name.as_str()) {
                    let entity_id = player_id;
                    if let Some(player) = self
                        .state.data
                        .players
                        .iter_mut()
                        .find(|player| player.info.as_ref().expect("player had no info").entity_id == entity_id)
                        // Had to make a slight change to the above as player.entity is private;
                        // This means I had to go into the info first.
                    {
                        match table_name.as_str() {
                            "m_iTeam" => {
                                player.team =
                                    Team::new(i64::try_from(&prop.value).unwrap_or_default())
                            }
                            "m_iMaxHealth" => {
                                player.max_health =
                                    i64::try_from(&prop.value).unwrap_or_default() as u16
                            }
                            "m_iPlayerClass" => {
                                player.class =
                                    Class::new(i64::try_from(&prop.value).unwrap_or_default());
                                println!("updated player {} class to {:?}", player.info.as_ref().unwrap().user_id, player.class);
                                player.class_info = ClassInfo::from(player.class).ok();
                            }
                            "m_iChargeLevel" => {
                                player.charge = i64::try_from(&prop.value).unwrap_or_default() as u8
                            }
                            "m_iPing" => {
                                player.ping = i64::try_from(&prop.value).unwrap_or_default() as u16
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub fn handle_player_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let player = self.state.data.get_or_create_player(entity.entity_index);

        const HEALTH_PROP: SPI = SPI::new("DT_BasePlayer", "m_iHealth");
        const MAX_HEALTH_PROP: SPI = SPI::new("DT_BasePlayer", "m_iMaxHealth");
        const LIFE_STATE_PROP: SPI =
        SPI::new("DT_BasePlayer", "m_lifeState");

        const LOCAL_ORIGIN: SPI =
        SPI::new("DT_TFLocalPlayerExclusive", "m_vecOrigin");
        const NON_LOCAL_ORIGIN: SPI =
        SPI::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin");
        const LOCAL_ORIGIN_Z: SPI =
        SPI::new("DT_TFLocalPlayerExclusive", "m_vecOrigin[2]");
        const NON_LOCAL_ORIGIN_Z: SPI =
        SPI::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin[2]");
        const LOCAL_EYE_ANGLES: SPI =
        SPI::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[1]");
        const NON_LOCAL_EYE_ANGLES: SPI =
        SPI::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
        const LOCAL_PITCH_ANGLES: SPI =
        SPI::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[0]");
        const NON_LOCAL_PITCH_ANGLES: SPI =
        SPI::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[0]");
            
        const SIMTIME_PROP: SPI =
        SPI::new("DT_BaseEntity", "m_flSimulationTime");

        player.in_pvs = entity.in_pvs;

        for prop in entity.props(parser_state) {
            if let Some(prop) = self.handle_map.handle_prop(entity, &prop) {
                match prop.identifier {
                    HEALTH_PROP => {
                        player.health = i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    MAX_HEALTH_PROP => {
                        player.max_health = i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    LIFE_STATE_PROP => {
                        player.state = PlayerState::new(i64::try_from(&prop.value).unwrap_or_default() as u8)
                    }
                    LOCAL_ORIGIN | NON_LOCAL_ORIGIN => {
                        let pos_xy = VectorXY::from(
                            TFVecXY::try_from(&prop.value).unwrap_or_default()
                        );
                        player.position.x = pos_xy.x;
                        player.position.y = pos_xy.y;
                    }
                    LOCAL_ORIGIN_Z | NON_LOCAL_ORIGIN_Z => {
                        player.position.z = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    LOCAL_EYE_ANGLES | NON_LOCAL_EYE_ANGLES => {
                        player.view_angle = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    LOCAL_PITCH_ANGLES | NON_LOCAL_PITCH_ANGLES => {
                        player.pitch_angle = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    SIMTIME_PROP => {
                        player.simtime = i64::try_from(&prop.value).unwrap_or_default() as u16
                    }

                    // Player summary stats:

                    _ => {}
                }
            }
        }
    }

    pub fn handle_world_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        if let (
            Some(SendProp {
                value: SendPropValue::Vector(boundary_min),
                ..
            }),
            Some(SendProp {
                value: SendPropValue::Vector(boundary_max),
                ..
            }),
        ) = (
            entity.get_prop_by_name("DT_WORLD", "m_WorldMins", parser_state),
            entity.get_prop_by_name("DT_WORLD", "m_WorldMaxs", parser_state),
        ) {
            self.state.world = Some(World {
                bound_min: Vector::from(boundary_min),
                bound_max: Vector::from(boundary_max),
            })
        }
    }

    pub fn handle_sentry_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const ANGLE: SPI = SPI::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
        const MINI: SPI = SPI::new("DT_BaseObject", "m_bMiniBuilding");
        const CONTROLLED: SPI = SPI::new("DT_ObjectSentrygun", "m_bPlayerControlled");
        const TARGET: SPI = SPI::new("DT_ObjectSentrygun", "m_hAutoAimTarget");
        const SHELLS: SPI = SPI::new("DT_ObjectSentrygun", "m_iAmmoShells");
        const ROCKETS: SPI = SPI::new("DT_ObjectSentrygun", "m_iAmmoRockets");

        if entity.update_type == UpdateType::Delete {
            self.state.data.remove_building(entity.entity_index);
            return;
        }

        self.handle_building(entity, parser_state, BuildingClass::Sentry);

        let building = self
            .state.data
            .get_or_create_building(entity.entity_index, BuildingClass::Sentry);

        if let Building::Sentry(sentry) = building {
            for prop in entity.props(parser_state) {
                match prop.identifier {
                    ANGLE => sentry.angle = f32::try_from(&prop.value).unwrap_or_default(),
                    MINI => sentry.is_mini = i64::try_from(&prop.value).unwrap_or_default() > 0,
                    CONTROLLED => {
                        sentry.player_controlled =
                            i64::try_from(&prop.value).unwrap_or_default() > 0
                    }
                    TARGET => {
                        sentry.auto_aim_target =
                            i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    SHELLS => sentry.shells = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    ROCKETS => {
                        sentry.rockets = i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn handle_teleporter_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const RECHARGE_TIME: SPI = SPI::new("DT_ObjectTeleporter", "m_flRechargeTime");
        const RECHARGE_DURATION: SPI = SPI::new("DT_ObjectTeleporter", "m_flCurrentRechargeDuration");
        const TIMES_USED: SPI = SPI::new("DT_ObjectTeleporter", "m_iTimesUsed");
        const OTHER_END: SPI = SPI::new("DT_ObjectTeleporter", "m_bMatchBuilding");
        const YAW_TO_EXIT: SPI = SPI::new("DT_ObjectTeleporter", "m_flYawToExit");
        const IS_ENTRANCE: SPI = SPI::new("DT_BaseObject", "m_iObjectMode");

        if entity.update_type == UpdateType::Delete {
            self.state.data.remove_building(entity.entity_index);
            return;
        }

        self.handle_building(entity, parser_state, BuildingClass::Teleporter);

        let building = self
            .state.data
            .get_or_create_building(entity.entity_index, BuildingClass::Teleporter);

        if let Building::Teleporter(teleporter) = building {
            for prop in entity.props(parser_state) {
                match prop.identifier {
                    RECHARGE_TIME => {
                        teleporter.recharge_time = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    RECHARGE_DURATION => {
                        teleporter.recharge_duration =
                            f32::try_from(&prop.value).unwrap_or_default()
                    }
                    TIMES_USED => {
                        teleporter.times_used =
                            i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    OTHER_END => {
                        teleporter.other_end =
                            i64::try_from(&prop.value).unwrap_or_default() as u32
                    }
                    YAW_TO_EXIT => {
                        teleporter.yaw_to_exit = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    IS_ENTRANCE => {
                        teleporter.is_entrance = i64::try_from(&prop.value).unwrap_or_default() == 0
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn handle_dispenser_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const AMMO: SPI = SPI::new("DT_ObjectDispenser", "m_iAmmoMetal");
        const HEALING: SPI = SPI::new("DT_ObjectDispenser", "healing_array");

        if entity.update_type == UpdateType::Delete {
            self.state.data.remove_building(entity.entity_index);
            return;
        }

        self.handle_building(entity, parser_state, BuildingClass::Dispenser);

        let building = self
            .state.data
            .get_or_create_building(entity.entity_index, BuildingClass::Dispenser);

        if let Building::Dispenser(dispenser) = building {
            for prop in entity.props(parser_state) {
                match prop.identifier {
                    AMMO => dispenser.metal = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    HEALING => {
                        let values = match &prop.value {
                            SendPropValue::Array(vec) => vec.as_slice(),
                            _ => Default::default(),
                        };

                        dispenser.healing = values
                            .iter()
                            .map(|val| i64::try_from(val).unwrap_or_default() as u16)
                            .collect()
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_building(
        &mut self,
        entity: &PacketEntity,
        parser_state: &ParserState,
        class: BuildingClass,
    ) {
        let building = self
            .state.data
            .get_or_create_building(entity.entity_index, class);

        const LOCAL_ORIGIN: SPI = SPI::new("DT_BaseEntity", "m_vecOrigin");
        const TEAM: SPI = SPI::new("DT_BaseEntity", "m_iTeamNum");
        const ANGLE: SPI = SPI::new("DT_BaseEntity", "m_angRotation");
        const SAPPED: SPI = SPI::new("DT_BaseObject", "m_bHasSapper");
        const BUILDING: SPI = SPI::new("DT_BaseObject", "m_bBuilding");
        const LEVEL: SPI = SPI::new("DT_BaseObject", "m_iUpgradeLevel");
        const BUILDER: SPI = SPI::new("DT_BaseObject", "m_hBuilder");
        const MAX_HEALTH: SPI = SPI::new("DT_BaseObject", "m_iMaxHealth");
        const HEALTH: SPI = SPI::new("DT_BaseObject", "m_iHealth");

        match building {
            Building::Sentry(Sentry {
                position,
                team,
                angle,
                sapped,
                builder,
                level,
                building,
                max_health,
                health,
                ..
            })
            | Building::Dispenser(Dispenser {
                position,
                team,
                angle,
                sapped,
                builder,
                level,
                building,
                max_health,
                health,
                ..
            })
            | Building::Teleporter(Teleporter {
                position,
                team,
                angle,
                sapped,
                builder,
                level,
                building,
                max_health,
                health,
                ..
            }) => {
                for prop in entity.props(parser_state) {
                    match prop.identifier {
                        LOCAL_ORIGIN => {
                            *position = Vector::from(TFVec::try_from(&prop.value).unwrap_or_default())
                        }
                        TEAM => *team = Team::new(i64::try_from(&prop.value).unwrap_or_default()),
                        ANGLE => *angle = f32::try_from(&prop.value).unwrap_or_default(),
                        SAPPED => *sapped = i64::try_from(&prop.value).unwrap_or_default() > 0,
                        BUILDING => *building = i64::try_from(&prop.value).unwrap_or_default() > 0,
                        LEVEL => *level = i64::try_from(&prop.value).unwrap_or_default() as u8,
                        BUILDER => {
                            *builder =
                                i64::try_from(&prop.value).unwrap_or_default() as u16
                        }
                        MAX_HEALTH => {
                            *max_health = i64::try_from(&prop.value).unwrap_or_default() as u16
                        }
                        HEALTH => *health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                        _ => {}
                    }
                }
            }
        }
    }

    fn parse_user_info(
        &mut self,
        index: usize,
        text: Option<&str>,
        data: Option<Stream>,
    ) -> ReadResult<()> {
        if let Some(user_info) =
            tf_demo_parser::demo::data::UserInfo::parse_from_string_table(index as u16, text, data)?
        {
            let id = user_info.entity_id;
            self.state.data.get_or_create_player(id).info = Some(user_info.into());
        }

        Ok(())
    }
}

impl BorrowMessageHandler for GameStateAnalyserPlus {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.state
    }
}
