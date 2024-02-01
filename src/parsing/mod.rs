////////////////////////////////////////////////////
//! # Parsing
//! 
//! Contains structures that run through a demo file.
//! This is kept completely separate from ui and drawing.
//! 
//! ## Mods
//! 
//! [`customanalyser`] analyses the data collected from [`datacollect`].
//! [`datacollect`] collects data for each individual game state.
//! [`types`] contains types relevant to other mods in this package.
//! 
//! ## Structures
//! [`Parsing`] takes a demo file and collects data about it.
//! [`ParseDrawInfo`] contains information about a parse relevant to rendering
//! any part of it.

pub mod datacollection;
pub mod internals;

// INCLUDES
use super::types::game::entities::World;
use super::types::demo::DemoData;

use self::datacollection::GameStateAnalyserPlus;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use tf_demo_parser::demo::parser::ParseError;
use tf_demo_parser::DemoParser;

#[derive(Debug)]
pub enum ParseWorkerError {
    ParseError(ParseError),
    IoError(std::io::Error),
    SendError
}

impl From<ParseError> for ParseWorkerError {
    fn from(value: ParseError) -> Self {
        ParseWorkerError::ParseError(value)
    }
}

impl From<mpsc::SendError<ParseProgressReport>> for ParseWorkerError {
    fn from(_value: mpsc::SendError<ParseProgressReport>) -> Self {
        ParseWorkerError::SendError
    }
}

impl From<std::io::Error> for ParseWorkerError {
    fn from(value: std::io::Error) -> Self {
        ParseWorkerError::IoError(value)
    }
}

#[derive(Default, Debug)]
pub enum ParseProgressReport {
    #[default]
    Waiting,
    Info(u32),
    Working(u32),
    Done(DemoData, ParseDrawInfo),
    Error(ParseWorkerError)
}

pub struct ParseWorker {
    #[allow(dead_code)]
    handle: JoinHandle<()>,
    prog_recv: mpsc::Receiver<ParseProgressReport>,
}

/// Information required to properly render the parse.
#[derive(Default, Debug, Clone)]
pub struct ParseDrawInfo {
    pub max_players: u32,
    pub max_projectiles: u32,
    pub world_max: World,
    pub player_at_max: World
}

impl ParseWorker {
    pub fn new(fpath: PathBuf) -> std::io::Result<Self> {
        log::info!("Beginning parse: {:?}", fpath.clone());

        let (prog_send, prog_recv) = mpsc::channel::<ParseProgressReport>();

        let handle = thread::spawn(move || {
            use ParseProgressReport::*;
            let error_catch = || -> Result<(), ParseWorkerError> {
                let mut seen_zero = false;
                let file = std::fs::read(fpath.clone())?;
                let demo = tf_demo_parser::Demo::new(&file);

                let parser = DemoParser::new_all_with_analyser(
                    demo.get_stream(),
                    GameStateAnalyserPlus::new()
                );

                let (header, mut ticker) = parser.ticker()?;

                let mut result_data = DemoData {
                    demo_filename: fpath.clone(),
                    map_name: header.map,
                    duration: header.duration,
                    ..Default::default()
                };

                prog_send.send(Info(header.ticks))?;

                let mut draw_data = ParseDrawInfo::default();

                while ticker.tick()? {
                    let state = ticker.state();
                    if state.data.tick <= 10 {
                        seen_zero = true;
                    }

                    if seen_zero {
                        prog_send.send(Working(u32::from(state.data.tick)))?;
                    }

                    // Update the rounds in our results. Copy is just simpler.
                    result_data.rounds = state.rounds.clone();
                    result_data.kills.extend(state.kills.clone());
                    result_data.tick_states.insert(u32::from(state.data.tick), state.data.clone());

                    // Update draw data
                    // TODO: Max projectiles
                    if let Some(world) = &state.world {
                        draw_data.world_max = draw_data.world_max.adjoin_bounds(world);
                    }
                    
                    for player in &state.data.players {
                        draw_data.player_at_max.stretch_to_include(player.position);
                    }
                }

                // deliberate lack of ?
                // we want the done() to be the last thing we could potentially send
                prog_send.send(Done(result_data, draw_data)).map_err(|e| ParseWorkerError::from(e))

                //Ok(())
            };

            if let Err(err) = error_catch() {
                let _ = prog_send.send(Error(err));
            }
        });

        Ok(ParseWorker {
            handle,
            prog_recv
        })
    }

    
    #[allow(dead_code)]
    pub fn get_next(&self) -> Option<ParseProgressReport> {
        match self.prog_recv.try_recv() {
            Ok(report) => {
                Some(report)
            },
            _ => {None}
        }
    }

    pub fn get_most_recent(&self) -> Option<ParseProgressReport> {
        let mut last: Option<ParseProgressReport> = None;
        
        loop {
            match self.prog_recv.try_recv() {
                Ok(report) => {
                    last = Some(report);
                },
                _ => {break}
            }
        }

        last
    }
}




// //use egui::mutex::Mutex;
// use tf_demo_parser::{self, DemoParser};
// use std::{collections::BTreeMap, sync::{RwLock, Arc}, thread::{self, JoinHandle}, path::PathBuf};

// // PUBLIC INCLUDES & RE-EXPORTS


// pub use tf_demo_parser::demo::data::DemoTick;
// pub use self::datacollection::TickGameState;
// //pub use super::analysis::StateAnalysisData;



// /// Contains the results of a demo parse.
// #[derive(Default, Clone)]
// pub struct ParseResult {
//     // Game data
//     pub demo_fname: PathBuf,
//     pub tick_data: BTreeMap<DemoTick, TickGameState>,
//     pub rounds: Vec<types::Round>,
//     pub map_name: String,
//     pub duration: f32,  //Playback time in seconds

//     // Draw information
//     pub draw_info: ParseDrawInfo,
// }

// #[derive(Default)]
// struct ParseProgress {
//     pub map_name: String,
//     pub max_tick: u32,
//     pub current_tick: u32,
// }

// /// An in-progress parse. Returned from [`parse_demo`].
// /// 
// /// Call `get_result` to get the result of the parse.
// pub struct ParseInProgress {
//     pub demo_fname: PathBuf,
//     handle: Option<JoinHandle<ParseResult>>,
//     progress: Arc<RwLock<ParseProgress>>
// }

// impl ParseInProgress {
//     /// Returns the [`ParseResult`] of the parse.
//     /// This will block the current thread until it is finished.
//     /// Consider calling `in_progress` until it returns
//     /// false before calling this function.
//     pub fn get_result(&mut self) -> thread::Result<ParseResult> {
//         self.handle.take().map(JoinHandle::<ParseResult>::join).unwrap()
//     }

//     pub fn get_current_tick(&self) -> u32 {
//         self.progress.read().expect("could not read lock parse thread").current_tick
//     }

//     pub fn get_max_tick(&self) -> u32 {
//         self.progress.read().expect("could not read lock parse thread").max_tick
//     }

//     pub fn in_progress(&self) -> bool {
//         match &self.handle {
//             Some(handle) => !handle.is_finished(),
//             None => false
//         }
//     }
// }

// /// Spawns a thread that parses a demo file at a given path.
// /// Returns an in-progress parse that can be used to get the result and
// /// check the progress of the parse.
// pub fn parse_demo(fpath: std::path::PathBuf) -> ParseInProgress {
//     let prog = Arc::new(RwLock::new(ParseProgress::default()));
//     let moveprog = prog.clone();

//     ParseInProgress {
//         demo_fname: fpath.clone(),
//         handle: Some(thread::spawn(move || {_parse_demo(moveprog, fpath.clone())})),
//         progress: prog
//     }
// }

// /// Actual parsing. This is the 
// fn _parse_demo(
//     prog: Arc<RwLock<ParseProgress>>,
//     fpath: std::path::PathBuf
// ) -> ParseResult {
//     log::info!("Beginning parse of {:?}", fpath);

//     // Reading the file & starting the parse
//     let file = std::fs::read(fpath.clone()).unwrap();
//     let demo = tf_demo_parser::Demo::new(&file);

//     let parser = DemoParser::new_all_with_analyser(
//         demo.get_stream(),
//         GameStateAnalyserPlus::new()
//     );

//     // Get the header information and ticker
//     let (header, mut ticker) = parser.ticker().unwrap();

//     prog.write().unwrap().map_name = header.map.clone();
//     prog.write().unwrap().max_tick = header.ticks;

//     // Set up result return values
//     let mut results = ParseResult{
//         demo_fname: fpath.clone(),
//         map_name: header.map,
//         ..Default::default()
//     };

//     while ticker.tick().unwrap_or(false) {
//         let state = ticker.state();

//         log::debug!("Parsing tick {}", state.gs.tick);
//         prog.write().unwrap().current_tick = u32::from(state.gs.tick);

//         // Clone the state into our result tick data
//         results.tick_data.insert(state.gs.tick, state.clone());

//         // Check rounds
//         if state.rounds.len() > results.rounds.len() {
//             log::info!("New round found ({})", results.rounds.len());

//             results.rounds = state.rounds.to_vec();
//         }

//         if state.gs.players.len() > results.draw_info.max_players as usize {
//             results.draw_info.max_players = state.gs.players.len() as u32;
//         }

//         // Player data tracking:
//         for player in &state.gs.players {
//             results.draw_info.player_at_max.boundary_min.x = f32::min(results.draw_info.player_at_max.boundary_min.x, player.position.x);
//             results.draw_info.player_at_max.boundary_min.y = f32::min(results.draw_info.player_at_max.boundary_min.y, player.position.y);
//             results.draw_info.player_at_max.boundary_min.z = f32::min(results.draw_info.player_at_max.boundary_min.z, player.position.z);
//             results.draw_info.player_at_max.boundary_max.x = f32::max(results.draw_info.player_at_max.boundary_max.x, player.position.x);
//             results.draw_info.player_at_max.boundary_max.y = f32::max(results.draw_info.player_at_max.boundary_max.y, player.position.y);
//             results.draw_info.player_at_max.boundary_max.z = f32::max(results.draw_info.player_at_max.boundary_max.z, player.position.z);
//         }

//         // Projectile tracking:
//         if state.projectile_update_this_tick.len() > results.draw_info.max_projectiles as usize {
//             results.draw_info.max_projectiles = state.projectile_update_this_tick.len() as u32;
//         }
//         if let Some(world) = &state.gs.world {
//             results.draw_info.world_max = world.clone();
//         }
//     }

//     log::debug!("PARSE INFO:\n- World: {} {} {} to {} {} {}\n- Max Players: {}\n- Max Projectiles: {}",
//         results.draw_info.world_max.boundary_min.x,
//         results.draw_info.world_max.boundary_min.y,
//         results.draw_info.world_max.boundary_min.z,
//         results.draw_info.world_max.boundary_max.x,
//         results.draw_info.world_max.boundary_max.y,
//         results.draw_info.world_max.boundary_max.z,
//         results.draw_info.max_players,
//         results.draw_info.max_projectiles,
//     );

//     log::debug!("- Player Reach: {} {} {} to {} {} {}",
//         results.draw_info.player_at_max.boundary_min.x,
//         results.draw_info.player_at_max.boundary_min.y,
//         results.draw_info.player_at_max.boundary_min.z,
//         results.draw_info.player_at_max.boundary_max.x,
//         results.draw_info.player_at_max.boundary_max.y,
//         results.draw_info.player_at_max.boundary_max.z,
//     );

//     // Returns
//     results
// }
/*How to get DemoAnalysis project up and running:
	1. Install rustup and then Rust
	2. Install git
	2. Install python (at least 3.8) if not already on the computer
		OR
	2. Install pyenv:
		2.0 Windows, in powershell: 
		2.1 Run "pip install pyenv-win --target $HOME\\.pyenv" to install
		3.2 Add system settings as environment variables:
			3.2.1: Set PYENV, PYENV_HOME, and PYENV_ROOT to $HOME\.pyenv\pyenv-win
			3.2.2: Add $HOME\.pyenv\pyenv-win\bin and $HOME\.pyenv\pyenv-win\shims to Path 
			(https://github.com/pyenv-win/pyenv-win/blob/master/docs/installation.md#add-system-settings)
	4. Install maturin:
		4.0 In windows,
	4. Download VS Code and install rust and python packages & set them up
*/