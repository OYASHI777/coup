use super::Collator;
use crate::{
    history_public::ActionObservation, prob_manager::engine::models::game_state::GameData,
};
/// Indicates people eligible to challenge
pub struct Indicate;

impl Collator for Indicate {
    #[inline(always)]
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        let participants = std::array::from_fn(|p| data.influence()[p] > 0);
        vec![ActionObservation::CollectiveChallenge {
            participants,
            opposing_player_id: player,
            final_actioner: player,
        }]
    }

    fn block(player: usize, data: &GameData) -> Vec<ActionObservation> {
        let participants = std::array::from_fn(|p| data.influence()[p] > 0);
        vec![ActionObservation::CollectiveBlock {
            participants,
            opposing_player_id: player,
            final_actioner: player,
        }]
    }
}
