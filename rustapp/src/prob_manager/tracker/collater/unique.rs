use crate::{history_public::ActionObservation, prob_manager::engine::models::game_state::GameData};
use super::Collator;

pub struct Unique;

impl Collator for Unique {
    #[inline(always)]
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        let inf = data.influence();

        (0..6)
            .filter(|&i| inf[i] > 0) // keep only eligible players
            .map(|i| {
                ActionObservation::CollectiveChallenge {
                    participants: [false; 6],
                    opposing_player_id: i,
                    final_actioner: player,
                }
            })
            .collect()
    }
}