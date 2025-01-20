use ahash::AHashSet;
use crate::history_public::{Card, AOName, ActionObservation};
use super::coup_const::{BAG_SIZES, TOKENS, MAX_PERM_STATES};
use super::permutation_generator::gen_table_combinations;
pub struct BruteCollectiveConstraint {
    all_states: AHashSet<String>,
    public_constraints: Vec<Vec<Card>>,
}


impl BruteCollectiveConstraint {
    pub fn game_start() -> Self {
        let permutations = gen_table_combinations("AAABBBCCCDDDEEE", &vec![2, 2, 2, 2, 2, 2, 3]);
        let all_states: AHashSet<String> = permutations.into_iter().collect();
        let public_constraints: Vec<Vec<Card>> = vec![Vec::with_capacity(2); 6];
        BruteCollectiveConstraint {
            all_states,
            public_constraints,
        }
    }
    pub fn reveal(&mut self, player_id: usize, card: Card) {
        let player_indexes = match player_id {
            0 => vec![0, 1],
            1 => vec![2, 3],
            2 => vec![4, 5],
            3 => vec![6, 7],
            4 => vec![8, 9],
            5 => vec![10, 11],
            6 => vec![12, 13, 14],
            _ => panic!("Invalid player_id"),
        };
        
        let card_char = card.card_to_char();
        
        self.all_states.retain(|state| {
            player_indexes.iter()
                .any(|&index| state.chars().nth(index).unwrap() == card_char)
        });
    }
    pub fn mix(&mut self, player_i: usize, player_j: usize) {
        todo!()
    }
    pub fn death(&mut self, player_id: usize, card: Card) {
        self.public_constraints[player_id].push(card);
        self.reveal(player_id, card);
    }
    pub fn reveal_redraw(&mut self, player_id: usize, card: Card) {
        self.reveal(player_id, card)
    }
    pub fn ambassador_public() {
        todo!()
    }
    /// Returns an array indexed by [player][card] that indicates if a player can have a particular card
    /// true => impossible
    /// false => possible
    pub fn generate_one_card_impossibilities_player_card_indexing(&self) -> [[bool; 5]; 7]{
        todo!()
    }

}