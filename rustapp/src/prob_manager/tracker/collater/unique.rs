use super::Collator;
use crate::{
    history_public::ActionObservation,
    prob_manager::engine::{constants::MAX_PLAYERS_EXCL_PILE, models::game_state::GameData},
};

/// Returns a `CollectiveChallenge` for each eligible player
pub struct Unique;

impl Collator for Unique {
    #[inline(always)]
    fn challenge(player: usize, data: &GameData) -> Vec<crate::history_public::ActionObservation> {
        let inf = data.influence();

        (0..MAX_PLAYERS_EXCL_PILE)
            .filter(|&i| inf[i] > 0) // keep only eligible players
            .map(|i| ActionObservation::CollectiveChallenge {
                participants: [false; MAX_PLAYERS_EXCL_PILE],
                opposing_player_id: player,
                final_actioner: i,
            })
            .collect()
    }

    fn block(player: usize, data: &GameData) -> Vec<ActionObservation> {
        let inf = data.influence();
        (0..MAX_PLAYERS_EXCL_PILE)
            .filter(|&i| inf[i] > 0) // keep only eligible players
            .map(|i| ActionObservation::CollectiveBlock {
                participants: [false; MAX_PLAYERS_EXCL_PILE],
                opposing_player_id: player,
                final_actioner: i,
            })
            .collect()
    }
}
