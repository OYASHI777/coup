use super::engine_state::CoupTransition;
use crate::history_public::ActionObservation;
use super::engine_state::EngineState;
use super::game_state::GameData;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct End {
}

impl CoupTransition for End {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(&self, _action: &ActionObservation, _game_data: &mut GameData) -> EngineState {
        unimplemented!("Game has ended! Cannot leave this state!")
    }

    fn state_leave_reverse(&self, _action: &ActionObservation, _game_data: &mut GameData) {
        unimplemented!("Game has ended! You should not have called this at all!")
    }
}