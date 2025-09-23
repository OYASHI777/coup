use super::info_array_trait::InfoArrayTrait;
use crate::{
    history_public::Card,
    prob_manager::engine::constants::{
        MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PILE, MAX_HAND_SIZE_PLAYER, MAX_NUM_PER_CARD,
        MAX_PLAYERS_INCL_PILE,
    },
};
/// For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
#[derive(Clone, Debug)]
pub struct InfoArray {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_2:
        [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_3:
        [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE],
}

impl InfoArray {
    pub fn initial() -> Self {
        Self::start_public()
    }
    pub fn start_public() -> Self {
        let public_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::new(),
        ];
        let inferred_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PILE),
        ];
        let impossible_constraints: [[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] =
            [[false; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_2: [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
            MAX_PLAYERS_INCL_PILE] =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_3: [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
            MAX_CARD_PERMS_ONE] =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }
    pub fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self {
        debug_assert!(cards.len() < 3, "player has too many cards!");
        let public_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::new(),
        ];
        let mut inferred_constraints: Vec<Vec<Card>> = vec![
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PILE),
        ];
        inferred_constraints[player].push(cards[0]);
        inferred_constraints[player].push(cards[1]);
        // Start takes the inferred information discovered via a pathdependent lookback
        let mut impossible_constraints = [[false; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        impossible_constraints[player] = [true; MAX_CARD_PERMS_ONE];
        impossible_constraints[player][cards[0] as usize] = false;
        impossible_constraints[player][cards[1] as usize] = false;
        let mut impossible_constraints_2 =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        impossible_constraints_2[player] = [[true; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
        let mut impossible_constraints_3 =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
        impossible_constraints_3[cards[0] as usize][cards[0] as usize][cards[0] as usize] = true;
        impossible_constraints_3[cards[1] as usize][cards[1] as usize][cards[1] as usize] = true;
        if cards[0] == cards[1] {
            // update impossible_2
            for p in 0..MAX_PLAYERS_INCL_PILE {
                impossible_constraints_2[p][cards[0] as usize][cards[0] as usize] = true;
            }
            // update impossible_3 where more than 2
            for c in 0..MAX_CARD_PERMS_ONE {
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
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PLAYER),
            Vec::with_capacity(MAX_HAND_SIZE_PILE),
        ];
        let impossible_constraints: [[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] =
            [[false; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_2: [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
            MAX_PLAYERS_INCL_PILE] =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_3: [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
            MAX_CARD_PERMS_ONE] =
            [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
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
    pub fn impossible_constraints(&self) -> &[[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        &self.impossible_constraints
    }
    pub fn impossible_constraints_mut(
        &mut self,
    ) -> &mut [[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        &mut self.impossible_constraints
    }
    pub fn impossible_constraints_2(
        &self,
    ) -> &[[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        &self.impossible_constraints_2
    }
    pub fn impossible_constraints_2_mut(
        &mut self,
    ) -> &mut [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        &mut self.impossible_constraints_2
    }
    pub fn impossible_constraints_3(
        &self,
    ) -> &[[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE] {
        &self.impossible_constraints_3
    }
    pub fn impossible_constraints_3_mut(
        &mut self,
    ) -> &mut [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE] {
        &mut self.impossible_constraints_3
    }
    /// Changes stored impossible_constraints
    pub fn set_impossible_constraints(
        &mut self,
        impossible_constraints: &[[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE],
    ) {
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
        self.player_cards_known(player_id) == MAX_HAND_SIZE_PLAYER
            && self.inferred_constraints[player_id.into()]
                .iter()
                .all(|&c| c == card)
            && self.public_constraints[player_id.into()]
                .iter()
                .all(|&c| c == card)
    }
}

impl InfoArrayTrait for InfoArray {
    fn start_public() -> Self {
        InfoArray::start_public()
    }

    fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self {
        InfoArray::start_private(player, cards)
    }

    fn clone_public(&self) -> Self {
        self.clone_public()
    }

    fn public_constraints(&self) -> &Vec<Vec<Card>> {
        &self.public_constraints
    }

    fn public_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.public_constraints
    }

    fn sort_public_constraints(&mut self) {
        self.sort_public_constraints()
    }

    fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        &self.inferred_constraints
    }

    fn inferred_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        &mut self.inferred_constraints
    }

    fn sort_inferred_constraints(&mut self) {
        self.sort_inferred_constraints()
    }

    fn set_inferred_constraints(&mut self, inferred_constraints: &Vec<Vec<Card>>) {
        self.set_inferred_constraints(inferred_constraints)
    }

    fn get_impossible_constraint(&self, player: usize, card: usize) -> bool {
        self.impossible_constraints[player][card]
    }

    fn set_impossible_constraint(&mut self, player: usize, card: usize, value: bool) {
        self.impossible_constraints[player][card] = value;
    }

    fn set_all_impossible_constraints(&mut self, player: usize, value: bool) {
        self.impossible_constraints[player] = [value; MAX_CARD_PERMS_ONE];
    }

    fn get_impossible_constraint_2(&self, player: usize, card1: usize, card2: usize) -> bool {
        self.impossible_constraints_2[player][card1][card2]
    }

    fn set_impossible_constraint_2(
        &mut self,
        player: usize,
        card1: usize,
        card2: usize,
        value: bool,
    ) {
        self.impossible_constraints_2[player][card1][card2] = value;
        self.impossible_constraints_2[player][card2][card1] = value;
    }

    fn set_all_impossible_constraints_2(&mut self, player: usize, value: bool) {
        self.impossible_constraints_2[player] = [[value; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
    }

    fn get_impossible_constraint_3(&self, card1: usize, card2: usize, card3: usize) -> bool {
        self.impossible_constraints_3[card1][card2][card3]
    }

    fn set_impossible_constraint_3(
        &mut self,
        card1: usize,
        card2: usize,
        card3: usize,
        value: bool,
    ) {
        self.impossible_constraints_3[card1][card2][card3] = value;
        self.impossible_constraints_3[card1][card3][card2] = value;
        self.impossible_constraints_3[card2][card1][card3] = value;
        self.impossible_constraints_3[card2][card3][card1] = value;
        self.impossible_constraints_3[card3][card2][card1] = value;
        self.impossible_constraints_3[card3][card1][card2] = value;
    }

    fn player_cards_known<T>(&self, player_id: T) -> usize
    where
        T: Into<usize> + Copy,
    {
        self.player_cards_known(player_id)
    }

    fn player_has_public_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.player_has_public_constraint(player_id, card)
    }

    fn player_has_inferred_constraint<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.player_has_inferred_constraint(player_id, card)
    }

    fn player_constraints_all_full<T>(&self, player_id: T, card: Card) -> bool
    where
        T: Into<usize> + Copy,
    {
        self.player_constraints_all_full(player_id, card)
    }

    fn format_impossible_constraints(&self) -> String {
        format!("{:?}", self.impossible_constraints)
    }

    fn format_impossible_constraints_2(&self) -> String {
        format!("{:?}", self.impossible_constraints_2)
    }

    fn format_impossible_constraints_3(&self) -> String {
        format!("{:?}", self.impossible_constraints_3)
    }

    fn impossible_constraints(&self) -> [[bool; 5]; 7] {
        self.impossible_constraints
    }

    fn impossible_constraints_paired(&self) -> [[[bool; 5]; 5]; 7] {
        self.impossible_constraints_2
    }

    fn impossible_constraints_triple(&self) -> [[[bool; 5]; 5]; 5] {
        self.impossible_constraints_3
    }

    fn count_possible_single_constraints(&self, player: usize) -> u8 {
        self.impossible_constraints[player]
            .iter()
            .map(|b| !*b as u8)
            .sum::<u8>()
    }

    fn find_only_possible_single_constraint(&self, player: usize) -> Option<usize> {
        self.impossible_constraints[player].iter().position(|b| !*b)
    }

    fn all_cards_dead(&self, card: Card) -> bool {
        self.public_constraints
            .iter()
            .map(|v| v.iter().filter(|&&c| c == card).count())
            .sum::<usize>()
            >= MAX_NUM_PER_CARD as usize
    }
}
