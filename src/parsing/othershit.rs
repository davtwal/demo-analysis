

#[derive(Debug)]
pub enum RelevantEvent {
    Kill(Kill),
}


// The state of the game at any given tick.
#[derive(Default, Debug)]
pub struct TickState {
    pub players: Vec<Player>,
    pub buildings: BTreeMap<EntityId, Building>,
    pub relev_events: Vec<RelevantEvent>,
}

impl TickState {
    pub fn new() -> Self {
        Self::default()
    }


    pub fn get_or_create_player(&mut self, entity_id: EntityId) -> &mut Player {
        // ...
    }


    pub fn get_or_create_building(...) {
        // ...
    }


    pub fn remove_building(...) {
        // ...
    }
}


#[derive(Default, Debug)]
pub struct StateMap(BTreeMap<DemoTick, TickState>);

impl IntoIterator for StateMap {
    type Item = (DemoTick, TickState);
    type IntoIter = std::collections::btree_map::IntoIter<DemoTick, TickState>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

use tf_demo_parser::demo::parser::gamestateanalyser::{GameState, GameStateAnalyser};

// The reason behind this ""renaming"" is that the data stored in the game state analyser is EXACTLY what we want, but we want it for EVERY TICK. 
#[derive(Default, Debug)]
pub struct GameData(GameStateAnalyser);

// The game analyser contains all of the data that we care about.
// As the demo parser runs through the demo, messages will get passed to
// our analyser for us to analyse. 
//
//
#[derive(Default, Debug)]
pub struct GameAnalyser {
    pub data: GameData,


}


impl GameAnalyser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grab_all_relevant_events(self: &Self) -> BTreeMap<DemoTick, Vec<RelevantEvent>> {
        let mut ret_map = BTreeMap::<DemoTick, Vec<RelevantEvent>>::new();
        for (tick, state) in self.states {
            if state.relev_events.len() > 0 {
                ret_map.insert(tick, state.relev_events);
            }
        }


        ret_map // return
    }
}

impl BorrowMessageHandler for GameAnalyser {
    fn borrow_output(&self, state: &ParserState) -> &Self::Output {
        &self.states
    }
}

impl MessageHandler for GameAnalyser {
    type Output = <StateMap as IntoIterator>::Item;

    fn does_handle(message_type: tf_demo_parser::MessageType) -> bool {
        
    }

    fn handle_data_tables(
            &mut self,
            _tables: &[tf_demo_parser::demo::packet::datatable::ParseSendTable],
            _server_classes: &[tf_demo_parser::demo::packet::datatable::ServerClass],
            _parser_state: &tf_demo_parser::ParserState,
        ) {
        
    }

    fn handle_header(&mut self, _header: &tf_demo_parser::demo::header::Header) {
        
    }

    fn handle_packet_meta(&mut self, tick: DemoTick,
        _meta: &MessagePacketMeta, _parser_state: &ParserState
    ) {
        &mut tickstate = self.states.insert(tick, TickState::new());
        self.tick = tick;
    }

    fn handle_string_entry(
        &mut self,
        _table: &str,
        _index: usize,
        _entries: &tf_demo_parser::demo::packet::stringtable::StringTableEntry,
        _parser_state: &tf_demo_parser::ParserState,
    ) {
        
    }

    fn handle_message(&mut self, message: &Message, tick: DemoTick, parser_state: &ParserState) {
        // ...
    }

    fn into_output(self, state: &tf_demo_parser::ParserState) -> Self::Output {
        
    }
    //and a couple others
}


impl GameAnalyser {
    pub fn handle_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        // forward to a different handler based on type of entity
    }


    pub fn
}



