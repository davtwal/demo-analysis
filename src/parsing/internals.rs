//! A game state analyser for viewing the internals of each packet and
//! message.

use std::hash::Hash;
use std::thread::{self, JoinHandle};

//use itertools::{Itertools, Update};
use tf_demo_parser::demo::data::DemoTick;
use tf_demo_parser::demo::gamevent::{GameEvent, GameEventType};
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::message::packetentities::{PacketEntity, UpdateType};
use tf_demo_parser::demo::parser::handler::BorrowMessageHandler;
use tf_demo_parser::demo::parser::MessageHandler;

use tf_demo_parser::demo::packet::datatable::{ParseSendTable, ServerClass, ServerClassName};
use tf_demo_parser::demo::packet::stringtable::StringTableEntry;
use tf_demo_parser::demo::sendprop::SendPropIdentifier;
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
/// 
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum InternalParseResult {
    Header(Header),
    Tick(InternalGameState),    // Requested tick was reached, here is gamestate
    #[allow(dead_code)]
    Error,                      // Some error occurred
    #[default]
    Done                        // Finished parsing
}

#[allow(dead_code)]
pub struct InternalParse {
    _handle: JoinHandle<()>,
    instruct_send: Sender<InternalParseInstruction>,
    result_recv: Receiver<InternalParseResult>
}

#[allow(dead_code)]
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

////////////////
/// Gatherer

/// Gathers every single message, data table, etc.
/// This

pub enum GatherSeen {
    Header(tf_demo_parser::demo::header::Header),
    _DataTables(Vec<tf_demo_parser::demo::packet::datatable::SendTableName>),
    StringEntry(String, usize, StringTableEntry<'static>),
    PacketMeta(u32),
}

#[derive(Default)]
pub struct GathererState {
    pub seen: Vec<GatherSeen>,
    pub seen_string_table_names: HashSet<String>,
    pub seen_message_types: HashMap<u32, Vec<tf_demo_parser::MessageType>>,
    pub seen_packet_entities_types: HashSet<String>,
    pub seen_game_event_types: HashSet<GameEventType>,

    pub seen_wearables: HashSet<u32>,
    pub seen_player_entids: HashSet<u32>,
    pub seen_ent_handles: HashSet<u32>,

    pub interesting_entities: HashMap<String, HashSet<(String, String)>>,
    pub interesting_events: HashMap<u32, Vec<GameEventType>>,
    pub interesting_datatable_entries: Vec<ParseSendTable>,

    pub outer_map: HashMap<u32, u32>,
}

#[derive(Default)]
pub struct Gatherer {
    pub state: GathererState, 
    class_names: Vec<ServerClassName>,
    match_attempts: Vec<MatchAttempt>,
    
}

use std::io::{self, Read, Write};

use crate::types::game::{Class, Team};
#[allow(dead_code)]
fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

impl MessageHandler for Gatherer {
    type Output = GathererState;

    fn does_handle(_message_type: tf_demo_parser::MessageType) -> bool {
        true
    }

    fn handle_data_tables(
            &mut self,
            tables: &[ParseSendTable],
            server_classes: &[ServerClass],
            _parser_state: &ParserState,
        ) {

        let tables = Vec::from(tables);

        for table in tables {
            match table.name.as_str() {
                "m_iOwner" 
                | "m_iHealing"
                | "m_hMyWeapons"
                | "_LPT_m_hMyWearables_8"
                | "_ST_m_hMyWearables_8"
                | "DT_WeaponMedigun"
                => {
                    self.state.interesting_datatable_entries.push(table.clone());
                }
                _ => {}
            }
        }

        self.class_names = server_classes
            .iter()
            .map(|class| &class.name)
            .cloned()
            .collect();
    }

    fn handle_header(&mut self, header: &Header) {
        self.state.seen.push(GatherSeen::Header(header.clone()));
    }

    fn handle_message(&mut self, message: &Message, tick: DemoTick, _parser_state: &ParserState) {
        self.state.seen_message_types
            .entry(u32::from(tick))
            .or_insert(Vec::new())
            .push(message.get_message_type());

        // Interesting message types:
        match message {
            Message::ClassInfo(m) => {
                println!("---- ({}) Class info spotted; it's private though", tick);
                println!("- {:?}", m);
            },
            Message::ServerInfo(m) => {
                println!("---- ({}) Server info spotted; it's private though", tick);
                println!("- {:?}", m);
            }
            Message::SetConVar(_) => {
                println!("--- SetConVar spotted at {} ", tick);
            }
            // Message::GameEventList(m) => {
            //     println!("---- ({}) Game Event List: ", tick);
            //     for gedef in &m.event_list {
            //         println!("-- {:?} : {:?}", gedef.event_type, gedef.id);
            //         println!("- Entries: {:?}", gedef.entries.iter().map(|item| &item.name).collect::<Vec<&String>>())
            //     }
            // }
            Message::PacketEntities(m) => {
                for ent in &m.entities {
                    self.handle_entity(ent, tick, _parser_state);

                    // let class_name: &str = self
                    //     .class_names
                    //     .get(usize::from(ent.server_class))
                    //     .map(|class_name| class_name.as_str())
                    //     .unwrap_or("");

                    // self.state.seen_packet_entities_types.insert(class_name.to_string());

                    // const BENT_TEAMNUM: SendPropIdentifier = 
                    //     SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");

                    // const BENT_HOWNER: SendPropIdentifier =
                    //     SendPropIdentifier::new("DT_BaseEntity", "m_hOwnerEntity");


                    // match class_name {
                    //     "CTFPlayer" => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             println!(":: Player enter, entity ID: {}, serial number: {}", ent.entity_index, ent.serial_number);
                    //         }
                    //         self.state.seen_player_entids.insert(u32::from(ent.entity_index));
                    //     },

                    //     "CTFRocketLauncher"
                    //     | "CTFRocketLauncher_AirStrike"
                    //     | "CTFRocketLauncher_DirectHit" 
                    //     // not _mortar as it's unused
                    //     // 
                    //     => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("rocket launch enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFScatterGun" => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("scatter enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFFlamethrower" => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("flamethrower enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFGrenadeLauncher" 
                    //     | "CTFPipebombLauncher"
                    //     => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("grenade/pipe launch enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFWrench"
                    //     => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("wrench enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFSniperRifle"
                    //     | "CTFCompoundBow"
                    //     => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("snip/compound enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     "CTFRevolver"
                    //     => {
                    //         if ent.update_type == UpdateType::Enter {
                    //             let mut howner = 0;
                    //             let mut team = 10;
                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BENT_HOWNER => {
                    //                         howner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                         self.state.seen_ent_handles.insert(howner);
                    //                         //println!("rocket launch enter w/ hOwner: {}", hand);
                    //                     },
                    //                     BENT_TEAMNUM => {
                    //                         team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     }
                    //                     _ => {}
                    //                 }
                    //             }

                    //             println!("revolver enter: hown {} team {}", howner, team);
                    //         }
                    //     }

                    //     // "CSniperDot" 
                    //     // | "CTFGrenadePipebombProjectile"
                    //     // "CTFPlayer"
                    //     // => {
                    //     //     const 
                    //     // },
                    //     // | "CTFPlayerResource"
                    //     // | "CTFProjectile_Rocket"
                    //     // | "CTFProjectile_HealingBolt"
                    //     // | "CTFProjectile_SentryRocket"
                    //     // | "CTFTeam"
                    //     // | "CWeaponMedigun"
                    //     // => {
                    //     //     for prop in ent.props(_parser_state) {
                    //     //         let propident = prop.identifier.names().unwrap_or((prop.identifier.to_string().into(), "uhoh".into()));

                    //     //         self.state.interesting_entities
                    //     //             .entry(class_name.to_string())
                    //     //             .or_default()
                    //     //             .insert((propident.0.to_string(), propident.1.to_string()));
                    //     //     }
                    //     // }
                    //     "CWeaponMedigun" => {
                    //         const BENT_HOWNER: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_BaseEntity", "m_hOwnerEntity");
                    //         const BENT_MOVEPARENT: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_BaseEntity", "moveparent");

                    //         // CBaseCombatWeapon
                    //         const BCOMBWEP_HOWNER: SendPropIdentifier = 
                    //             SendPropIdentifier::new("DT_BaseCombatWeapon", "m_hOwner");
                    //         const BCOMBWEP_STATE: SendPropIdentifier = 
                    //             SendPropIdentifier::new("DT_BaseCombatWeapon", "m_iState");

                    //         // CWeaponMedigun
                    //         const WEP_MEDIGUN_BHEALING: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_WeaponMedigun", "m_bHealing");
                    //         const WEP_MEDIGUN_HHEALTARGET: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_WeaponMedigun", "m_hHealingTarget");
                    //         const WEP_MEDIGUN_HLASTHEALTARGET: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_WeaponMedigun", "m_hLastHealingTarget");
                    //         const WEP_MEDIGUN_BHOLSTERED: SendPropIdentifier =
                    //             SendPropIdentifier::new("DT_WeaponMedigun", "m_bHolstered");

                    //         #[derive(Default, Debug)]
                    //         struct MedigunParse {
                    //             pub bent_owner: Option<u32>,
                    //             pub bent_moveparent: Option<String>,
                    //             pub bcomw_owner: Option<u32>,
                    //             pub bcomw_state: Option<u32>,
                    //             pub wmed_healing: Option<bool>,
                    //             pub wmed_holstered: Option<bool>,
                    //             pub wmed_healtarg: Option<u32>,
                    //             pub wmed_lasthealtarg: Option<u32>,
                    //         }

                    //         impl MedigunParse {
                    //             pub fn has(&self) -> bool {
                    //                 self.bent_owner.is_some()
                    //                 || self.bent_moveparent.is_some()
                    //                 || self.bcomw_owner.is_some()
                    //                 || self.bcomw_state.is_some()
                    //                 || self.wmed_healing.is_some()
                    //                 || self.wmed_holstered.is_some()
                    //                 || self.wmed_healtarg.is_some()
                    //                 || self.wmed_lasthealtarg.is_some()
                    //             }
                    //         }

                    //         let mut parse = MedigunParse::default();

                    //         for prop in ent.props(_parser_state) {
                    //             match prop.identifier {
                    //                 BENT_TEAMNUM => {
                    //                     let team = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     println!("medigun team: {}", team);
                    //                 }
                    //                 BENT_HOWNER => {
                    //                     let hand = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     self.state.seen_ent_handles.insert(hand);
                    //                     println!("saw howner bent {:?}", hand);
                    //                     parse.bent_owner = Some(hand);
                    //                 },
                    //                 BENT_MOVEPARENT => {
                    //                     if let SendPropValue::String(s) = prop.value {
                    //                         parse.bent_moveparent = s.into();
                    //                     }
                    //                     else {println!("what? lol prop value bent moveparent not str: {:?}", prop.value);}
                    //                 }
                    //                 BCOMBWEP_HOWNER => {
                    //                     let hand = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     self.state.seen_ent_handles.insert(hand);
                    //                     println!("saw howner combwep {:?}", hand);
                    //                     parse.bcomw_owner = Some(hand);
                    //                 }
                    //                 BCOMBWEP_STATE => {
                    //                     parse.bcomw_state = Some(i64::try_from(&prop.value).unwrap_or_default() as u32);
                    //                 }
                    //                 WEP_MEDIGUN_BHEALING => {
                    //                     parse.wmed_healing = Some(i64::try_from(&prop.value).unwrap_or_default() != 0);
                    //                 }
                    //                 WEP_MEDIGUN_BHOLSTERED => {
                    //                     parse.wmed_holstered = Some(i64::try_from(&prop.value).unwrap_or_default() != 0);
                    //                 }
                    //                 WEP_MEDIGUN_HHEALTARGET => {
                    //                     let hand = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     self.state.seen_ent_handles.insert(hand);
                    //                     parse.wmed_healtarg = Some(hand);
                    //                 }
                    //                 WEP_MEDIGUN_HLASTHEALTARGET => {
                    //                     let hand = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    //                     self.state.seen_ent_handles.insert(hand);
                    //                     parse.wmed_lasthealtarg = Some(hand);
                    //                 }
                    //                 _ => {}
                    //             }
                    //         }

                    //         if parse.has() {
                    //             if let Some(targ_id) = parse.wmed_healtarg {
                    //                 if _parser_state.entity_classes.contains_key(&EntityId::from(targ_id)) {
                    //                     println!("nice try");
                    //                 }
                    //                 if self.state.seen_player_entids.contains(&targ_id) {
                    //                     println!(":: Medigun target matched entity id");
                    //                 }
                    //                 else {
                    //                     //println!(":: Medigun target {} did not match any player entity id", targ_id);
                    //                     // aight fuck it we ball

                    //                     // check EVERYTHING we can for this target id
                    //                     //println!("ent class: {}, ent class id: {}",
                    //                     //    _parser_state.entity_classes.contains_key(&EntityId::from(targ_id)),
                    //                     //    _parser_state.entity_classes.values().contains(&ClassId::from(targ_id as u16))
                    //                     //);
                    //                 }
                    //             }
                    //             //println!(":: Medigun parse has props: {:?}", ent.props(_parser_state).collect_vec());
                    //             //println!(":: Medigun parse has props: {:?}", parse);
                    //         }
                    //     }

                    //     "CTFProjectile_Rocket"
                    //     | "CTFGrenadePipebombProjectile"
                    //     => {
                    //         const BASEENTITY_HOWNER: SendPropIdentifier = 
                    //             SendPropIdentifier::new("DT_BaseEntity", "m_hOwnerEntity");

                            

                    //         if ent.update_type == UpdateType::Enter {
                    //             // println!("- {:}", class_name);
                    //             // for prop in ent.props(_parser_state) {
                    //             //     println!(":: {:?}", prop);
                    //             // }

                    //             for prop in ent.props(_parser_state) {
                    //                 match prop.identifier {
                    //                     BASEENTITY_HOWNER => {
                    //                         //println!("Projectile Enter: HOwner {}", prop.value);
                    //                     },
                    //                     _ => {}
                    //                 }
                    //             }
                    //         }
                    //     }
                    //     _ => {}
                    // }
                }
            }
            Message::GameEvent(m) => {
                self.state.seen_game_event_types.insert(m.event.event_type());

                match &m.event {
                    GameEvent::TeamPlayPointCaptured(evt) => {
                        println!("{} Point {} captured (\"{:?}\") by {:?} team: {:?}",
                            tick, evt.cp, evt.cp_name, Team::new(evt.team), evt.cappers);

                        
                    }

                    // GameEvent::PlayerSpawn(evt) => {
                    //     println!("{} player spawn: {} (team: {}, class: {:?})", tick, evt.user_id, evt.team, crate::types::game::Class::new(evt.class))
                    // }

                    // GameEvent::PlayerHealed(_evt) => {
                    //     // Player healed reports:
                    //     // 1: A full ~second of healing by a medic
                    //     // 2: If the player only gets healed a little bit by a medic
                    //     // 3: Crossbow heals
                    //     // 4: Dispenser heals (definitely)
                    //     // 5: Regeneration (medic, cozy, etc.)
                    //     // Essentially, it reports whenever the +hp appears above your healthbar
                    //     //println!("{} player healed: {} healed {} by {}", tick, evt.patient, evt.amount, evt.healer);
                    // }

                    // GameEvent::CrossbowHeal(_evt) => {
                    //     //println!("{} crossbow heal: {} healed {} by {}", tick, evt.target, evt.amount, evt.healer)
                    // }

                    // GameEvent::MedicDeath(_evt) => {
                    //     //println!("{} MEDIC DEATH: {}, was healing {}", tick, evt.user_id, evt.healing);
                    // }

                    // GameEvent::PlayerRegenerate(_) => {
                    //     // Player regenerate = class change probably
                    //     //println!("{} player regenerate", tick)
                    // }
                    _ => {}
                }

                match &m.event {
                    // GameEvent::RoundStart(_)
                    GameEvent::TeamPlayRoundStart(_)
                    | GameEvent::TeamPlayRestartRound(_)
                    // | GameEvent::TeamPlayMapTimeRemaining(_)
                    // | GameEvent::PlayerChargeDeployed(_)
                    | GameEvent::TeamPlayRoundWin(_)
                    | GameEvent::ControlPointStartTouch(_)
                    | GameEvent::ControlPointEndTouch(_)
                    | GameEvent::TeamPlayCaptureBlocked(_)
                    | GameEvent::TeamPlayCaptureBroken(_)
                    | GameEvent::TeamPlayPointCaptured(_)
                    // | GameEvent::PlayerRegenerate(_)
                    // | GameEvent::NpcHurt(_)
                    //GameEvent::PlayerHealed(_)
                    //| GameEvent::PlayerHurt(_)
                    //| GameEvent::RocketJump(_)
                    //| GameEvent::StickyJump(_)
                    //| GameEvent::RocketJumpLanded(_)
                    //| GameEvent::StickyJumpLanded(_)
                    => {
                        self.state.interesting_events
                            .entry(u32::from(tick))
                            .or_default()
                            .push(m.event.event_type());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_packet_meta(
            &mut self,
            tick: DemoTick,
            _meta: &tf_demo_parser::demo::packet::message::MessagePacketMeta,
            _parser_state: &ParserState,
        ) {
        self.state.seen.push(GatherSeen::PacketMeta(u32::from(tick)));
    }

    fn handle_string_entry(
            &mut self,
            table: &str,
            index: usize,
            _entries: &StringTableEntry,
            _parser_state: &ParserState,
        ) {
        self.state.seen.push(GatherSeen::StringEntry(String::from(table), index, _entries.to_owned()));
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }
}

//use std::str::FromStr;

pub enum WeaponSlot {
    Primary = 0,
    Secondary = 1,  // Includes sapper & medigun
    Melee = 2,
    PDA1 = 3,       // Build PDA / Invis watch
    PDA2 = 4 ,      // Destroy PDA / Disguise kit
}

fn item_useable_class(class_name: &str) -> Option<(Class, WeaponSlot)> {
    // Divide the class names into ones that are:
    // 1: Class specific
    // 2: Carry information about the team
    // 3: Have a moveparent and/or handle to their owner

    // All weapons have the following class hierarchy:
    // CBaseEntity > CBaseAnimating > CEconEntity > CBaseCombatWeapon > CTFWeaponBase
    // Within CBaseEntity: https://sigwiki.potato.tf/index.php/CBaseEntity
    //      - m_iTeamNum            The team this object is on. For weapons on players should be 2/3.
    //      - m_hOwnerEntity        The handle we care about and are trying to match, typically
    //      - moveparent            Immediate parent in movement hierarchy. Typically, is the player.
    //                              Notably, the wiki reports this as a String, but it is an Integer.
    
    // Within CBaseCombatWeapon:
    //      - m_hOwner              Should be the same as m_hOwnerEntity ?


    // After WeaponBase, there are a few more classes that specialize:
    // CTFWeaponBaseMelee   - the base class for all melee weapons
    // CTFWeaponBaseGun     - All ranged weapons, e.g. primaries and secondaries
    // CTFLunchBox          - Any edible items
    // CTFWeaponBuilder     - Engineer's hidden "builder" weapon and Spy's stock Sapper
    // CTFWeaponInvis       - Invis watches
    // CTFWeaponPDA         - Base class for Engineer PDAs and Diguise Kit

    use Class::*;
    use WeaponSlot::*;
    // https://sigwiki.potato.tf/index.php/Entity_Properties
    match class_name {
        // All scout primaries are covered
        "CTFScattergun"             // Stock, FaN, and Back Scatter
        | "CTFSodaPopper"           // Soda popper
        | "CTFPEPBrawlerBlaster"    // Baby Face's
        | "CTFPistol_ScoutPrimary"  // Shortstop
        => Some((Scout, Primary)),

        "CTFPistol_ScoutSecondary"  // 
        | "CTFPistol_Scout"         // < CTFWeaponBaseGun
        | "CTFLunchBox_Drink"       // < CTFLunchBox    drinks
        | "CTFCleaver"              // < CTFJar         cleaver
        | "CTFJarMilk"              // < CTFJar         mad milk
        => Some((Scout, Secondary)),

        "CTFBat"                // < CTFWeaponBaseMelee all other bats
        | "CTFBat_Fish"         // < CTFBat             FISH
        | "CTFBat_Wood"         // < CTFBat             sandman
        | "CTFBat_Giftwrap"     // < CTFBat_Wood        wrap assassin
        => Some((Scout, Melee)),

        // All soldier primaries are covered, so soldier is covered
        "CTFRocketLauncher"             // < CTFWeaponBaseGun
        | "CTFRocketLauncher_AirStrike" // < CTFRocketLauncher
        | "CTFRocketLauncher_DirectHit" // < ^
        | "CTFRocketLauncher_Mortar"    // < ^ unused in-game but included for posterity
        | "CTFParticleCannon"           // < ^Cow Mangler
        => Some((Soldier, Primary)),

        "CTFShotgun_Soldier"
        | "CTFBuffItem"
        | "CTFParachute_Secondary"
        | "CTFRaygun"
        => Some((Soldier, Secondary)),

        "CTFShovel" // couldn't find pickaxes ?
        => Some((Soldier, Melee)),

        // All pyro primaries are covered
        "CTFFlamethrower"
        | "CTFWeaponFlameBall"
        => Some((Pyro, Primary)),

        "CTFFlareGun"
        | "CTFFlareGun_Revenge" // man melter
        | "CTFShotgun_Pyro"
        | "CTFJarGas"
        => Some((Pyro, Secondary)),

        "CTFFireAxe"
        | "CTFSlap"
        | "CTFBreakableSign"    // neon annihilator
        => Some((Pyro, Melee)),

        // Holes in finding demo handles:
        // -> Booties + Shield + Katana/Pain Train
        // Would need to look at wearables :(
        "CTFGrenadeLauncher"
        | "CTFCannon"
        | "CTFParachute_Primary"
        => Some((Demoman, Primary)),

        "CTFPipebombLauncher"
        => Some((Demoman, Secondary)),

        "CTFBottle"
        | "CTFStickBomb"        // caber
        | "CTFSword"        // does not include katana; that's CTFKatana
        => Some((Demoman, Melee)),

        "CTFMinigun"
        => Some((Heavy, Primary)),

        "CTFShotgun_HWG"
        | "CTFLunchBox"
        => Some((Heavy, Secondary)),

        "CTFFists"
        => Some((Heavy, Melee)),

        "CTFShotgun_Revenge"           // frontier
        | "CTFShotgunBuildingRescue"    // rescue ranger
        => Some((Engineer, Primary)),

        "CTFMechanicalArm"      // short circuit
        | "CTFLaserPointer"     // wrangler
        => Some((Engineer, Secondary)),

        "CTFWrench"
        | "CTFRobotArm"
        => Some((Engineer, Melee)),

        "CTFWeaponPDA"
        | "CTFWeaponPDA_Engineer_Build"
        => Some((Engineer, PDA1)),

        "CTFWeaponPDA_Engineer_Destroy"
        => Some((Engineer, PDA2)),

        "CTFSyringeGun"
        | "CTFCrossbow" // inherits from CTFRocketLauncher funnily enough
        => Some((Medic, Primary)),

        "CWeaponMedigun"
        => Some((Medic, Secondary)),

        "CTFBonesaw"
        => Some((Medic, Melee)),

        "CTFSniperRifle"
        | "CTFSniperRifleClassic"
        | "CTFSniperRifleDecap"
        | "CTFCompoundBow"  // both hunts & compound, inherits from PipebombLauncher
        => Some((Sniper, Primary)),

        "CTFSMG"
        | "CTFChargedSMG"
        | "CTFJar"
        => Some((Sniper, Secondary)),

        "CTFClub"
        => Some((Sniper, Melee)),

        "CTFRevolver"
        => Some((Spy, Primary)),

        "CTFWeaponSapper"
        => Some((Spy, Secondary)),

        "CTFKnife"
        => Some((Spy, Melee)),

        "CTFWeaponInvis"
        => Some((Spy, PDA1)),

        "CTFWeaponPDA_Spy"
        => Some((Spy, PDA2)),

        _ => None
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MatchAttempt {
    pub h_owner: u32,
    pub team: Team,
    pub class: Class,
}


impl Gatherer {
    fn handle_entity(&mut self, ent: &PacketEntity, tick: DemoTick, parser_state: &ParserState) {
        let class_name: &str = self
            .class_names
            .get(usize::from(ent.server_class))
            .map(|class_name| class_name.as_str())
            .unwrap_or("");

        for prop in ent.props(parser_state) {
            if let Some((table_name, prop_name)) = prop.identifier.names() {
                if table_name == "DT_AttributeContainer" && prop_name == "m_hOuter" {
                    let val = i64::try_from(&prop.value).unwrap_or_default() as u32;
                    println!("{:} @ {} m_hOuter: {} ({:X})", class_name, ent.entity_index, val, val);
                    self.state.outer_map.insert(
                        val,
                        u32::from(ent.entity_index)
                    );
                }
            }
        }

        if let Some((class, _)) = item_useable_class(class_name) {
            self.handle_class_entity(ent, tick, parser_state, class);
        }
        else {
            match class_name {
                "CTFPlayer" => self.handle_player_entity(ent, tick, parser_state),
                //"CTFPlayerResource" => self.handle_player_resource(ent, tick, parser_state),
                _ => {}
            }
        }
    }

    fn handle_player_entity(
        &mut self,
        ent: &PacketEntity,
        _tick: DemoTick,
        _parser_state: &ParserState
    ) {
        if ent.update_type == UpdateType::Enter {
            println!("player enter table names & associated properties: ");
            for prop in ent.props(_parser_state) {
                if let Some((table_name, prop_name)) = prop.identifier.names() {
                    println!("{:}.{:}: {:?}", table_name, prop_name, prop.value);
                }
            }
        }

        self.state.seen_player_entids.insert(u32::from(ent.entity_index));
    }

    fn handle_class_entity(&mut self, ent: &PacketEntity, _tick: DemoTick, parser_state: &ParserState, class: Class) {
        const BENT_TEAMNUM: SendPropIdentifier = 
            SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");

        const BENT_HOWNER: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseEntity", "m_hOwnerEntity");

        //const BENT_MOVEPARENT: SendPropIdentifier =
        //    SendPropIdentifier::new("DT_BaseEntity", "moveparent");
        

        if ent.update_type == UpdateType::Enter {
            let mut matmp = MatchAttempt::default();
            matmp.class = class;
            for prop in ent.props(parser_state) {
                match prop.identifier {
                    BENT_HOWNER => {
                        matmp.h_owner = i64::try_from(&prop.value).unwrap_or_default() as u32;
                        self.state.seen_ent_handles.insert(matmp.h_owner);
                        // if let Some((table_name, prop_name)) = prop.identifier.names() {
                        //     if let Ok(player_id) = u32::from_str(prop_name.as_str()) {
                        //         println!("what the fuck: {}", player_id)
                        //     }
                        // }
                    },
                    BENT_TEAMNUM => {
                        matmp.team = Team::new(i64::try_from(&prop.value).unwrap_or_default());
                    },
                    _ => {}
                }
            }

            if matmp.h_owner != 0 && matmp.team.is_player() {
                self.match_attempts.push(matmp);
            }
        }
    }
}

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

    pub fn handle_projectile_entity(&mut self, entity: &PacketEntity, _parser_state: &ParserState) {
        let class_name: &str = get_class_name!(self, entity.server_class);
        println!("PROJECTILE {:?} ({:} @ {})", entity.update_type, class_name, entity.entity_index);
        println!("Props: {:?}", entity.props);
    }
}