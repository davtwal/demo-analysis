//////////////////////////////
//! Author: David Walker
//! Date:   11.30.23
//! Name:   datacollect.rs
//! Purpose:    
//!     The [`viewing`] crate wants to be able to see some of the tick information
//!     gathered, so this 
//!     analysis is one in rust instead of in python.
//////////////////////////////


use std::collections::HashMap;

use quake_inverse_sqrt::QSqrt;
use tf_demo_parser::demo::gamevent::GameEventType;

use crate::types::{DemoTick, EntityId};
use crate::types::demo::TickData;
use crate::types::game::{Class, Team};
use crate::types::game::entities::{Player, PlayerState};
use crate::types::math::{Vector, VectorXY};

use super::parsing::datacollection::TickGameState;

#[derive(Debug, Clone, Default)]
pub enum GroupingType {
    #[default]
    None = 0,           // Not valid
	Isolated = 1,   	// (1)  Only one player.
	IsolatedCombo = 2,  // (2)  Contains the medic and one other players.
	Combo = 3,      	// (3+) Contains the medic and at least two other players.
	Flank = 4,      	// (2+) Does not contain the medic, but has at least two players.
}

/*
A quick word about the data structures in here.
DO NOT create - on your own - ANY of the structures in this module
EXCEPT StateAnalysisData, and ONLY use the From::<GameStatePlus> function to
create them. Using that function will fill out all of the remainder of the insides.
Table:
    - StateAnalysisData is the parent structure that contains all of the
        analysis data for the specified game state. There should only be one
        of these per game state, and it will create a copy of the game state
        passed into the From function.

    - TeamData contains the data for each team.
*/

fn get_avg_pos(players: &Vec<&Player>) -> Vector {
    players.iter().cloned()
                  .filter(|p| p.state == PlayerState::Alive)
                  .map(|p| p.position)
                  .zip(1..)
                  .fold(Vector::default(),
                    |acc, x| acc + Vector{
                        x: (x.0.x - acc.x) / x.1 as f32,
                        y: (x.0.y - acc.y) / x.1 as f32,
                        z: (x.0.z - acc.z) / x.1 as f32
                  })
}

#[derive(Default, Debug, Clone)]
pub struct PlayerData {
    pub id: u32,
    pub dist_from_team_avg: f32,
    pub dist_from_group_avg: f32,
    pub dist_from_medic: f32,
}

// This is the only structure
impl PlayerData {
    pub fn new(player: &Player, team: &TeamData, grouping: &Grouping) -> Self {
        PlayerData {
            id: player.info.as_ref().unwrap().entity_id,
            dist_from_team_avg: player.position.dist_to(&team.avg_position),
            dist_from_group_avg: player.position.dist_to(&grouping.avg_position),
            dist_from_medic: player.position.dist_to(&team.medic_position),
        }
    }
}

impl From<&Vec<&Player>> for GroupingType {
	fn from(val: &Vec<&Player>) -> Self {
    	match val.len() {
        	1 => GroupingType::Isolated,
        	2 => {
            	if val.iter().filter(|p| p.class == Class::Medic).collect::<Vec<&&Player>>().len() == 0 {
                	GroupingType::Flank
            	} else {
                	GroupingType::IsolatedCombo
            	}
        	}
        	_ => {
            	if val.iter().filter(|p| p.class == Class::Medic).collect::<Vec<&&Player>>().len() == 0 {
                	GroupingType::Flank
            	} else {
                	GroupingType::Combo
            	}
        	}
    	}
	}
}

#[derive(Clone)]
pub struct Grouping {
	pub r#type: GroupingType,
	pub player_ids: Vec<u32>,
    pub avg_position: Vector,
}

impl From<&Vec<&Player>> for Grouping {
    fn from(value: &Vec<&Player>) -> Self {
        Grouping {
            r#type: GroupingType::from(value),
            player_ids: value.iter().map(|p| p.info.as_ref().unwrap().entity_id).collect(),
            avg_position: get_avg_pos(&value),
        }
    }
}

pub fn build_groupings(players: &Vec<&Player>, grouping_threshold: f32) -> Vec<Grouping> {
    let mut close = Vec::<Vec<&Player>>::new();

    let grouped: Vec<usize> = Vec::new();
    for i in 0..players.len() {
        if grouped.contains(&i) {continue;}
        let player = players.get(i).unwrap();
        let mut group = Vec::<&Player>::new();

        for j in i..players.len() {
            if grouped.contains(&j) {continue;}
            
            if player.position.dist_to(&players.get(j).unwrap().position) < grouping_threshold {
                group.push(players.get(j).unwrap());
            }
        }

        group.push(player);
        close.push(group);
    }

    close.iter().cloned().map(|c| Grouping::from(&c)).collect()
}

#[derive(Clone)]
pub struct TeamData {
    pub team: Team,
    pub groupings: Vec<Grouping>,
    pub avg_position: Vector,
    pub medic_position: Vector,
    pub player_data: HashMap<u32, PlayerData>,
}

impl TeamData {
    pub fn new(players: &Vec<&Player>, id_to_player: &HashMap<u32, &Player>) -> Option<Self> {
        // each player will have the same team as the first player
        // as they were already filtered

        // Make sure we filter out dead players here
        if players.len() == 0 {
            return None;
        }

        let avg_pos = get_avg_pos(&players);

        let medic_pos = players.iter().filter(|p| p.class == Class::Medic)
                                      .map(|p| p.position)
                                      .collect::<Vec<Vector>>()
                                      .get(0).map_or(Vector::default(), |x| *x);
        
        Some(TeamData {
            team: players.first().unwrap().team,
            groupings: build_groupings(&players, 600.0),
            avg_position: avg_pos,
            medic_position: medic_pos,
            player_data: HashMap::new()
        }.fill_player_data(&id_to_player))
    }

    fn fill_player_data(mut self, id_to_player: &HashMap<u32, &Player>) -> Self {
        //Fills in the rest of the data, primarily the player data.
        for grouping in &self.groupings {
            for pid in &grouping.player_ids {
                self.player_data.insert(*pid, PlayerData::new(
                    id_to_player.get(pid).unwrap(),
                    &self,
                    &grouping
                ));
            }
        }

        self
    }
}

#[derive(Clone)]
pub struct TickAnalysisData {
    pub state: TickData,
    pub red_teamdata: Option<TeamData>,
    pub blue_teamdata: Option<TeamData>,
    player_id_map: HashMap<u32, usize>,
}

impl TickAnalysisData {
    pub fn id_to_player<'a>(&'a self, id: u32) -> &'a Player {
        self.state.players.get(self.player_id_map[&id]).unwrap()
    }
}

// THIS IS THE FUNCTION WHERE STATE ANALYSIS STARTS!
impl From<&TickData> for TickAnalysisData {
    fn from(value: &TickData) -> Self {
        let red_iter = value.players.iter().filter(|p| p.team == Team::Red);
        let blu_iter = value.players.iter().filter(|p| p.team == Team::Blue);

        let id_to_player = value.players.iter()
                                           .map(|p| p.info.as_ref().unwrap().entity_id)
                                           .zip(&value.players)
                                           .collect::<HashMap<u32, &Player>>();

        let player_id_map = value.players.iter()
                                            .map(|p| p.info.as_ref().unwrap().entity_id)
                                            .enumerate()
                                            .map(|(i,p)| (p, i)) // Reverse order
                                            .collect::<HashMap<u32, usize>>();

        let red_data = TeamData::new(&red_iter.collect::<Vec<&Player>>(), &id_to_player);
        let blue_data = TeamData::new(&blu_iter.collect::<Vec<&Player>>(), &id_to_player);
        
        TickAnalysisData { 
            state: value.clone(),
            red_teamdata: red_data,
            blue_teamdata: blue_data,
            player_id_map,
        }
    }
}


//use self::data::PlayerData;

//pub mod sums;
//pub mod data;
pub mod grouping;

pub mod data;

/*impl<'a> From<&TickGameState> for TickAnalysis {
    fn from(value: &TickGameState) -> Self {
        
    }
}*/

/*
When analysing the data from a demo, we primarily want to look at the data from rounds.
As such, this is the hierarchy of analysis:
    DEMO ANALYSIS
    \-  ROUND ANALYSIS
        \- TICK ANALYSIS

*/
