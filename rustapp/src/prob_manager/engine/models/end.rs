use super::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone)]
pub struct End {
}
// TODO: Mate put all of these in GameState not vice versa
impl End {
    pub fn new() -> Self {
        todo!()
    }
}

impl CoupTransition for End {
    fn state_enter_update(&mut self, game_data: &mut GameData) {
        todo!()
    }
    fn state_enter_reverse(&mut self, game_data: &mut GameData) {
        todo!()
    }
    fn state_leave_update(&self, action: &crate::history_public::ActionObservation, game_data: &mut GameData) -> EngineState {
        panic!("Game has ended")
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        todo!()
    }
}