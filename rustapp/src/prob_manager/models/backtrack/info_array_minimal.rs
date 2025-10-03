use super::info_array_trait::InfoArrayTrait;
use crate::{
    history_public::Card,
    prob_manager::engine::constants::{
        MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PILE, MAX_HAND_SIZE_PLAYER, MAX_NUM_PER_CARD,
        MAX_PLAYERS_INCL_PILE,
    },
};

/// Minimal InfoArray that only contains public_constraints and inferred_constraints
#[derive(Clone, Debug)]
pub struct InfoArrayMinimal {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
}

impl InfoArrayMinimal {
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
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::new(); 7];
        Self {
            public_constraints,
            inferred_constraints,
        }
    }

    pub fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self {
        debug_assert!(cards.len() < 3, "player has too many cards!");
        let mut info_array = Self::start_public();
        info_array.inferred_constraints[player].push(cards[0]);
        info_array.inferred_constraints[player].push(cards[1]);
        info_array
    }

    pub fn clone_public(&self) -> Self {
        let public_constraints: Vec<Vec<Card>> = self.public_constraints.clone();
        let inferred_constraints: Vec<Vec<Card>> = vec![Vec::new(); 7];
        Self {
            public_constraints,
            inferred_constraints,
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

    pub fn set_inferred_constraints(&mut self, inferred_constraints: &[Vec<Card>]) {
        self.inferred_constraints = inferred_constraints.to_vec();
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

impl InfoArrayTrait for InfoArrayMinimal {
    fn start_public() -> Self {
        InfoArrayMinimal::start_public()
    }

    fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self {
        InfoArrayMinimal::start_private(player, cards)
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

    fn set_inferred_constraints(&mut self, inferred_constraints: &[Vec<Card>]) {
        self.set_inferred_constraints(inferred_constraints)
    }

    fn get_impossible_constraint(&self, _player: usize, _card: usize) -> bool {
        false
    }

    fn set_impossible_constraint(&mut self, _player: usize, _card: usize, _value: bool) {
        unimplemented!()
    }

    fn set_all_impossible_constraints(&mut self, _player: usize, _value: bool) {
        unimplemented!()
    }

    fn get_impossible_constraint_2(&self, _player: usize, _card1: usize, _card2: usize) -> bool {
        false
    }

    fn set_impossible_constraint_2(
        &mut self,
        _player: usize,
        _card1: usize,
        _card2: usize,
        _value: bool,
    ) {
        unimplemented!()
    }

    fn set_all_impossible_constraints_2(&mut self, _player: usize, _value: bool) {
        unimplemented!()
    }

    fn get_impossible_constraint_3(&self, _card1: usize, _card2: usize, _card3: usize) -> bool {
        false
    }

    fn set_impossible_constraint_3(
        &mut self,
        _card1: usize,
        _card2: usize,
        _card3: usize,
        _value: bool,
    ) {
        unimplemented!()
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
        unimplemented!()
    }

    fn format_impossible_constraints_2(&self) -> String {
        unimplemented!()
    }

    fn format_impossible_constraints_3(&self) -> String {
        unimplemented!()
    }

    fn impossible_constraints(&self) -> [[bool; 5]; 7] {
        unimplemented!()
    }

    fn impossible_constraints_paired(&self) -> [[[bool; 5]; 5]; 7] {
        unimplemented!()
    }

    fn impossible_constraints_triple(&self) -> [[[bool; 5]; 5]; 5] {
        unimplemented!()
    }

    fn count_possible_single_constraints(&self, _player: usize) -> u8 {
        unimplemented!()
    }

    fn find_only_possible_single_constraint(&self, _player: usize) -> Option<usize> {
        unimplemented!()
    }

    fn all_cards_dead(&self, card: Card) -> bool {
        self.public_constraints
            .iter()
            .map(|v| v.iter().filter(|&&c| c == card).count())
            .sum::<usize>()
            >= MAX_NUM_PER_CARD as usize
    }
}
