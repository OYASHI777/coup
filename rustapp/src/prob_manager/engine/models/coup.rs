use super::end::End;
use super::engine_state::CoupTransition;
use super::engine_state::EngineState;
use super::game_state::GameData;
use super::turn_start::TurnStart;
use crate::history_public::ActionObservation;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CoupHit {
    pub player_turn: usize,
    pub player_hit: usize,
}

impl CoupTransition for CoupHit {
    fn state_enter_update(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_enter_reverse(&mut self, _game_data: &mut GameData) {
        // nothing
    }
    fn state_leave_update(
        &self,
        action: &ActionObservation,
        game_data: &mut GameData,
    ) -> EngineState {
        match action {
            ActionObservation::Discard {
                player_id,
                no_cards,
                ..
            } => match game_data.game_will_be_won(*player_id, *no_cards as u8) {
                true => EngineState::End(End {}),
                false => EngineState::TurnStart(TurnStart {
                    player_turn: self.player_turn,
                }),
            },
            _ => {
                panic!("Illegal Move");
            }
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, _game_data: &mut GameData) {
        debug_assert!(
            matches!(action, ActionObservation::Discard { .. }),
            "Illegal Move!"
        )
    }
}
