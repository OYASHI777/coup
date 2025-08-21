use crate::history_public::Card;
/// For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
#[derive(Clone, Debug)]
pub struct InfoArray {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [[bool; 5]; 7],
    pub impossible_constraints_2: [[[bool; 5]; 5]; 7],
    pub impossible_constraints_3: [[[bool; 5]; 5]; 5],
}

impl InfoArray {
    pub fn initial() -> Self {
        Self::start_public()
    }
    pub fn start_public() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::new(),
        ];
        let inferred_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(3),
        ];
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
        let impossible_constraints_2: [[[bool; 5]; 5]; 7] = [[[false; 5]; 5]; 7];
        let impossible_constraints_3: [[[bool; 5]; 5]; 5] = [[[false; 5]; 5]; 5];
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }
    pub fn start_private(player: usize, cards: &[Card; 2]) -> Self {
        debug_assert!(cards.len() < 3, "player has too many cards!");
        let public_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::new(),
        ];
        let mut inferred_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(3),
        ];
        inferred_constraints[player].push(cards[0]);
        inferred_constraints[player].push(cards[1]);
        // Start takes the inferred information discovered via a pathdependent lookback
        let mut impossible_constraints = [[false; 5]; 7];
        impossible_constraints[player] = [true; 5];
        impossible_constraints[player][cards[0] as usize] = false;
        impossible_constraints[player][cards[1] as usize] = false;
        let mut impossible_constraints_2 = [[[false; 5]; 5]; 7];
        impossible_constraints_2[player] = [[true; 5]; 5];
        let mut impossible_constraints_3 = [[[false; 5]; 5]; 5];
        impossible_constraints_3[cards[0] as usize][cards[0] as usize][cards[0] as usize] = true;
        impossible_constraints_3[cards[1] as usize][cards[1] as usize][cards[1] as usize] = true;
        if cards[0] == cards[1] {
            // update impossible_2
            for p in 0..7 {
                impossible_constraints_2[p][cards[0] as usize][cards[0] as usize] = true;
            }
            // update impossible_3 where more than 2
            for c in 0..5 {
                impossible_constraints_3[cards[0] as usize][cards[0] as usize][c] = true;
                impossible_constraints_3[cards[0] as usize][c][cards[0] as usize] = true;
                impossible_constraints_3[c][cards[0] as usize][cards[0] as usize] = true;
            }
        }
        impossible_constraints_2[player][cards[0] as usize][cards[1] as usize] = false;
        impossible_constraints_2[player][cards[1] as usize][cards[0] as usize] = false;
        // StartInferred takes the inferred information from start, and runs add_inferred_information
        // This seperation prevents handling cases where you add discovered information that is already inside due to add_inferred_information
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }
    /// Clones meta_data but only with public data copied
    pub fn clone_public(&self) -> Self {
        let public_constraints: Vec<Vec<Card>> = self.public_constraints.clone();
        let inferred_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(2),
            Vec::with_capacity(3),
        ];
        let impossible_constraints: [[bool; 5]; 7] = [[false; 5]; 7];
        let impossible_constraints_2: [[[bool; 5]; 5]; 7] = [[[false; 5]; 5]; 7];
        let impossible_constraints_3: [[[bool; 5]; 5]; 5] = [[[false; 5]; 5]; 5];
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }
    pub fn public_constraints(&self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }
    pub fn public_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.public_constraints
    }
    pub fn sort_public_constraints(&mut self) {
        self.public_constraints
            .iter_mut()
            .for_each(|v| v.sort_unstable());
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }
    pub fn inferred_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.inferred_constraints
    }
    pub fn sort_inferred_constraints(&mut self) {
        self.inferred_constraints
            .iter_mut()
            .for_each(|v| v.sort_unstable());
    }
    pub fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>) {
        self.inferred_constraints = inferred_constraints.clone();
    }
    pub fn impossible_constraints(&self) -> &[[bool; 5]; 7] {
        &self.impossible_constraints
    }
    pub fn impossible_constraints_mut(&mut self) -> &mut [[bool; 5]; 7] {
        &mut self.impossible_constraints
    }
    pub fn impossible_constraints_2(&self) -> &[[[bool; 5]; 5]; 7] {
        &self.impossible_constraints_2
    }
    pub fn impossible_constraints_2_mut(&mut self) -> &mut [[[bool; 5]; 5]; 7] {
        &mut self.impossible_constraints_2
    }
    pub fn impossible_constraints_3(&self) -> &[[[bool; 5]; 5]; 5] {
        &self.impossible_constraints_3
    }
    pub fn impossible_constraints_3_mut(&mut self) -> &mut [[[bool; 5]; 5]; 5] {
        &mut self.impossible_constraints_3
    }
    /// Changes stored impossible_constraints
    pub fn set_impossible_constraints(&mut self, impossible_constraints: &[[bool; 5]; 7]) {
        self.impossible_constraints = impossible_constraints.clone();
    }
    pub fn player_cards_known<T>(&self, player_id: T) -> usize
    where
        T: Into<usize> + Copy,
    {
        self.public_constraints[player_id.into()].len()
            + self.inferred_constraints[player_id.into()].len()
    }
    pub fn player_has_public_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.public_constraints[player_id.into()].contains(&card)
    }
    pub fn player_has_inferred_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.inferred_constraints[player_id.into()].contains(&card)
    }
    pub fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.player_cards_known(player_id) == 2
            && self.inferred_constraints[player_id.into()]
                .iter()
                .all(|&c| c == card)
            && self.public_constraints[player_id.into()]
                .iter()
                .all(|&c| c == card)
    }
}
