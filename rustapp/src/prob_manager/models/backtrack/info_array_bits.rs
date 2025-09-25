use super::{
    info_array_trait::InfoArrayTrait, ImpossibleField1, ImpossibleField2, ImpossibleField3,
};
use crate::{
    history_public::Card,
    prob_manager::engine::constants::{
        MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PILE, MAX_HAND_SIZE_PLAYER, MAX_NUM_PER_CARD,
        MAX_PLAYERS_INCL_PILE,
    },
};

/// For each player store an array of bool where each index is a Card, this represents whether a player cannot have a card true => cannot
#[derive(Clone, Debug)]
pub struct InfoArrayBits {
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    pub impossible_constraints: [ImpossibleField1; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_2: [ImpossibleField2; MAX_PLAYERS_INCL_PILE],
    pub impossible_constraints_3: ImpossibleField3,
}

impl InfoArrayBits {
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
        let impossible_constraints: [ImpossibleField1; MAX_PLAYERS_INCL_PILE] =
            [ImpossibleField1::zero(); MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_2: [ImpossibleField2; MAX_PLAYERS_INCL_PILE] =
            [ImpossibleField2::zero(); MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_3 = ImpossibleField3::zero();
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

        let mut impossible_constraints = [ImpossibleField1::zero(); MAX_PLAYERS_INCL_PILE];
        // Set all cards impossible for this player, then make known cards possible
        for card in 0..MAX_CARD_PERMS_ONE {
            impossible_constraints[player].set(card as u8, true);
        }
        impossible_constraints[player].set(cards[0] as u8, false);
        impossible_constraints[player].set(cards[1] as u8, false);

        let mut impossible_constraints_2 = [ImpossibleField2::zero(); MAX_PLAYERS_INCL_PILE];
        // Set all card pairs impossible for this player, then make the actual pair possible
        for card_a in 0..MAX_CARD_PERMS_ONE {
            for card_b in 0..MAX_CARD_PERMS_ONE {
                impossible_constraints_2[player].set(card_a as u8, card_b as u8, true);
            }
        }
        impossible_constraints_2[player].set(cards[0] as u8, cards[1] as u8, false);

        let mut impossible_constraints_3 = ImpossibleField3::zero();
        // Set some impossible triple constraints based on known cards
        impossible_constraints_3.set(cards[0] as u8, cards[0] as u8, cards[0] as u8, true);
        impossible_constraints_3.set(cards[1] as u8, cards[1] as u8, cards[1] as u8, true);
        if cards[0] == cards[1] {
            // If player has duplicate cards, make more constraints impossible
            for c in 0..MAX_CARD_PERMS_ONE {
                impossible_constraints_3.set(cards[0] as u8, cards[0] as u8, c as u8, true);
            }
        }

        Self {
            public_constraints,
            inferred_constraints,
            impossible_constraints,
            impossible_constraints_2,
            impossible_constraints_3,
        }
    }

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
        let impossible_constraints: [ImpossibleField1; MAX_PLAYERS_INCL_PILE] =
            [ImpossibleField1::zero(); MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_2: [ImpossibleField2; MAX_PLAYERS_INCL_PILE] =
            [ImpossibleField2::zero(); MAX_PLAYERS_INCL_PILE];
        let impossible_constraints_3 = ImpossibleField3::zero();
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

impl InfoArrayTrait for InfoArrayBits {
    fn start_public() -> Self {
        InfoArrayBits::start_public()
    }

    fn start_private(player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) -> Self {
        InfoArrayBits::start_private(player, cards)
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

    fn get_impossible_constraint(&self, player: usize, card: usize) -> bool {
        self.impossible_constraints[player].get(card as u8)
    }

    fn set_impossible_constraint(&mut self, player: usize, card: usize, value: bool) {
        self.impossible_constraints[player].set(card as u8, value);
    }

    fn set_all_impossible_constraints(&mut self, player: usize, value: bool) {
        for card in 0..MAX_CARD_PERMS_ONE {
            self.impossible_constraints[player].set(card as u8, value);
        }
    }

    fn get_impossible_constraint_2(&self, player: usize, card1: usize, card2: usize) -> bool {
        self.impossible_constraints_2[player].get(card1 as u8, card2 as u8)
    }

    fn set_impossible_constraint_2(
        &mut self,
        player: usize,
        card1: usize,
        card2: usize,
        value: bool,
    ) {
        self.impossible_constraints_2[player].set(card1 as u8, card2 as u8, value);
    }

    fn set_all_impossible_constraints_2(&mut self, player: usize, value: bool) {
        for card1 in 0..MAX_CARD_PERMS_ONE {
            for card2 in 0..MAX_CARD_PERMS_ONE {
                self.impossible_constraints_2[player].set(card1 as u8, card2 as u8, value);
            }
        }
    }

    fn get_impossible_constraint_3(&self, card1: usize, card2: usize, card3: usize) -> bool {
        self.impossible_constraints_3
            .get(card1 as u8, card2 as u8, card3 as u8)
    }

    fn set_impossible_constraint_3(
        &mut self,
        card1: usize,
        card2: usize,
        card3: usize,
        value: bool,
    ) {
        self.impossible_constraints_3
            .set(card1 as u8, card2 as u8, card3 as u8, value);
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
        let mut result = [[false; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        for (player, player_result) in result.iter_mut().enumerate() {
            for (card, card_result) in player_result.iter_mut().enumerate() {
                *card_result = self.impossible_constraints[player].get(card as u8);
            }
        }
        result
    }

    fn impossible_constraints_paired(&self) -> [[[bool; 5]; 5]; 7] {
        let mut result = [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE];
        for (player, player_result) in result.iter_mut().enumerate() {
            for (card1, card1_result) in player_result.iter_mut().enumerate() {
                for (card2, card2_result) in card1_result.iter_mut().enumerate() {
                    *card2_result =
                        self.impossible_constraints_2[player].get(card1 as u8, card2 as u8);
                }
            }
        }
        result
    }

    fn impossible_constraints_triple(&self) -> [[[bool; 5]; 5]; 5] {
        let mut result = [[[false; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE];
        for (card1, card1_result) in result.iter_mut().enumerate() {
            for (card2, card2_result) in card1_result.iter_mut().enumerate() {
                for (card3, card3_result) in card2_result.iter_mut().enumerate() {
                    *card3_result =
                        self.impossible_constraints_3
                            .get(card1 as u8, card2 as u8, card3 as u8);
                }
            }
        }
        result
    }

    fn count_possible_single_constraints(&self, player: usize) -> u8 {
        (0..MAX_CARD_PERMS_ONE)
            .map(|card| !self.impossible_constraints[player].get(card as u8) as u8)
            .sum::<u8>()
    }

    fn find_only_possible_single_constraint(&self, player: usize) -> Option<usize> {
        (0..MAX_CARD_PERMS_ONE).find(|&card| !self.impossible_constraints[player].get(card as u8))
    }

    fn all_cards_dead(&self, card: Card) -> bool {
        self.public_constraints
            .iter()
            .map(|v| v.iter().filter(|&&c| c == card).count())
            .sum::<usize>()
            >= MAX_NUM_PER_CARD as usize
    }
}
