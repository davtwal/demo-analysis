#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
#[repr(u8)]
pub enum GroupingType {
    #[default]
    None = 0,           // Not valid
	Isolated = 1,   	// (1)  Only one player.
	IsolatedCombo = 2,  // (2)  Contains the medic and one other players.
	Combo = 3,      	// (3+) Contains the medic and at least two other players.
	Flank = 4,      	// (2+) Does not contain the medic, but has at least two players.
}

#[derive(Default, Debug, Clone)]
pub struct TickPlayerGrouping<'a> {
    pub group_type: GroupingType,
    pub players: Vec<&'a Player>,
    pub avg_pos: Vector,
}
