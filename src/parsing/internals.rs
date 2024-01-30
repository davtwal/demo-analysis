//! A game state analyser for viewing the internals of each packet and
//! message.

use std::hash::Hash;
use std::thread::{self, JoinHandle};

use tf_demo_parser::demo::data::DemoTick;
use tf_demo_parser::demo::gamevent::GameEventType;
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::message::packetentities::PacketEntity;
use tf_demo_parser::demo::parser::handler::BorrowMessageHandler;
use tf_demo_parser::demo::parser::MessageHandler;

use tf_demo_parser::demo::packet::datatable::{ParseSendTable, ServerClass, ServerClassName};
use tf_demo_parser::demo::packet::stringtable::StringTableEntry;
use tf_demo_parser::{ParserState, DemoParser};

macro_rules! get_class_name {
    ($s:expr, $id:expr) => {
        $s.class_names
            .get(usize::from($id))
            .map(|class_name| class_name.as_str())
            .unwrap_or("")
    }
}

///////////
/// Threading Control Structures
#[derive(Default)]
pub enum InternalParseInstruction {
    #[default]
    ParseNext,
    ParseNextX(u32),
    ParseUntil(DemoTick),
    ParseUntilEnd,
    NextWithDataTable,
    NextWithStringEntry,
    NextWithPacketMeta,
    StopParse
}

#[derive(Default)]
pub enum InternalParseResult {
    Header(Header),
    Tick(InternalGameState),    // Requested tick was reached, here is gamestate
    #[allow(dead_code)]
    Error,                      // Some error occurred
    #[default]
    Done                        // Finished parsing
}

pub struct InternalParse {
    _handle: JoinHandle<()>,
    instruct_send: Sender<InternalParseInstruction>,
    result_recv: Receiver<InternalParseResult>
}

impl InternalParse {
    pub fn new(fpath: std::path::PathBuf) -> Self {
        println!("internal parse new");
        let (instruct_send, instruct_recv) = mpsc::channel::<InternalParseInstruction>();
        let (result_send, result_recv) = mpsc::channel::<InternalParseResult>();

        let _handle = thread::spawn(move || {
            println!("internal parse new thread");
            use InternalParseInstruction::*;
            use InternalParseResult::*;
            let file = std::fs::read(fpath.clone()).unwrap();
            let demo = tf_demo_parser::Demo::new(&file);

            let parser = DemoParser::new_all_with_analyser(
                demo.get_stream(),
                InternalsAnalyser::new()
            );

            let (header, mut ticker) = parser.ticker().unwrap();

            
            println!("internal parse thread: right before send, should go to loop");
            result_send.send(Header(header)).unwrap();

            'messageloop: loop {
                //println!("internal parse thread loop: tick {}", ticker.state().tick);
                // We want to wait for an instruction then do what it says
                match instruct_recv.recv().unwrap() {
                    ParseNext => {
                        if !ticker.tick().unwrap_or(false) {
                            break 'messageloop;
                        }
                    },
                    ParseNextX(count) => {
                        let until: DemoTick = ticker.state().tick + count;
                        while ticker.state().tick < until {
                            if !ticker.tick().unwrap_or(false) {
                                break 'messageloop;
                            }
                        }
                    },
                    ParseUntil(tick) => {
                        // If tick <= cur tick, do nothing
                        while ticker.state().tick < tick {
                            if !ticker.tick().unwrap_or(false) {
                                break 'messageloop;
                            }
                        }
                    },
                    ParseUntilEnd => {
                        while ticker.tick().unwrap_or(false) {}
                        result_send.send(Tick(ticker.state().clone())).unwrap();
                        break 'messageloop;
                    },
                    NextWithDataTable => {
                        loop {
                            if !ticker.tick().unwrap_or(false) {
                                break 'messageloop;
                            }

                            if ticker.state().last_data_table == ticker.state().tick {
                                break;
                            }
                        }
                        //result_send.send(Tick(ticker.state().clone())).unwrap();
                    },
                    NextWithStringEntry => {
                        loop {
                            if !ticker.tick().unwrap_or(false) {
                                break 'messageloop;
                            }

                            if ticker.state().last_string_entry == ticker.state().tick {
                                break;
                            }
                        }
                        //result_send.send(Tick(ticker.state().clone())).unwrap();
                    },
                    NextWithPacketMeta => {
                        loop {
                            if !ticker.tick().unwrap_or(false) {
                                break 'messageloop;
                            }

                            if ticker.state().last_packet_meta == ticker.state().tick {
                                break;
                            }
                        }
                        //result_send.send(Tick(ticker.state().clone())).unwrap();
                    }
                    StopParse => {
                        break 'messageloop;
                    }
                }

                if !ticker.state().tick_seen_table_entry.is_empty() {
                    println!("Seen Table Entries: {:?}", ticker.state().tick_seen_table_entry);
                }
                result_send.send(Tick(ticker.state().clone())).unwrap();
            }

            println!("Parse Internal Analysis Summary:");
            println!("-- Entity Types Seen: {:?}", ticker.state().all_seen_entity_types.keys());
            println!("-- Temp Entity Types: {:?}", ticker.state().all_seen_temp_entity_types.keys());
            println!("-- Game Events Seen: {:?}", ticker.state().all_seen_game_event_types.keys());
            
            println!("\n\nEVENT SEEN LIST:");
            println!("\\- ");

            result_send.send(InternalParseResult::Done).unwrap();
        });

        InternalParse {
            _handle,
            instruct_send,
            result_recv
        }
    }

    pub fn recv(&self) -> InternalParseResult {
        println!("recv");
        self.result_recv.recv().unwrap()
    }

    pub fn try_recv(&self) -> Option<InternalParseResult> {
        println!("try_recv");
        match self.result_recv.try_recv() {
            Ok(result) => {
                Some(result)
            },
            Err(e) => match e {
                mpsc::TryRecvError::Empty => {
                    None
                },
                mpsc::TryRecvError::Disconnected => {
                    // This is fine, as the last message sent by the thread worker will be ::Done
                    panic!("Attempted try_recv on a disconnected channel (you forgot to drop the worker");
                }
            }
        }
    }

    pub fn send(&self, instruction: InternalParseInstruction) {
        println!("send");
        self.instruct_send.send(instruction).unwrap()
    }
}

use std::sync::mpsc;
pub use std::sync::mpsc::{Sender, Receiver};
use std::collections::{HashSet, HashMap};

// Actual analyser

#[derive(Default, Debug, Clone)]
pub struct InternalGameState {
    pub last_data_table: DemoTick,
    pub last_header: DemoTick,
    pub last_message: DemoTick,
    pub last_packet_meta: DemoTick,
    pub last_string_entry: DemoTick,
    pub tick_seen_entities: Vec<String>,
    pub tick_seen_table_entry: HashSet<String>,

    pub all_seen_entity_types: HashMap<String, Vec<DemoTick>>,
    pub all_seen_table_entries: HashMap<String, Vec<DemoTick>>,
    pub all_seen_game_event_types: HashMap<GameEventType, Vec<DemoTick>>,
    pub all_seen_temp_entity_types: HashMap<String, Vec<DemoTick>>,
    
    pub tick: DemoTick,
}

#[derive(Default, Debug)]
pub struct InternalsAnalyser {
    pub state: InternalGameState,
    tick: DemoTick,
    class_names: Vec<ServerClassName>,
}

/*
Current seen types I may care about:
Tick 334 in demofile.dem is when the game starts
Tick 413 first rocket jump
MESSAGES:
    - GameEventList
    - SetView
    - NetTick
    - SignOnState (state:Spawn ?)
    - PacketEntities (contains a list of entities)
    - TempEntities (?)
    - GameEvent
    - EntityMessage ???

TABLES SEEN:
    - downloadables
    - modelprecache
    - genericprecache
    - soundprecache
    - decalprecache
    - instancebaseline
    - lightstyles
    - userinfo -> BIG ONE
    - server_query_info
    - ParticleEffectNames
    - VguiScreen
    - Materials
    - InfoPanel
    - Scenes
    - ServerMapCycle
    - ServerMapCycleMvM
    - GameRulesCreation

GAME EVENTS SEEN:
    - RocketJump
    - PlayerHurt
    - RocketJumpLanded
    - ScoreStatsAccumulatedUpdate
    - ScoreStatsAccumulatedReset
    - TeamPlayMapTimeRemaining
    - PlayerRegenerate
    - PlayerHealed
    - StatsResetRound
    - RecalculateHolidays
    - PostInventoryApplication
    - PlayerSpawn
    - TeamPlayRestartRound
    - RemoveNemesisRelationships
    - DemomanDetStickies
    - StickyJump

NOTABLE ENTITY CLASS NAMES SEEN:
    - CTFPipebombLauncher
    - CTFRocketLauncher
    - CTFGrenadePipebombProjectile  // Probably stickies
    - CTFProjectile_Rocket
    - CTFGrenadeLauncher
    - CWeaponMedigun
    - CTFScatterGun
    - CTFBat
    - A bunch of other weapon types - why?
    - CTFPistol_ScoutSecondary
    - CTFDroppedWeapon
    - CTFAmmoPack
    = CTFPistol_Scout ???
    - CTFGameRulesProxy
    - CTeamRoundTimer
    - CTFCrossbow

TEMP ENTITIES SEEN:
    - CTETFBlood
    - CTETFExplosion
    - CTEFireBullets
*/

impl MessageHandler for InternalsAnalyser {
    type Output = InternalGameState;

    fn does_handle(_message_type: tf_demo_parser::MessageType) -> bool {
        true
    }

    fn handle_data_tables(
            &mut self,
            _tables: &[ParseSendTable],
            server_classes: &[ServerClass],
            _parser_state: &ParserState,
    ) {
        self.state.last_data_table = self.tick;
        //println!("### HANDLE DATA TABLES ({}) ###", u32::from(self.tick));
        //println!("|- TABLES: {:?}", tables);
        //println!("\\- SERVER CLASSES: {:?}", server_classes);
        self.class_names = server_classes
            .iter()
            .map(|class| &class.name)
            .cloned()
            .collect();
    }

    fn handle_header(&mut self, _header: &tf_demo_parser::demo::header::Header) {
        self.state.last_header = self.tick;
        //println!("### HEADER ({}) ###", u32::from(self.tick));
        //println!("{:?}", header);
    }

    fn handle_message(
        &mut self, 
        message: &Message<'_>, 
        _tick: DemoTick, 
        parser_state: &ParserState
    ) {
        
        self.state.last_message = self.tick;
        //println!("$$ {:?} ({:?}) [{:?}] $$",
        //    message.get_message_type(), u32::from(self.tick), u32::from(tick));
        
        match message {
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity, parser_state)
                }
            }

            Message::TempEntities(message) => {
            //    println!("$ Temp Entity $");
                //println!("\\- Events: {:?}", message.events);
                for event in &message.events {
                    let class_name: &str = get_class_name!(self, event.class_id);
                    update(&mut self.state.all_seen_temp_entity_types, class_name.to_string(), self.tick);
             //       println!(" \\- {:?}", class_name);
                }
            }

            Message::GameEvent(message) => {
                
                update(&mut self.state.all_seen_game_event_types, message.event.event_type(), self.tick);
                //println!("\\- Type: {:?}", message.event.event_type());
            }
            _ => {}
        }
        //println!("{:?}", message);

    }

    fn handle_packet_meta(
            &mut self,
            tick: DemoTick,
            _meta: &tf_demo_parser::demo::packet::message::MessagePacketMeta,
            _parser_state: &ParserState,
    ) {
        println!("$$$$ NEW PACKET: Tick #{:?} $$$$", u32::from(self.tick));
        self.tick = tick;
        self.state.tick = tick;
        self.state.last_packet_meta = tick;
        self.state.tick_seen_entities.clear();
        self.state.tick_seen_table_entry.clear();
        //println!("\\- Meta: {:?}", meta);
    }

    fn handle_string_entry(
            &mut self,
            table: &str,
            _index: usize,
            _entries: &StringTableEntry<'_>,
            _parser_state: &ParserState,
    ) {
        self.state.last_string_entry = self.tick;
        update(&mut self.state.all_seen_table_entries, table.to_string(), self.tick);
        if !self.state.tick_seen_table_entry.contains(table) {
            //println!("## New Table Usage ({:?}) ##", u32::from(self.tick));
            //println!("\\- Table: {:?}", table);
            self.state.tick_seen_table_entry.insert(table.to_string());
        }
        //println!("\\- Entries: {:?}", entries);
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }
}

impl BorrowMessageHandler for InternalsAnalyser {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.state
    }
}

fn update<T: Hash + Eq, U>(map: &mut HashMap<T, Vec<U>>, k: T, addval: U) {
    if map.contains_key(&k) {
        map.get_mut(&k).unwrap().push(addval);
    }
    else {
        map.insert(k, vec![addval]);
    }
}

impl InternalsAnalyser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let class_name: &str = get_class_name!(self, entity.server_class);

        update(&mut self.state.all_seen_entity_types, class_name.to_string(), self.tick);

        //if self.state.all_seen_entity_types.contains_key(class_name) {
        //    self.state.all_seen_entity_types.
        //}

        //self.state.all_seen_entity_types.iter_mut().filter(|(k,v)| {/
//
  //      }).collect();
        self.state.tick_seen_entities.push(class_name.to_string());
        //println!("$ Entity {} ({}): {:?} $", entity.entity_index, class_name, entity.update_type);
    
        match class_name {
            "CTFProjectile_Flare"
            | "CTFProjectile_HealingBolt"
            | "CTFProjectile_Rocket"
            | "CTFProjectile_SentryRocket"
            | "CTFGrenadePipebombProjectile"
            => self.handle_projectile_entity(entity, parser_state),

            "CTFPlayerResource" => self.handle_player_resource(entity, parser_state),
            _ => self.handle_unknown(entity, parser_state)
        }
    }

    pub fn handle_player_resource(&mut self, _entity: &PacketEntity, _parser_state: &ParserState) {

    }

    pub fn handle_unknown(&mut self, entity: &PacketEntity, _parser_state: &ParserState) {
        let class_name: &str = get_class_name!(self, entity.server_class);
        println!("Unknown: {:#?}", class_name)
    }

    pub fn handle_projectile_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let class_name: &str = get_class_name!(self, entity.server_class);
        println!("PROJECTILE {:?} ({:} @ {})", entity.update_type, class_name, entity.entity_index);
        println!("Props: {:?}", entity.props);
    }
}