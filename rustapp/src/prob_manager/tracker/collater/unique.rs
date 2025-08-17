use crate::{history_public::ActionObservation, prob_manager::engine::models::game_state::GameData};
use super::Collator;

pub struct Unique;

impl Collator for Unique {
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}