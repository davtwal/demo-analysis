//! Data types for passing to analysis

use pyo3::prelude::*;

use std::collections::HashMap;

use crate::types::game::{Team, Class};
use crate::types::entities::Player;
use crate::types::demo::TickData;
use crate::types::math::Vector;

#[derive(Debug, Clone)]
pub struct TickPlayerData<'a> {
    pub player: &'a Player,
    pub time_since_last_hit: f32,
    pub dist_from_team_avg: f32,
    //pub dist_from_group_avg: f32,
    pub dist_from_medic: f32,
}

#[pyclass(name="TickPlayerData", get_all)]
#[derive(Default, Debug, Clone)]
pub struct TickPlayerDataPy {
    pub entity_id: u32,
    pub user_id: u16,
    pub time_since_last_hit: f32,
    pub dist_from_team_avg: f32,
    //pub dist_from_group_avg: f32,
    pub dist_from_medic: f32,
}

impl IntoPy<TickPlayerDataPy> for TickPlayerData<'_> {
    fn into_py(self, _py: Python<'_>) -> TickPlayerDataPy {
        let entity_id = self.player.info.as_ref().expect("x").entity_id;
        let user_id = self.player.info.as_ref().expect("x").user_id;
        TickPlayerDataPy {
            entity_id,
            user_id,
            time_since_last_hit: self.time_since_last_hit,
            //dist_from_group_avg: self.dist_from_group_avg,
            dist_from_medic: self.dist_from_medic,
            dist_from_team_avg: self.dist_from_team_avg,
        }
    }
}


#[derive(Default, Debug, Clone)]
pub struct TickTeamAnalysis<'a> {
    pub team: Team,
    pub medic: Option<&'a Player>,
    pub playerdata: HashMap<u16, TickPlayerData<'a>>,
}

impl IntoPy<TickTeamAnalysisPy> for TickTeamAnalysis<'_> {
    fn into_py(self, py: Python<'_>) -> TickTeamAnalysisPy {
        TickTeamAnalysisPy {
            team: self.team,
            medic: self.medic.cloned(),
            playerdata: self.playerdata.iter().map(|(id, pd)| (*id, pd.clone().into_py(py))).collect()
        }
    }
}

#[pyclass(name="TickTeamAnalysis", get_all)]
#[derive(Default, Debug, Clone)]
pub struct TickTeamAnalysisPy {
    pub team: Team,
    pub medic: Option<Player>,
    pub playerdata: HashMap<u16, TickPlayerDataPy>,
}

impl<'a> TickTeamAnalysis<'a> {
    pub fn new<T, U>(team: Team, playerlist: T, _other_team: U) -> Self
    where
        T: Iterator<Item = &'a Player> + Clone,
        U: Iterator<Item = &'a Player> + Clone
    {
        let mut playerdata = HashMap::new();
        let mut medic: Option<&'a Player> = None;

        // Step 1: Generate groupings
        // TODO

        // Step 2: Find the medic & generate averages
        let mut avg_position = Vector::default();
        for (i, player) in playerlist.clone().enumerate() {
            if player.class == Class::Medic {
                medic = Some(&player);
            }

            // Rolling average to preserve floating point precision?
            avg_position = ((i-1) as f32) / (i as f32) * avg_position 
                         + (1 as f32) / (i as f32) * player.position;
        }

        // Step 3: Generate player data
        for player in playerlist {
            playerdata.insert(player.info.as_ref().unwrap().user_id, TickPlayerData {
                player: &player,
                //dist_from_group_avg: 0f32, // TODO
                time_since_last_hit: 0f32, // TODO
                dist_from_medic: match medic {Some(m) => player.distance_from(m), None=> -1f32},
                dist_from_team_avg: player.position.dist_to(&avg_position),
            });
        }

        TickTeamAnalysis {
            team,
            medic,
            playerdata
        }
    }
}

#[derive(Debug, Clone)]
pub struct TickAnalysis<'a> {
    pub tickdata: &'a TickData,
    pub redteam: Option<TickTeamAnalysis<'a>>,
    pub bluteam: Option<TickTeamAnalysis<'a>>,

}

#[pyclass(name="TickAnalysis", get_all)]
#[derive(Default, Debug, Clone)]
pub struct TickAnalysisPy {
    pub redteam: Option<TickTeamAnalysisPy>,
    pub bluteam: Option<TickTeamAnalysisPy>,

}

impl<'a> From<&'a TickData> for TickAnalysis<'a> {
    fn from(value: &'a TickData) -> Self {
        let red_iter = value.players.iter().filter(|p| p.team == Team::Red);
        let blu_iter = value.players.iter().filter(|p| p.team == Team::Blue);

        TickAnalysis {
            tickdata: value,
            redteam: Some(TickTeamAnalysis::new(Team::Red, red_iter.clone(), blu_iter.clone())),
            bluteam: Some(TickTeamAnalysis::new(Team::Blue, blu_iter, red_iter))
        }
    }
}