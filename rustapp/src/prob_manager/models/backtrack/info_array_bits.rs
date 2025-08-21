use crate::history_public::Card;
use super::{ImpossibleField2, ImpossibleField3};

/// For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
#[derive(Clone, Debug)]
pub struct InfoArrayBits {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [[bool; 5]; 7],
    pub impossible_constraints_2: [ImpossibleField2; 7],
    pub impossible_constraints_3: ImpossibleField3,
}