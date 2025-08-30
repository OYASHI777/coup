use crate::{history_public::Card, prob_manager::engine::constants::MAX_PLAYERS_INCL_PILE};
use super::{ImpossibleField1, ImpossibleField2, ImpossibleField3};

/// For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
#[derive(Clone, Debug)]
pub struct InfoArrayBits {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [ImpossibleField1; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_2: [ImpossibleField2; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_3: ImpossibleField3,
}