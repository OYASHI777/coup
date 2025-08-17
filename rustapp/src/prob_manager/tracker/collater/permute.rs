use crate::prob_manager::engine::models::game_state::GameData;
use super::Collator;

pub struct Permute;

impl Collator for Permute {
    fn challenge(_player: usize, _data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}