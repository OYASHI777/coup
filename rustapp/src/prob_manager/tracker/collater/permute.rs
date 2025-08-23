use crate::prob_manager::engine::models::game_state::GameData;
use super::Collator;

/// Returns a `CollectiveChallenge` for each permutation of participants for all eligible players
pub struct Permute;

impl Collator for Permute {
    fn challenge(_player: usize, _data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
    
    fn block(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}