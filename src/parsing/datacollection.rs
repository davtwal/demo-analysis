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

use clap::parser;
use tf_demo_parser::demo::message::tempentities::EventInfo;
use std::convert::TryFrom;
use std::str::FromStr;

use tf_demo_parser::demo::gameevent_gen::ObjectDestroyedEvent;
use tf_demo_parser::demo::gamevent::GameEvent;
use tf_demo_parser::demo::message::packetentities::{PacketEntity, UpdateType};
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::packet::datatable::{ParseSendTable, ServerClass, ServerClassName};
use tf_demo_parser::demo::packet::message::MessagePacketMeta;
use tf_demo_parser::demo::packet::stringtable::StringTableEntry;
use tf_demo_parser::demo::parser::handler::BorrowMessageHandler;
use tf_demo_parser::demo::parser::MessageHandler;
use tf_demo_parser::demo::sendprop::{SendProp, SendPropIdentifier, SendPropValue};

use tf_demo_parser::{MessageType, ParserState, ReadResult, Stream};

use tf_demo_parser::demo::vector::{Vector as TFVec, VectorXY as TFVecXY};

use crate::types::{DemoTick, EntityId};
use crate::types::math::{Vector, VectorXY};
use crate::types::game::{Round, Team, Class};
use crate::types::game::events::Kill;
use crate::types::game::entities::*;

//use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy)]
pub struct ProjectileUpdate {
    pub entityid: EntityId,
    pub update_type: UpdateType
}

impl Default for ProjectileUpdate {
    fn default() -> Self {
        ProjectileUpdate {
            entityid: EntityId::default(),
            update_type: UpdateType::Preserve
        }
    }
}

use crate::types::demo::TickData;

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

#[derive(Default, Debug)]
pub struct GameStateAnalyserPlus {
    pub state: TickGameState,
    tick: DemoTick,
    class_names: Vec<ServerClassName>,
}


impl MessageHandler for GameStateAnalyserPlus {
    type Output = TickGameState;

    // Legit just copied
    fn does_handle(message_type: MessageType) -> bool {
        matches!(
            message_type,
            MessageType::PacketEntities | MessageType::GameEvent | MessageType::TempEntities
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


    // Legit just copied
    fn handle_packet_meta(
        &mut self,
        tick: DemoTick,
        _meta: &MessagePacketMeta,
        _parser_state: &ParserState,
    ) {
        // self.state.messages_this_tick.clear();
        // self.state.events_this_tick.clear();
        // self.state.projectile_update_this_tick.clear();
        // self.state.temp_entity_messages.clear();
        self.state.data.tick = tick;
        self.state.kills.clear();
        self.tick = tick;
    }


    // Legit just copied
    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }


    // This is where the new stuff is
    fn handle_message(&mut self, message: &Message<'_>, tick: DemoTick, parser_state: &ParserState) {    
        //self.state.messages_this_tick.push(message.get_message_type());

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

/*fn why_is_kill_private(tick: DemoTick, death: &PlayerDeathEvent) -> Kill {
    Kill {
        attacker_id: death.attacker,
        assister_id: death.assister,
        victim_id: death.user_id,
        weapon: death.weapon.to_string(),
        tick,
    }
}*/

use crate::types::game::entities::ProjectileType;

impl GameStateAnalyserPlus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_temp_entity(&mut self, _events: &Vec<EventInfo>) {
        //self.state.temp_entity_messages.push(events.clone());
    }

    pub fn handle_event(&mut self, event: &GameEvent, tick: DemoTick) {
        const WIN_REASON_TIME_LIMIT: u8 = 6;

        //self.state.events_this_tick.push(event.event_type());

        match event {
            GameEvent::PlayerDeath(event) => {
                self.state.kills.push(Kill::from_event(tick, event));
            }
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
            GameEvent::ObjectDestroyed(ObjectDestroyedEvent{index, ..}) => {
                self.state.data.remove_building(*index as u32);
            }
            _ => {}
        }
    }

    // All of the below is copied from gamestateanalyser.

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
            "CTFPlayer" => self.handle_player_entity(entity, parser_state),
            "CTFPlayerResource" => self.handle_player_resource(entity, parser_state),
            "CWorld" => self.handle_world_entity(entity, parser_state),
            "CObjectSentrygun" => self.handle_sentry_entity(entity, parser_state),
            "CObjectDispenser" => self.handle_dispenser_entity(entity, parser_state),
            "CObjectTeleporter" => self.handle_teleporter_entity(entity, parser_state),
            // Projectiles

            "CTFProjectile_Rocket" => self.handle_rocket(entity, parser_state, ProjectileType::Rocket),
            _ => {
                //println!("[{}] Handling Entity: {:?}{:?}", self.tick, entity.update_type, class_name);
            }
        }
    }

    pub fn handle_rocket(&mut self, entity: &PacketEntity, parser_state: &ParserState, projectile_type: ProjectileType) {
        let projectile = self.state.data.get_or_create_projectile(entity.entity_index, projectile_type);

        const ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFBaseRocket", "m_vecOrigin");
        
        const INITIAL_VELOCITY : SendPropIdentifier =
            SendPropIdentifier::new("DT_TFBaseRocket", "m_vInitialVelocity");

        const ANG_ROTATION : SendPropIdentifier =
            SendPropIdentifier::new("DT_TFBaseRocket", "m_vInitialVelocity");

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
                                    Class::new(i64::try_from(&prop.value).unwrap_or_default())
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

        const HEALTH_PROP: SendPropIdentifier =
            SendPropIdentifier::new("DT_BasePlayer", "m_iHealth");
        const MAX_HEALTH_PROP: SendPropIdentifier =
            SendPropIdentifier::new("DT_BasePlayer", "m_iMaxHealth");
        const LIFE_STATE_PROP: SendPropIdentifier =
            SendPropIdentifier::new("DT_BasePlayer", "m_lifeState");

        const LOCAL_ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_vecOrigin");
        const NON_LOCAL_ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin");
        const LOCAL_ORIGIN_Z: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_vecOrigin[2]");
        const NON_LOCAL_ORIGIN_Z: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin[2]");
        const LOCAL_EYE_ANGLES: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[1]");
        const NON_LOCAL_EYE_ANGLES: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
        const LOCAL_PITCH_ANGLES: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[0]");
        const NON_LOCAL_PITCH_ANGLES: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[0]");
            
        const SIMTIME_PROP: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseEntity", "m_flSimulationTime");

        player.in_pvs = entity.in_pvs;

        for prop in entity.props(parser_state) {
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
                _ => {}
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
        const ANGLE: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
        const MINI: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseObject", "m_bMiniBuilding");
        const CONTROLLED: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectSentrygun", "m_bPlayerControlled");
        const TARGET: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectSentrygun", "m_hAutoAimTarget");
        const SHELLS: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectSentrygun", "m_iAmmoShells");
        const ROCKETS: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectSentrygun", "m_iAmmoRockets");

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
        const RECHARGE_TIME: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectTeleporter", "m_flRechargeTime");
        const RECHARGE_DURATION: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectTeleporter", "m_flCurrentRechargeDuration");
        const TIMES_USED: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectTeleporter", "m_iTimesUsed");
        const OTHER_END: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectTeleporter", "m_bMatchBuilding");
        const YAW_TO_EXIT: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectTeleporter", "m_flYawToExit");
        const IS_ENTRANCE: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseObject", "m_iObjectMode");

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
        const AMMO: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectDispenser", "m_iAmmoMetal");
        const HEALING: SendPropIdentifier =
            SendPropIdentifier::new("DT_ObjectDispenser", "healing_array");

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

        const LOCAL_ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseEntity", "m_vecOrigin");
        const TEAM: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");
        const ANGLE: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_angRotation");
        const SAPPED: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_bHasSapper");
        const BUILDING: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseObject", "m_bBuilding");
        const LEVEL: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseObject", "m_iUpgradeLevel");
        const BUILDER: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_hBuilder");
        const MAX_HEALTH: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseObject", "m_iMaxHealth");
        const HEALTH: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_iHealth");

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
