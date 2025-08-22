use super::Collator;
use crate::{
    history_public::ActionObservation, prob_manager::engine::models::game_state::GameData,
};

pub struct Unique;

impl Collator for Unique {
    #[inline(always)]
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        let inf = data.influence();

        (0..6)
            .filter(|&i| inf[i] > 0) // keep only eligible players
            .map(|i| ActionObservation::CollectiveChallenge {
                participants: [false; 6],
                opposing_player_id: player,
                final_actioner: i,
            })
            .collect()
    }
}
