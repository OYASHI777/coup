use crate::{history_public::ActionObservation, prob_manager::engine::models::game_state::GameData};
use super::Collator;

pub struct Permute;

impl Collator for Permute {
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}