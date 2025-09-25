use crate::{
    history_public::ActionObservation, prob_manager::engine::models::game_state::GameData,
};

pub mod indicate;
pub mod permute;
pub mod unique;

pub use indicate::Indicate;
pub use permute::Permute;
pub use unique::Unique;

/// Generates suggested moves based for challenges
///
/// Options:
///
/// 1) `Indicate`
///    Indicates people eligible to challenge
///    Returns `CollectiveChallenge { participants: [true, true, true, true, true, true], opposing_player_id: player, final_actioner: player }`
/// 2) `Unique`
///    Returns a `CollectiveChallenge` for each eligible player
///    In the same example in (1), this would return 6 different `CollectiveChallenge`
/// 3) `Permute`
///    Returns a `CollectiveChallenge` for each permutation of participants for all eligible players
///    In the same example in (1), it would return 2^6 different `CollectiveChallenge`, as there are 6 eligible players
pub trait Collator {
    /// Returns a list of ActionObservation based on way
    /// challenges are handled
    fn challenge(player: usize, data: &GameData) -> Vec<ActionObservation>;
    /// Returns a list of ActionObservation based on way
    /// blocks are handled
    fn block(player: usize, data: &GameData) -> Vec<ActionObservation>;
}
