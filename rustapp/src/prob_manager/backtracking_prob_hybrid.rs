// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

// TODO: REFACTOR ActionInfo and ActionInfoName to BacktrackManager or its own file
use super::constants::MAX_GAME_LENGTH;
use super::models::backtrack::{ActionInfo, ActionInfoName};
use super::move_guard::MoveGuard;
use crate::prob_manager::models::backtrack::InfoArrayTrait;
use crate::traits::prob_manager::coup_analysis::CoupPossibilityAnalysis;
use crate::{
    history_public::{ActionObservation, Card},
    prob_manager::engine::constants::{
        INDEX_PILE, MAX_CARDS_DISCARD, MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PILE,
        MAX_HAND_SIZE_PLAYER, MAX_NUM_PER_CARD, MAX_PLAYERS_INCL_PILE,
    },
    traits::prob_manager::coup_analysis::CoupTraversal,
};

#[derive(Clone, Debug)]
pub struct SignificantAction<T: InfoArrayTrait> {
    player: u8,
    action_info: ActionInfo,
    meta_data: T,
}
// TODO: Implement analysis for SignificantAction
impl<T: InfoArrayTrait> SignificantAction<T> {
    pub fn new(player: u8, action_info: ActionInfo, meta_data: T) -> Self {
        Self {
            player,
            action_info,
            meta_data,
        }
    }
    pub fn name(&self) -> ActionInfoName {
        self.action_info.name()
    }
    pub fn player(&self) -> u8 {
        self.player
    }
    pub fn action_info(&self) -> &ActionInfo {
        &self.action_info
    }
    pub fn action_info_mut(&mut self) -> &mut ActionInfo {
        &mut self.action_info
    }
    pub fn meta_data(&self) -> &T {
        &self.meta_data
    }
    pub fn public_constraints(&self) -> &Vec<Vec<Card>> {
        self.meta_data.public_constraints()
    }
    pub fn inferred_constraints(&self) -> &Vec<Vec<Card>> {
        self.meta_data.inferred_constraints()
    }
    pub fn inferred_constraints_mut(&mut self) -> &mut Vec<Vec<Card>> {
        self.meta_data.inferred_constraints_mut()
    }
    pub fn set_inferred_constraints(&mut self, inferred_constraints: &[Vec<Card>]) {
        self.meta_data
            .set_inferred_constraints(inferred_constraints)
    }
    pub fn get_impossible_constraint(&self, player: usize, card: usize) -> bool {
        self.meta_data.get_impossible_constraint(player, card)
    }
    pub fn set_impossible_constraint(&mut self, player: usize, card: usize, value: bool) {
        self.meta_data
            .set_impossible_constraint(player, card, value);
    }
    pub fn set_all_impossible_constraints(&mut self, player: usize, value: bool) {
        self.meta_data.set_all_impossible_constraints(player, value);
    }
    pub fn get_impossible_constraint_2(&self, player: usize, card1: usize, card2: usize) -> bool {
        self.meta_data
            .get_impossible_constraint_2(player, card1, card2)
    }
    pub fn set_impossible_constraint_2(
        &mut self,
        player: usize,
        card1: usize,
        card2: usize,
        value: bool,
    ) {
        self.meta_data
            .set_impossible_constraint_2(player, card1, card2, value);
    }
    pub fn set_all_impossible_constraints_2(&mut self, player: usize, value: bool) {
        self.meta_data
            .set_all_impossible_constraints_2(player, value);
    }
    pub fn get_impossible_constraint_3(&self, card1: usize, card2: usize, card3: usize) -> bool {
        self.meta_data
            .get_impossible_constraint_3(card1, card2, card3)
    }
    pub fn set_impossible_constraint_3(
        &mut self,
        card1: usize,
        card2: usize,
        card3: usize,
        value: bool,
    ) {
        self.meta_data
            .set_impossible_constraint_3(card1, card2, card3, value);
    }
    pub fn add_inferred_constraints(&mut self, player_id: usize, card: Card) {
        self.meta_data.inferred_constraints_mut()[player_id].push(card);
        debug_assert!(
            player_id < INDEX_PILE && self.meta_data.inferred_constraints()[player_id].len() < 3
                || player_id == INDEX_PILE
                    && self.meta_data.inferred_constraints()[player_id].len() < 4,
            "bad push"
        );
    }
    pub fn check_add_inferred_constraints(&mut self, player_id: usize, card: Card) -> bool {
        if !self.meta_data.inferred_constraints()[player_id].contains(&card) {
            self.meta_data.inferred_constraints_mut()[player_id].push(card);
            debug_assert!(
                player_id < INDEX_PILE
                    && self.meta_data.inferred_constraints()[player_id].len()
                        <= MAX_HAND_SIZE_PLAYER
                    || player_id == INDEX_PILE
                        && self.meta_data.inferred_constraints()[INDEX_PILE].len()
                            <= MAX_HAND_SIZE_PILE,
                "bad push"
            );
            return true;
        }
        false
    }
    pub fn player_cards_known<U>(&self, player_id: U) -> usize
    where
        U: Into<usize> + Copy,
    {
        self.meta_data.player_cards_known(player_id)
    }
    pub fn player_has_public_constraint<U>(&self, player_id: U, card: Card) -> bool
    where
        U: Into<usize> + Copy,
    {
        self.meta_data.player_has_public_constraint(player_id, card)
    }
    pub fn player_has_inferred_constraint<U>(&self, player_id: U, card: Card) -> bool
    where
        U: Into<usize> + Copy,
    {
        self.meta_data
            .player_has_inferred_constraint(player_id, card)
    }
    pub fn player_constraints_all_full<U>(&self, player_id: U, card: Card) -> bool
    where
        U: Into<usize> + Copy,
    {
        self.meta_data.player_constraints_all_full(player_id, card)
    }
    pub fn known_card_count(&self, card: Card) -> u8 {
        self.meta_data
            .inferred_constraints()
            .iter()
            .map(|v| v.iter().filter(|c| **c == card).count() as u8)
            .sum::<u8>()
            + self
                .meta_data
                .public_constraints()
                .iter()
                .map(|v| v.iter().filter(|c| **c == card).count() as u8)
                .sum::<u8>()
    }
    pub fn action_info_str(&self) -> String {
        format!(
            "Player: {} {:?} public_constraints: {:?}, inferred_constraints: {:?}",
            self.player,
            self.action_info,
            self.public_constraints(),
            self.inferred_constraints()
        )
    }
    pub fn clone_public_meta_data(&self) -> T {
        self.meta_data.clone_public()
    }
    pub fn clone_meta_data(&self) -> T {
        self.meta_data.clone()
    }
    pub fn printlog(&self) {
        log::info!("Public Constraints: {:?}", self.public_constraints());
        log::info!("Inferred Constraints: {:?}", self.inferred_constraints());
        log::info!(
            "Impossible Constraints: {}",
            self.meta_data.format_impossible_constraints()
        );
        log::info!(
            "Impossible Constraints 2: {}",
            self.meta_data.format_impossible_constraints_2()
        );
        log::info!(
            "Impossible Constraints 3: {}",
            self.meta_data.format_impossible_constraints_3()
        );
    }
}
impl<T: InfoArrayTrait> CoupPossibilityAnalysis for SignificantAction<T> {
    fn public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.meta_data.public_constraints()
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.meta_data.sort_public_constraints();
        self.meta_data.public_constraints()
    }

    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.meta_data.inferred_constraints()
    }
    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.meta_data.sort_inferred_constraints();
        self.meta_data.inferred_constraints()
    }

    fn player_impossible_constraints(
        &mut self,
    ) -> [[bool; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        self.meta_data.impossible_constraints()
    }

    fn player_impossible_constraints_paired(
        &mut self,
    ) -> [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_PLAYERS_INCL_PILE] {
        self.meta_data.impossible_constraints_paired()
    }

    fn player_impossible_constraints_triple(
        &mut self,
    ) -> [[[bool; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE]; MAX_CARD_PERMS_ONE] {
        self.meta_data.impossible_constraints_triple()
    }

    fn player_can_have_card_alive(&mut self, player: usize, card: Card) -> bool {
        !self
            .meta_data
            .get_impossible_constraint(player, card as usize)
    }

    fn player_can_have_card_alive_lazy(&mut self, _player: usize, _card: Card) -> bool {
        unimplemented!()
    }

    fn player_can_have_cards_alive(&mut self, player: usize, cards: &[Card]) -> bool {
        if player < INDEX_PILE {
            if cards.len() == 2 {
                return !self.meta_data.get_impossible_constraint_2(
                    player,
                    cards[0] as usize,
                    cards[1] as usize,
                );
            } else if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0]);
            }
        } else if player == INDEX_PILE {
            if cards.len() == 1 {
                return self.player_can_have_card_alive(player, cards[0]);
            } else if cards.len() == 2 {
                return !self.meta_data.get_impossible_constraint_2(
                    player,
                    cards[0] as usize,
                    cards[1] as usize,
                );
            } else if cards.len() == 3 {
                return !self.meta_data.get_impossible_constraint_3(
                    cards[0] as usize,
                    cards[1] as usize,
                    cards[2] as usize,
                );
            }
        }
        false
    }
    fn player_can_have_cards_alive_lazy(&mut self, _player: usize, _cards: &[Card]) -> bool {
        unimplemented!()
    }

    fn is_legal_move_public(&mut self, _action_observation: &ActionObservation) -> bool {
        unimplemented!()
    }

    fn is_legal_move_private(&mut self, _action_observation: &ActionObservation) -> bool {
        unimplemented!()
    }
}
// TODO: Store also a version of constraint_history but split by players
// TODO: Improve analysis interface when using the manager... using last_constraint then the analysis is very clunky
// So it is easier to know the first time a player does something
// May be useful later
pub struct BackTrackCardCountManager<T: InfoArrayTrait> {
    private_player: Option<usize>,
    constraint_history: Vec<SignificantAction<T>>,
    move_no_history: Vec<usize>, // TODO: determine if more optimal to put in constraint_history
    move_no: usize,
}
impl<T: InfoArrayTrait> Default for BackTrackCardCountManager<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: InfoArrayTrait> BackTrackCardCountManager<T> {
    /// Constructor
    pub fn new() -> Self {
        let constraint_history = Vec::with_capacity(MAX_GAME_LENGTH);
        let move_no_history = Vec::with_capacity(MAX_GAME_LENGTH);
        Self {
            private_player: None,
            constraint_history,
            move_no_history,
            move_no: 1, // First move will be move 1, post-increment this (saving 0 for initial game state)
        }
    }
    /// Buffer for doing backtracking with
    pub fn create_buffer() -> (Vec<Vec<Card>>, Vec<Vec<Card>>) {
        let public_constraints: Vec<Vec<Card>> = (0..MAX_PLAYERS_INCL_PILE)
            .map(|_| Vec::with_capacity(2))
            .collect();
        let inferred_constraints: Vec<Vec<Card>> = (0..MAX_PLAYERS_INCL_PILE)
            .map(|_| Vec::with_capacity(4))
            .collect();
        (public_constraints, inferred_constraints)
    }
    pub fn clear_buffer(
        public_constraints: &mut [Vec<Card>],
        inferred_constraints: &mut [Vec<Card>],
    ) {
        public_constraints.iter_mut().for_each(|v| v.clear());
        inferred_constraints.iter_mut().for_each(|v| v.clear());
    }
    /// Logs the constraint's log
    pub fn printlog(&self) {
        log::trace!("Constraint History Len{}", self.constraint_history.len());
        log::trace!("history_move_no: {:?}", self.move_no_history);
        log::trace!("move_no: {:?}", self.move_no);
        if let Some(constraint) = self.constraint_history.last() {
            constraint.printlog();
        } else {
            log::trace!("Failed to print log, empty constraint history");
        }
    }
    /// Gets the Latest Constraint
    pub fn latest_constraint(&self) -> &SignificantAction<T> {
        // Should never pop() to 0
        self.constraint_history.last().unwrap()
    }
    pub fn latest_constraint_mut(&mut self) -> &mut SignificantAction<T> {
        // Should never pop() to 0
        self.constraint_history.last_mut().unwrap()
    }
    /// Updated for discard
    pub fn add_move_discard(
        &mut self,
        player_id: usize,
        cards: &[Card; MAX_CARDS_DISCARD],
        no_cards: usize,
    ) {
        // Assumes no_cards is either 1 or 2 only
        let action_info = ActionInfo::Discard { discard: cards[0] };
        let mut significant_action = SignificantAction::new(
            player_id as u8,
            action_info,
            self.constraint_history
                .last()
                .unwrap()
                .clone_public_meta_data(),
        );
        significant_action.meta_data.public_constraints_mut()[player_id].push(cards[0]);
        self.constraint_history.push(significant_action);
        if no_cards == 2 {
            let action_info = ActionInfo::Discard { discard: cards[1] };
            let mut significant_action = SignificantAction::new(
                player_id as u8,
                action_info,
                self.constraint_history
                    .last()
                    .unwrap()
                    .clone_public_meta_data(),
            );
            significant_action.meta_data.public_constraints_mut()[player_id].push(cards[1]);
            self.constraint_history.push(significant_action);
        }
        self.move_no_history.push(self.move_no);
    }
    /// Update for added move
    pub fn add_move_clone_public(&mut self, player_id: usize, action_info: ActionInfo) {
        let significant_action = SignificantAction::new(
            player_id as u8,
            action_info,
            self.constraint_history
                .last()
                .unwrap()
                .clone_public_meta_data(),
        );
        self.constraint_history.push(significant_action);
        self.move_no_history.push(self.move_no);
    }
    /// Update for added move
    pub fn add_move_clone_all(&mut self, player_id: usize, action_info: ActionInfo) {
        // We clone all to preserve impossible_constraint in this case!
        let significant_action = SignificantAction::new(
            player_id as u8,
            action_info,
            self.constraint_history.last().unwrap().clone_meta_data(),
        );
        self.constraint_history.push(significant_action);
        self.move_no_history.push(self.move_no);
    }
    // TODO: [OPTIMIZE] Add support for if constraint has been generated already!
    /// Generates all possible constraints
    pub fn generate_all_constraints(&mut self) {
        self.generate_impossible_constraints();
        self.generate_inferred_constraints();
    }
    // TODO: [OPTIMIZE] so we index impossible only via ascending order
    /// Does Backtracking to calculate impossibilities
    pub fn generate_impossible_constraints(&mut self) {
        // TODO: [OPTIMIZE] consider total dead cards inferred etc...
        let (mut public_constraints, mut inferred_constraints) = Self::create_buffer();
        for player_of_interest in 0..MAX_PLAYERS_INCL_PILE {
            if self.latest_constraint_mut().public_constraints()[player_of_interest].len() == 2 {
                self.latest_constraint_mut()
                    .set_all_impossible_constraints(player_of_interest, true);
                continue;
            }
            for card in 0..MAX_CARD_PERMS_ONE {
                log::trace!("generate_impossible_constraints 1 card : {:?}", card);
                inferred_constraints[player_of_interest].push(Card::try_from(card as u8).unwrap());
                let is_impossible = !self.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                self.latest_constraint_mut().set_impossible_constraint(
                    player_of_interest,
                    card,
                    is_impossible,
                );
                Self::clear_buffer(&mut public_constraints, &mut inferred_constraints);
            }
        }
        for player_of_interest in 0..MAX_PLAYERS_INCL_PILE {
            if !self.latest_constraint_mut().public_constraints()[player_of_interest].is_empty() {
                self.latest_constraint_mut()
                    .set_all_impossible_constraints_2(player_of_interest, true);
                continue;
            }
            for card_a in 0..MAX_CARD_PERMS_ONE {
                if self
                    .latest_constraint_mut()
                    .get_impossible_constraint(player_of_interest, card_a)
                {
                    for card_other in 0..MAX_CARD_PERMS_ONE {
                        self.latest_constraint_mut().set_impossible_constraint_2(
                            player_of_interest,
                            card_a,
                            card_other,
                            true,
                        );
                    }
                    continue;
                }
                for card_b in card_a..MAX_CARD_PERMS_ONE {
                    if self
                        .latest_constraint_mut()
                        .get_impossible_constraint(player_of_interest, card_b)
                    {
                        continue;
                    }
                    log::trace!(
                        "generate_impossible_constraints 2 card : {:?}, {:?}",
                        card_a,
                        card_b
                    );
                    inferred_constraints[player_of_interest].extend_from_slice(&[
                        Card::try_from(card_a as u8).unwrap(),
                        Card::try_from(card_b as u8).unwrap(),
                    ]);
                    let output = !self.possible_to_have_cards_latest(
                        &mut public_constraints,
                        &mut inferred_constraints,
                    );
                    Self::clear_buffer(&mut public_constraints, &mut inferred_constraints);
                    self.latest_constraint_mut().set_impossible_constraint_2(
                        player_of_interest,
                        card_a,
                        card_b,
                        output,
                    );
                }
            }
        }
        for card_a in 0..MAX_CARD_PERMS_ONE {
            if self
                .latest_constraint_mut()
                .get_impossible_constraint(INDEX_PILE, card_a)
            {
                for i in 0..MAX_CARD_PERMS_ONE {
                    for j in 0..MAX_CARD_PERMS_ONE {
                        self.latest_constraint_mut()
                            .set_impossible_constraint_3(card_a, i, j, true);
                    }
                }
                continue;
            }
            for card_b in card_a..MAX_CARD_PERMS_ONE {
                if self
                    .latest_constraint_mut()
                    .get_impossible_constraint(INDEX_PILE, card_b)
                {
                    continue;
                }
                for card_c in card_b..MAX_CARD_PERMS_ONE {
                    if self
                        .latest_constraint_mut()
                        .get_impossible_constraint(INDEX_PILE, card_c)
                    {
                        continue;
                    }
                    log::trace!(
                        "generate_impossible_constraints 3 card : {:?}, {:?}, {:?}",
                        card_a,
                        card_b,
                        card_c
                    );
                    inferred_constraints[INDEX_PILE].extend_from_slice(&[
                        Card::try_from(card_a as u8).unwrap(),
                        Card::try_from(card_b as u8).unwrap(),
                        Card::try_from(card_c as u8).unwrap(),
                    ]);
                    let output = !self.possible_to_have_cards_latest(
                        &mut public_constraints,
                        &mut inferred_constraints,
                    );
                    Self::clear_buffer(&mut public_constraints, &mut inferred_constraints);
                    self.latest_constraint_mut()
                        .set_impossible_constraint_3(card_a, card_b, card_c, output);
                }
            }
        }
    }
    /// Generates based on impossible_constraints
    pub fn generate_inferred_constraints(&mut self) {
        self.latest_constraint_mut()
            .inferred_constraints_mut()
            .iter_mut()
            .for_each(|v| v.clear());
        for player in 0..INDEX_PILE {
            if self.latest_constraint_mut().public_constraints()[player].is_empty() {
                if self
                    .latest_constraint_mut()
                    .meta_data
                    .count_possible_single_constraints(player)
                    == 1
                {
                    if let Some(card_num) = self
                        .latest_constraint_mut()
                        .meta_data
                        .find_only_possible_single_constraint(player)
                    {
                        self.latest_constraint_mut().inferred_constraints_mut()[player]
                            .push(Card::try_from(card_num as u8).unwrap());
                        self.latest_constraint_mut().inferred_constraints_mut()[player]
                            .push(Card::try_from(card_num as u8).unwrap());
                        continue;
                    }
                }
                // if 1 card not impossible and all the rest impossible
                let mut must_have_card: [u8; MAX_CARD_PERMS_ONE] = [3; MAX_CARD_PERMS_ONE];
                'outer: for card_num_a in 0..MAX_CARD_PERMS_ONE {
                    for card_num_b in card_num_a..MAX_CARD_PERMS_ONE {
                        // AA AB BB
                        // means nothing, I need to check if all have A or all have B
                        // need count lol
                        if self
                            .latest_constraint_mut()
                            .get_impossible_constraint_2(player, card_num_a, card_num_b)
                        {
                            continue;
                        }
                        let mut next = [0u8; MAX_CARD_PERMS_ONE];
                        next[card_num_a] += 1;
                        next[card_num_b] += 1;
                        must_have_card
                            .iter_mut()
                            .zip(next.iter())
                            .for_each(|(m, n)| *m = (*m).min(*n));
                        if must_have_card == [0; MAX_CARD_PERMS_ONE] {
                            break 'outer;
                        }
                    }
                }
                for (card_num, card_count) in must_have_card.iter().enumerate() {
                    for _ in 0..*card_count {
                        self.latest_constraint_mut().inferred_constraints_mut()[player]
                            .push(Card::try_from(card_num as u8).unwrap());
                    }
                }
            } else if self.latest_constraint_mut().public_constraints()[player].len() == 1
                && self
                    .latest_constraint_mut()
                    .meta_data
                    .count_possible_single_constraints(player)
                    == 1
            {
                if let Some(card_num) = self
                    .latest_constraint_mut()
                    .meta_data
                    .find_only_possible_single_constraint(player)
                {
                    self.latest_constraint_mut().inferred_constraints_mut()[player]
                        .push(Card::try_from(card_num as u8).unwrap());
                    continue;
                }
            }
        }
        let mut must_have_card: [u8; MAX_CARD_PERMS_ONE] = [3; MAX_CARD_PERMS_ONE];
        for card_num_a in 0..MAX_CARD_PERMS_ONE {
            for card_num_b in card_num_a..MAX_CARD_PERMS_ONE {
                for card_num_c in card_num_b..MAX_CARD_PERMS_ONE {
                    // AA AB BB
                    // means nothing, I need to check if all have A or all have B
                    // need count lol
                    if self
                        .latest_constraint_mut()
                        .get_impossible_constraint_3(card_num_a, card_num_b, card_num_c)
                    {
                        continue;
                    }
                    let mut next = [0u8; MAX_CARD_PERMS_ONE];
                    next[card_num_a] += 1;
                    next[card_num_b] += 1;
                    next[card_num_c] += 1;
                    must_have_card
                        .iter_mut()
                        .zip(next.iter())
                        .for_each(|(m, n)| *m = (*m).min(*n));
                    if must_have_card == [0; MAX_CARD_PERMS_ONE] {
                        return;
                        // break 'outer;
                    }
                }
            }
        }
        for (card_num, card_count) in must_have_card.iter().enumerate() {
            for _ in 0..*card_count {
                log::trace!(
                    "generate_inferred_constraints must_have_card: {:?}",
                    must_have_card
                );
                log::trace!(
                    "generate_inferred_constraints pushing: {:?}",
                    Card::try_from(card_num as u8).unwrap()
                );
                self.latest_constraint_mut().inferred_constraints_mut()[INDEX_PILE]
                    .push(Card::try_from(card_num as u8).unwrap());
            }
        }
    }
    /// Does Backtracking to determine if at a particular point that particular player could not have had some set of cards at start of turn
    /// Assuming we won't be using this for ambassador?
    /// This is now an example as to how to use self.possible_to_have_cards_recurse
    #[allow(unused)]
    pub fn impossible_to_have_cards_general(
        &self,
        index_lookback: usize,
        player_of_interest: usize,
        cards: &[u8; MAX_CARD_PERMS_ONE],
        public_constraints: &mut Vec<Vec<Card>>,
        inferred_constraints: &mut Vec<Vec<Card>>,
    ) -> bool {
        log::trace!(
            "impossible_to_have_cards player_of_interest: {}, cards: {:?}",
            player_of_interest,
            cards
        );
        debug_assert!(
            player_of_interest != INDEX_PILE && cards.iter().sum::<u8>() <= 2
                || player_of_interest == INDEX_PILE && cards.iter().sum::<u8>() <= 3,
            "cards too long!"
        );
        for (card_num, card_count) in cards.iter().enumerate() {
            for _ in 0..*card_count {
                inferred_constraints[player_of_interest]
                    .push(Card::try_from(card_num as u8).unwrap());
            }
        }
        !self.possible_to_have_cards_recurse(
            index_lookback,
            public_constraints,
            inferred_constraints,
        )
    }
    /// Returns true if possible, false if impossible
    ///
    /// Traces the game tree in reverse (from latest move to earliest move) by backtracking.
    /// Tracks possible paths known cards could have come from in the past.
    ///
    /// # Returns
    /// - `true` if a state is found to satisfy cards at the index_of_interest
    /// - `false` if no valid state is found
    ///
    /// # Notes
    /// Checks if a player can have specific cards based on the latest game state.
    ///
    /// This method verifies whether it's possible for a player to hold certain cards
    /// given all public information and constraints from the game history.
    ///
    /// # Buffer Management
    /// This method uses pre-allocated buffers for efficiency. You must:
    /// 1. Create buffers once using `create_buffer()`
    /// 2. Add cards to `inferred_constraints[player_index]` to test
    /// 3. Call this method to check feasibility
    /// 4. Use `clear_buffer()` between checks to reuse the buffers
    ///
    /// # Example
    /// ```rust
    /// // Create buffers once (reuse across multiple checks)
    /// let (mut public_constraints, mut inferred_constraints) =
    ///     BackTrackCardCountManager::<InfoArray>::create_buffer();
    ///
    /// // Check if player 0 can have Ambassador
    /// inferred_constraints[0].push(Card::Ambassador);
    /// if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
    ///     &mut public_constraints,
    ///     &mut inferred_constraints,
    /// ) {
    ///     // Player can have Ambassador
    /// }
    ///
    /// // Clear buffers before next check
    /// BackTrackCardCountManager::<InfoArray>::clear_buffer(
    ///     &mut public_constraints,
    ///     &mut inferred_constraints,
    /// );
    ///
    /// // Check if player 0 can have both Duke and Captain
    /// inferred_constraints[0].extend_from_slice(&[Card::Duke, Card::Captain]);
    /// if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
    ///     &mut public_constraints,
    ///     &mut inferred_constraints,
    /// ) {
    ///     // Player can have both cards
    /// }
    /// ```
    ///
    /// # Notes
    /// - Use `INDEX_PILE` to check pile constraints
    /// - Always clear buffers between checks to avoid stale data
    pub fn possible_to_have_cards_latest(
        &self,
        public_constraints: &mut Vec<Vec<Card>>,
        inferred_constraints: &mut Vec<Vec<Card>>,
    ) -> bool {
        self.possible_to_have_cards_recurse(
            self.constraint_history.len() - 1,
            public_constraints,
            inferred_constraints,
        )
    }
    /// Returns true if possible, false if impossible
    ///
    /// Traces the game tree in reverse (from latest move to earliest move) by backtracking.
    /// Tracks possible paths known cards could have come from in the past.
    ///
    /// # Returns
    /// - `true` if a state is found to satisfy cards at the index_of_interest
    /// - `false` if no valid state is found
    ///
    /// # Notes
    /// - User is responsible for clearing the public_constraints and inferred_constraints buffer after use!
    /// - User is responsible for adding the initial cards into the buffer!
    ///   Please see `BacktrackCardCountManager::impossible_to_have_cards_general` as an example
    pub fn possible_to_have_cards_recurse(
        &self,
        index_loop: usize,
        public_constraints: &mut Vec<Vec<Card>>,
        inferred_constraints: &mut Vec<Vec<Card>>,
    ) -> bool {
        log::trace!("After");
        log::trace!(
            "possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}",
            self.constraint_history[index_loop].player(),
            self.constraint_history[index_loop]
        );
        log::trace!(
            "possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}",
            self.constraint_history[index_loop].public_constraints(),
            inferred_constraints
        );
        if !self.is_valid_combination(index_loop, inferred_constraints) {
            // early exit before terminal node
            log::trace!("is_valid_combination evaluated to false");
            return false;
        }
        log::trace!("is_valid_combination evaluated to true");
        let player_loop = self.constraint_history[index_loop].player() as usize;
        let mut response = false;
        match self.constraint_history[index_loop].action_info() {
            ActionInfo::Discard { discard } => {
                log::trace!("Before Discard");
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.constraint_history[index_loop].public_constraints(), inferred_constraints);
                return MoveGuard::discard(
                    public_constraints,
                    inferred_constraints,
                    player_loop,
                    *discard,
                    |pub_con, inf_con| {
                        inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                            && self.possible_to_have_cards_recurse(index_loop - 1, pub_con, inf_con)
                    },
                );
            }
            ActionInfo::RevealRedraw {
                reveal,
                redraw,
                relinquish,
            } => {
                // Check if will burst before pushing
                match redraw {
                    Some(redraw_i) => {
                        log::trace!("Before Reveal Redraw");
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.constraint_history[index_loop].public_constraints(), inferred_constraints);

                        response = MoveGuard::swap_ordered(
                            public_constraints,
                            inferred_constraints,
                            INDEX_PILE,
                            player_loop,
                            &[*redraw_i],
                            &[*reveal],
                            |pub_con, inf_con| {
                                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                    && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                    && self.possible_to_have_cards_recurse(
                                        index_loop - 1,
                                        pub_con,
                                        inf_con,
                                    )
                            },
                        );
                    }
                    None => {
                        match relinquish {
                            Some(_relinquish_i) => {
                                // swap cards around sir
                                // relinquish_i == *reveal always
                                // Case 0: player redrew card != reveal
                                // Case 1: player redrew card == reveal (reveal from pile)
                                if inferred_constraints[INDEX_PILE].len() == MAX_HAND_SIZE_PILE
                                    && !inferred_constraints[INDEX_PILE].contains(reveal)
                                {
                                    // This state cannot be arrive after the reveal_redraw
                                    return false;
                                }
                                log::trace!("Before Reveal Relinquish");
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.constraint_history[index_loop].public_constraints(), inferred_constraints);

                                if inferred_constraints[player_loop].is_empty() {
                                    log::trace!(
                                        "inferred_constraints[player_loop].is_empty(): {:?}",
                                        inferred_constraints[player_loop]
                                    );
                                    let _ = MoveGuard::swap_ordered(
                                        public_constraints,
                                        inferred_constraints,
                                        player_loop,
                                        INDEX_PILE,
                                        &[*reveal],
                                        &[],
                                        |pub_con, inf_con| {
                                            response = inf_con[player_loop].len()
                                                <= MAX_HAND_SIZE_PLAYER
                                                && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                                && self.possible_to_have_cards_recurse(
                                                    index_loop - 1,
                                                    pub_con,
                                                    inf_con,
                                                );
                                            // Force rollback regardless of `response`
                                            false
                                        },
                                    );
                                    return response;
                                }
                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();
                                for card_player in iter_cards.iter() {
                                    // Card Source was not from Pile
                                    log::trace!("Before Reveal Relinquish B");
                                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.constraint_history[index_loop].public_constraints(), inferred_constraints);

                                    if inferred_constraints[player_loop].len() < 2
                                        && MoveGuard::swap_ordered(
                                            public_constraints,
                                            inferred_constraints,
                                            player_loop,
                                            INDEX_PILE,
                                            &[*reveal],
                                            &[],
                                            |pub_con, inf_con| {
                                                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                                    && inf_con[INDEX_PILE].len()
                                                        <= MAX_HAND_SIZE_PILE
                                                    && self.possible_to_have_cards_recurse(
                                                        index_loop - 1,
                                                        pub_con,
                                                        inf_con,
                                                    )
                                            },
                                        )
                                    {
                                        return true;
                                    }
                                    // Card Source was from Pile
                                    if *card_player != *reveal {
                                        log::trace!("Before Reveal Relinquish C");
                                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", self.constraint_history[index_loop].public_constraints(), inferred_constraints);

                                        if MoveGuard::swap_ordered(
                                            public_constraints,
                                            inferred_constraints,
                                            player_loop,
                                            INDEX_PILE,
                                            &[*reveal],
                                            &[*card_player],
                                            |pub_con, inf_con| {
                                                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                                    && inf_con[INDEX_PILE].len()
                                                        <= MAX_HAND_SIZE_PILE
                                                    && self.possible_to_have_cards_recurse(
                                                        index_loop - 1,
                                                        pub_con,
                                                        inf_con,
                                                    )
                                            },
                                        ) {
                                            return true;
                                        }
                                    }
                                }
                            }
                            None => {
                                if inferred_constraints[player_loop].len()
                                    + inferred_constraints[INDEX_PILE].len()
                                    == MAX_HAND_SIZE_PLAYER + MAX_HAND_SIZE_PILE
                                    && !inferred_constraints[player_loop].contains(reveal)
                                    && !inferred_constraints[INDEX_PILE].contains(reveal)
                                {
                                    // This state cannot be arrive after the reveal_redraw
                                    return false;
                                }
                                if inferred_constraints[player_loop].is_empty() {
                                    return MoveGuard::swap_ordered(
                                        public_constraints,
                                        inferred_constraints,
                                        player_loop,
                                        INDEX_PILE,
                                        &[*reveal],
                                        &[],
                                        |pub_con, inf_con| {
                                            // keep your acceptance gates exactly as-is:
                                            inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                                && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                                && self.possible_to_have_cards_recurse(
                                                    index_loop - 1,
                                                    pub_con,
                                                    inf_con,
                                                )
                                        },
                                    );
                                }

                                let mut iter_cards = inferred_constraints[player_loop].clone();
                                iter_cards.sort_unstable();
                                iter_cards.dedup();

                                for card_player in iter_cards.iter() {
                                    if (*card_player != *reveal
                                        || inferred_constraints[INDEX_PILE].contains(reveal))
                                        && MoveGuard::swap_ordered(
                                            public_constraints,
                                            inferred_constraints,
                                            player_loop,
                                            INDEX_PILE,
                                            &[*reveal],
                                            &[],
                                            |pub_con, inf_con| {
                                                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                                    && inf_con[INDEX_PILE].len()
                                                        <= MAX_HAND_SIZE_PILE
                                                    && self.possible_to_have_cards_recurse(
                                                        index_loop - 1,
                                                        pub_con,
                                                        inf_con,
                                                    )
                                            },
                                        )
                                    {
                                        return true;
                                    }

                                    if MoveGuard::swap_ordered(
                                        public_constraints,
                                        inferred_constraints,
                                        INDEX_PILE,
                                        player_loop,
                                        &[*card_player],
                                        &[*reveal],
                                        |pub_con, inf_con| {
                                            inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                                && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                                && self.possible_to_have_cards_recurse(
                                                    index_loop - 1,
                                                    pub_con,
                                                    inf_con,
                                                )
                                        },
                                    ) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ActionInfo::ExchangeDrawChoice { .. } => {
                unimplemented!("Deprecated!");
                // response = self.recurse_variants_exchange_public(index_loop, player_loop, public_constraints, inferred_constraints);
            }
            ActionInfo::ExchangeDraw { draw } => {
                if !draw.is_empty() {
                    // Assumes draw is always 2 cards
                    response = MoveGuard::with_needed_cards_present(
                        public_constraints,
                        inferred_constraints,
                        INDEX_PILE,
                        draw,
                        |pub_con, inf_con| {
                            inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                && self.possible_to_have_cards_recurse(
                                    index_loop - 1,
                                    pub_con,
                                    inf_con,
                                )
                        },
                    );
                } else {
                    // [REQUIRED FOR LAZY EVAL] Although ExchangeChoice skips over this
                    // When we use lazy evaluation on previous moves,
                    // ExchangeDraw inferred impossibilities cannot just clone from the previous move
                    // We will need this to evaluate ExchangeDraw inference
                    response = self.possible_to_have_cards_recurse(
                        index_loop - 1,
                        public_constraints,
                        inferred_constraints,
                    );
                }
            }
            ActionInfo::ExchangeChoice { relinquish } => {
                if let ActionInfo::ExchangeDraw { draw } =
                    self.constraint_history[index_loop - 1].action_info()
                {
                    if draw.is_empty() {
                        response = self.recurse_variants_exchange_public(
                            index_loop,
                            player_loop,
                            public_constraints,
                            inferred_constraints,
                        );
                    } else {
                        // Assumes both relinquish cards are known
                        // Assumes hand cards are known (they are alive cards)
                        // Pool to choose from is hand + draw
                        response = self.recurse_variants_exchange_private(
                            index_loop,
                            player_loop,
                            draw,
                            relinquish,
                            public_constraints,
                            inferred_constraints,
                        );
                    }
                }
            }
            ActionInfo::StartInferred => {
                // TODO: OPTIMIZE this is only needed if bool_know_priv_info is true
                let mut buffer: Vec<(usize, Card)> = Vec::with_capacity(3);
                for (player, player_constraints) in inferred_constraints.iter_mut().enumerate() {
                    let mut card_counts_req = [0u8; MAX_CARD_PERMS_ONE];
                    let mut card_counts_cur = [0u8; MAX_CARD_PERMS_ONE];
                    for card_start in
                        self.constraint_history[index_loop].inferred_constraints()[player].iter()
                    {
                        card_counts_req[*card_start as usize] += 1;
                    }
                    for card_start in player_constraints.iter() {
                        card_counts_cur[*card_start as usize] += 1;
                    }
                    for card_num_to_add in 0..MAX_CARD_PERMS_ONE {
                        if card_counts_req[card_num_to_add] > card_counts_cur[card_num_to_add] {
                            for _ in 0..(card_counts_req[card_num_to_add]
                                - card_counts_cur[card_num_to_add])
                            {
                                let card_add = Card::try_from(card_num_to_add as u8).unwrap();
                                player_constraints.push(card_add);
                                buffer.push((player, card_add));
                            }
                        }
                    }
                }

                if self.possible_to_have_cards_recurse(
                    index_loop - 1,
                    public_constraints,
                    inferred_constraints,
                ) {
                    return true;
                }
                for (player_remove, card_remove) in buffer.iter() {
                    if let Some(pos) = inferred_constraints[*player_remove]
                        .iter()
                        .rposition(|c| *c == *card_remove)
                    {
                        inferred_constraints[*player_remove].swap_remove(pos);
                    }
                }
            }
            ActionInfo::Start => {
                // Managed to reach base
                log::trace!(
                    "possible_to_have_cards_recurse found true at index: {}",
                    index_loop
                );
                response = true;
            }
        }
        response
    }
    /// Return true if hypothesised card permutations cannot be shown to be impossible
    pub fn is_valid_combination(
        &self,
        index_loop: usize,
        inferred_constraints: &Vec<Vec<Card>>,
    ) -> bool {
        let public_constraints = self.constraint_history[index_loop].public_constraints();
        // Check actual constraints at leaf node
        // All public_constraints inside actual
        log::trace!("is_valid_combination for: index {}, considering public_constraints: {:?}, inferred_constraints: {:?}", index_loop, public_constraints, inferred_constraints);
        for card in [
            Card::Ambassador,
            Card::Assassin,
            Card::Captain,
            Card::Duke,
            Card::Contessa,
        ]
        .iter()
        {
            if inferred_constraints
                .iter()
                .map(|v| v.iter().filter(|c| **c == *card).count() as u8)
                .sum::<u8>()
                // Documentation comment => the following is useless if there are no revives in Coup
                // + public_constraints
                //     .iter()
                //     .map(|v| v.iter().filter(|c| **c == *card).count() as u8)
                //     .sum::<u8>()
                > 3
            {
                log::trace!("is_valid_combination constraints has too many {:?}", card);
                return false;
            }
        }
        // for player in 0..INDEX_PILE {
        //     if public_constraints[player].len() + inferred_constraints[player].len()
        //         > MAX_HAND_SIZE_PLAYER
        //     {
        //         log::trace!("is_valid_combination player {} has too many cards", player);
        //         return false;
        //     }
        // }
        // if public_constraints[INDEX_PILE].len() + inferred_constraints[INDEX_PILE].len()
        //     > MAX_HAND_SIZE_PILE
        // {
        //     log::trace!("is_valid_combination pile has too many cards");
        //     return false;
        // }
        for (player, player_constraints) in inferred_constraints.iter().enumerate() {
            if player_constraints.len() == 1
                && self.constraint_history[index_loop]
                    .get_impossible_constraint(player, player_constraints[0] as usize)
            {
                return false;
            }
            if player_constraints.len() == 2
                && self.constraint_history[index_loop].get_impossible_constraint_2(
                    player,
                    player_constraints[0] as usize,
                    player_constraints[1] as usize,
                )
            {
                return false;
            }
        }
        if inferred_constraints[INDEX_PILE].len() == 3
            && self.constraint_history[index_loop].get_impossible_constraint_3(
                inferred_constraints[INDEX_PILE][0] as usize,
                inferred_constraints[INDEX_PILE][1] as usize,
                inferred_constraints[INDEX_PILE][2] as usize,
            )
        {
            return false;
        }
        // =================== Required to test inferred at Start! ======================
        // This is Implemented in recursion
        // But this is the general form that can be used to check any particular state
        // if let ActionInfo::StartInferred{ .. } = self.constraint_history[index_loop].action_info() {
        //     for player in 0..INDEX_PILE {
        //         let mut current_card_counts: [u8; 5] = [0; 5];
        //         inferred_constraints[player].iter().for_each(|c| current_card_counts[*c as usize] += 1);

        //         let mut required_card_counts: [u8; 5] = [0; 5];
        //         self.constraint_history[index_loop].inferred_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);
        //         self.constraint_history[index_loop].public_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);

        //         let mut total_count : u8 = 0;
        //         current_card_counts.iter().zip(required_card_counts.iter()).for_each(|(cur, req)| total_count += *cur.max(req));
        //         let fulfilled = total_count <= 2;
        //         if !fulfilled {
        //             log::trace!("is_valid_combination player {} failed to fulfil previous state!", player);
        //             return false
        //         }
        //     }
        //     let player = INDEX_PILE;
        //     let mut current_card_counts: [u8; 5] = [0; 5];
        //     inferred_constraints[player].iter().for_each(|c| current_card_counts[*c as usize] += 1);

        //     let mut required_card_counts: [u8; 5] = [0; 5];
        //     self.constraint_history[index_loop].inferred_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);
        //     self.constraint_history[index_loop].public_constraints()[player].iter().for_each(|c| required_card_counts[*c as usize] += 1);

        //     let mut total_count : u8 = 0;
        //     current_card_counts.iter().zip(required_card_counts.iter()).for_each(|(cur, req)| total_count += *cur.max(req));
        //     let fulfilled = total_count <= 3;
        //     if !fulfilled {
        //         log::trace!("is_valid_combination player {} failed to fulfil previous state!", player);
        //         return false
        //     }
        // }
        true
    }
    /// Recursion case for exchange with no private information
    pub fn recurse_variants_exchange_public(
        &self,
        index_loop: usize,
        player_loop: usize,
        public_constraints: &mut Vec<Vec<Card>>,
        inferred_constraints: &mut Vec<Vec<Card>>,
    ) -> bool {
        let player_lives =
            2 - self.constraint_history[index_loop].public_constraints()[player_loop].len() as u8;
        let mut iter_cards_player = inferred_constraints[player_loop].clone();
        iter_cards_player.sort_unstable();
        iter_cards_player.dedup();
        let mut iter_cards_pile = inferred_constraints[INDEX_PILE].clone();
        iter_cards_pile.sort_unstable();
        iter_cards_pile.dedup();
        let mut player_count = [0u8; MAX_CARD_PERMS_ONE];
        let mut pile_count = [0u8; MAX_CARD_PERMS_ONE];
        inferred_constraints[player_loop]
            .iter()
            .for_each(|c| player_count[*c as usize] += 1);
        inferred_constraints[INDEX_PILE]
            .iter()
            .for_each(|c| pile_count[*c as usize] += 1);

        // 0 player_to_pile move, 0 pile_to_player move
        log::trace!("Before Exchange Same");
        log::trace!(
            "possible_to_have_cards_recurse: index_loop: {index_loop} move: player: {} {:?}",
            self.constraint_history[index_loop].player(),
            self.constraint_history[index_loop].action_info()
        );
        log::trace!(
            "possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}",
            public_constraints,
            inferred_constraints
        );

        if self.possible_to_have_cards_recurse(
            index_loop - 2,
            public_constraints,
            inferred_constraints,
        ) {
            return true;
        }
        // 1 player_to_pile move, 0 pile_to_player move
        if inferred_constraints[INDEX_PILE].len() < 3
            && !inferred_constraints[player_loop].is_empty()
        {
            for card_player in iter_cards_player.iter() {
                // move to pile
                if MoveGuard::swap(
                    public_constraints,
                    inferred_constraints,
                    player_loop,
                    INDEX_PILE,
                    &[*card_player],
                    &[],
                    |pub_con, inf_con| {
                        inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                            && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                            && self.possible_to_have_cards_recurse(index_loop - 2, pub_con, inf_con)
                    },
                ) {
                    return true;
                }
            }
        }
        // 0 player_to_pile move, 1 pile_to_player move
        if inferred_constraints[player_loop].len() < 2
            && !inferred_constraints[INDEX_PILE].is_empty()
        {
            for card_pile in iter_cards_pile.iter() {
                // move to player
                if MoveGuard::swap(
                    public_constraints,
                    inferred_constraints,
                    player_loop,
                    INDEX_PILE,
                    &[],
                    &[*card_pile],
                    |pub_con, inf_con| {
                        inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                            && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                            && self.possible_to_have_cards_recurse(index_loop - 2, pub_con, inf_con)
                    },
                ) {
                    return true;
                }
            }
        }
        // 1 player_to_pile move, 1 pile_to_player move
        if !inferred_constraints[player_loop].is_empty()
            && !inferred_constraints[INDEX_PILE].is_empty()
        {
            for card_player in iter_cards_player.iter() {
                for card_pile in iter_cards_pile.iter() {
                    if card_player == card_pile {
                        continue;
                    }
                    log::trace!("Before Exchange 1 player_to_pile 1 pile_to_player");
                    log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                    log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                    if MoveGuard::swap(
                        public_constraints,
                        inferred_constraints,
                        player_loop,
                        INDEX_PILE,
                        &[*card_player],
                        &[*card_pile],
                        |pub_con, inf_con| {
                            inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                && self.possible_to_have_cards_recurse(
                                    index_loop - 2,
                                    pub_con,
                                    inf_con,
                                )
                        },
                    ) {
                        return true;
                    }
                }
            }
        }
        if player_lives > 1 {
            // 2 player_to_pile move, 0 pile_to_player move
            if inferred_constraints[player_loop].len() == 2
                && inferred_constraints[INDEX_PILE].len() < 2
            {
                log::trace!("Before Exchange 2 player_to_pile 0 pile_to_player");
                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                if MoveGuard::swap(
                    public_constraints,
                    inferred_constraints,
                    player_loop,
                    INDEX_PILE,
                    &[
                        inferred_constraints[player_loop][0],
                        inferred_constraints[player_loop][1],
                    ],
                    &[],
                    |pub_con, inf_con| {
                        inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                            && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                            && self.possible_to_have_cards_recurse(index_loop - 2, pub_con, inf_con)
                    },
                ) {
                    return true;
                }
            }
            // 0 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].is_empty()
                && inferred_constraints[INDEX_PILE].len() > 1
            {
                for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                    for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len() {
                        if index_pile_to_player_0 == index_pile_to_player_1
                            && pile_count[iter_cards_pile[index_pile_to_player_0] as usize] < 2
                        {
                            continue; // Ensure enough cards to move
                        }
                        log::trace!("Before Exchange 0 player_to_pile 2 pile_to_player");
                        log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                        log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                        if MoveGuard::swap(
                            public_constraints,
                            inferred_constraints,
                            player_loop,
                            INDEX_PILE,
                            &[],
                            &[
                                iter_cards_pile[index_pile_to_player_0],
                                iter_cards_pile[index_pile_to_player_1],
                            ],
                            |pub_con, inf_con| {
                                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                    && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                    && self.possible_to_have_cards_recurse(
                                        index_loop - 2,
                                        pub_con,
                                        inf_con,
                                    )
                            },
                        ) {
                            return true;
                        }
                    }
                }
            }
            // 2 player_to_pile move, 1 pile_to_player move
            if !inferred_constraints[INDEX_PILE].is_empty()
                && inferred_constraints[INDEX_PILE].len() < 3
                && inferred_constraints[player_loop].len() > 1
            {
                for card_pile in iter_cards_pile.iter() {
                    for index_player_to_pile_0 in 0..iter_cards_player.len() {
                        // TODO: Shift index_player_to_pile == case shift here
                        if iter_cards_player[index_player_to_pile_0] == *card_pile {
                            continue; // Avoid duplicates
                        }
                        for index_player_to_pile_1 in
                            index_player_to_pile_0..iter_cards_player.len()
                        {
                            // Check DF
                            if iter_cards_player[index_player_to_pile_1] == *card_pile {
                                continue; // Avoid duplicates
                            }
                            if index_player_to_pile_0 == index_player_to_pile_1
                                && player_count[iter_cards_player[index_player_to_pile_0] as usize]
                                    < 2
                            {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue; // Ensure enough cards to move
                            }
                            log::trace!("Before Exchange 2 player_to_pile 1 pile_to_player");
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                            if MoveGuard::swap(
                                public_constraints,
                                inferred_constraints,
                                player_loop,
                                INDEX_PILE,
                                &[
                                    iter_cards_player[index_player_to_pile_0],
                                    iter_cards_player[index_player_to_pile_1],
                                ],
                                &[*card_pile],
                                |pub_con, inf_con| {
                                    inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                        && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                        && self.possible_to_have_cards_recurse(
                                            index_loop - 2,
                                            pub_con,
                                            inf_con,
                                        )
                                },
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
            // 1 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() == 1
                && inferred_constraints[INDEX_PILE].len() > 1
            {
                for card_player in iter_cards_player.iter() {
                    for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                        if iter_cards_pile[index_pile_to_player_0] == *card_player {
                            continue; // Avoid Duplicates
                        }
                        for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len()
                        {
                            // Check DF
                            if iter_cards_pile[index_pile_to_player_1] == *card_player {
                                continue; // Avoid Duplicates
                            }
                            if index_pile_to_player_0 == index_pile_to_player_1
                                && (pile_count[iter_cards_pile[index_pile_to_player_0] as usize]
                                    < 2)
                            {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue; // Ensure enough cards to move
                            }
                            log::trace!("Before Exchange 1 player_to_pile 2 pile_to_player");
                            log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                            log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                            if MoveGuard::swap(
                                public_constraints,
                                inferred_constraints,
                                player_loop,
                                INDEX_PILE,
                                &[*card_player],
                                &[
                                    iter_cards_pile[index_pile_to_player_0],
                                    iter_cards_pile[index_pile_to_player_1],
                                ],
                                |pub_con, inf_con| {
                                    inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                        && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                        && self.possible_to_have_cards_recurse(
                                            index_loop - 2,
                                            pub_con,
                                            inf_con,
                                        )
                                },
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
            // 2 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() > 1
                && inferred_constraints[INDEX_PILE].len() > 1
            {
                for index_player_to_pile_0 in 0..iter_cards_player.len() {
                    for index_player_to_pile_1 in index_player_to_pile_0..iter_cards_player.len() {
                        if index_player_to_pile_0 == index_player_to_pile_1
                            && player_count[iter_cards_player[index_player_to_pile_0] as usize] < 2
                        {
                            // Checks that player has enough cards to move out
                            // TODO: OPTIMIZE Can shift this out of for loop actually
                            continue; // Ensure enough cards to move
                        }
                        // Check DF
                        for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                            if iter_cards_pile[index_pile_to_player_0]
                                == iter_cards_player[index_player_to_pile_0]
                                || iter_cards_pile[index_pile_to_player_0]
                                    == iter_cards_player[index_player_to_pile_1]
                            {
                                continue; // Avoid Duplicates
                            }
                            for index_pile_to_player_1 in
                                index_pile_to_player_0..iter_cards_pile.len()
                            {
                                if iter_cards_pile[index_pile_to_player_1]
                                    == iter_cards_player[index_player_to_pile_0]
                                    || iter_cards_pile[index_pile_to_player_1]
                                        == iter_cards_player[index_player_to_pile_1]
                                {
                                    continue; // Avoid Duplicates
                                }
                                if index_pile_to_player_0 == index_pile_to_player_1
                                    && (pile_count
                                        [iter_cards_pile[index_pile_to_player_0] as usize]
                                        < 2)
                                {
                                    // Checks that player has enough cards to move out
                                    // TODO: OPTIMIZE Can shift this out of for loop actually
                                    continue; // Ensure enough cards to move
                                }
                                log::trace!("Before Exchange 2 player_to_pile 2 pile_to_player");
                                log::trace!("possible_to_have_cards_recurse: index_loop: {index_loop}, move: player: {} {:?}", self.constraint_history[index_loop].player(), self.constraint_history[index_loop].action_info());
                                log::trace!("possible_to_have_cards_recurse: public_constraints: {:?}, inferred_constraints: {:?}", public_constraints, inferred_constraints);
                                if MoveGuard::swap(
                                    public_constraints,
                                    inferred_constraints,
                                    player_loop,
                                    INDEX_PILE,
                                    &[
                                        iter_cards_player[index_player_to_pile_0],
                                        iter_cards_player[index_player_to_pile_1],
                                    ],
                                    &[
                                        iter_cards_pile[index_pile_to_player_0],
                                        iter_cards_pile[index_pile_to_player_1],
                                    ],
                                    |pub_con, inf_con| {
                                        inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                                            && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                                            && self.possible_to_have_cards_recurse(
                                                index_loop - 2,
                                                pub_con,
                                                inf_con,
                                            )
                                    },
                                ) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
    /// Recursion case for exchange with private information
    pub fn recurse_variants_exchange_private(
        &self,
        index_loop: usize,
        player_loop: usize,
        draw: &[Card],
        relinquish: &[Card],
        public_constraints: &mut Vec<Vec<Card>>,
        inferred_constraints: &mut Vec<Vec<Card>>,
    ) -> bool {
        log::trace!("In recurse_variants_exchange_private!");
        MoveGuard::swap_ordered(
            public_constraints,
            inferred_constraints,
            player_loop,
            INDEX_PILE,
            &[relinquish[0], relinquish[1]],
            &[draw[0], draw[1]],
            |pub_con, inf_con| {
                inf_con[player_loop].len() <= MAX_HAND_SIZE_PLAYER
                    && inf_con[INDEX_PILE].len() <= MAX_HAND_SIZE_PILE
                    && self.possible_to_have_cards_recurse(index_loop - 2, pub_con, inf_con)
            },
        )
    }
}

impl<T: InfoArrayTrait> CoupTraversal for BackTrackCardCountManager<T> {
    fn start_public(&mut self, _player: usize) {
        self.private_player = None;
        self.constraint_history.clear();
        self.move_no_history.clear();
        let start_public = T::start_public();
        self.constraint_history.push(SignificantAction::new(
            7,
            ActionInfo::Start,
            start_public.clone(),
        ));
        self.constraint_history.push(SignificantAction::new(
            7,
            ActionInfo::StartInferred,
            start_public,
        ));
        self.move_no_history.push(0);
        self.move_no_history.push(0);
        self.move_no = 1;
    }

    fn start_private(&mut self, player: usize, cards: &[Card; MAX_HAND_SIZE_PLAYER]) {
        self.private_player = Some(player);
        self.constraint_history.clear();
        self.move_no_history.clear();
        let start_private = T::start_private(player, cards);
        self.constraint_history.push(SignificantAction::new(
            7,
            ActionInfo::Start,
            T::start_public(),
        ));
        self.constraint_history.push(SignificantAction::new(
            7,
            ActionInfo::StartInferred,
            start_private,
        ));
        self.move_no_history.push(0);
        self.move_no_history.push(0);
        self.move_no = 1;
    }

    fn start_known(&mut self, _cards: &[Vec<Card>]) {
        unimplemented!()
    }

    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of public information but not private information
    fn push_ao_public(&mut self, action: &ActionObservation) {
        // Handle different move types
        match action {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                self.add_move_discard(*player_id, card, *no_cards);
                self.generate_all_constraints();
            }
            ActionObservation::RevealRedraw {
                player_id, reveal, ..
            } => {
                let action_info = ActionInfo::RevealRedraw {
                    reveal: *reveal,
                    redraw: None,
                    relinquish: None,
                };
                log::trace!("Adding move RevealRedraw");
                self.add_move_clone_public(*player_id, action_info);
                self.generate_all_constraints();
            }
            ActionObservation::ExchangeDraw { player_id, .. } => {
                let action_info = ActionInfo::ExchangeDraw {
                    draw: Vec::with_capacity(2),
                };
                log::trace!("Adding move ExchangeDraw");
                self.add_move_clone_all(*player_id, action_info);
            }
            ActionObservation::ExchangeChoice { player_id, .. } => {
                let action_info = ActionInfo::ExchangeChoice {
                    relinquish: Vec::with_capacity(2),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
                self.generate_all_constraints();
            }
            _ => {}
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    // TODO: [OPTIMIZE] Add checks for if previous move has been calculated properly
    // TODO: [OPTIMIZE] Consider Add clones for impossible states based on public constraints?
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of public information but not private information
    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        // Handle different move types
        match action {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                self.add_move_discard(*player_id, card, *no_cards);
            }
            ActionObservation::RevealRedraw {
                player_id, reveal, ..
            } => {
                let action_info = ActionInfo::RevealRedraw {
                    reveal: *reveal,
                    redraw: None,
                    relinquish: None,
                };
                log::trace!("Adding move RevealRedraw");
                self.add_move_clone_public(*player_id, action_info);
            }
            ActionObservation::ExchangeDraw { player_id, .. } => {
                let action_info = ActionInfo::ExchangeDraw {
                    draw: Vec::with_capacity(2),
                };
                log::trace!("Adding move ExchangeDraw");
                self.add_move_clone_all(*player_id, action_info);
            }
            ActionObservation::ExchangeChoice { player_id, .. } => {
                let action_info = ActionInfo::ExchangeChoice {
                    relinquish: Vec::with_capacity(2),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
            }
            _ => {}
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of private information
    fn push_ao_private(&mut self, action: &ActionObservation) {
        // Handle different move types
        match action {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                self.add_move_discard(*player_id, card, *no_cards);
                self.generate_all_constraints();
            }
            ActionObservation::RevealRedraw {
                player_id,
                reveal,
                redraw,
            } => {
                let action_info = ActionInfo::RevealRedraw {
                    reveal: *reveal,
                    redraw: Some(*redraw),
                    relinquish: None,
                };
                log::trace!("Adding move RevealRedraw");
                if *reveal == *redraw {
                    self.add_move_clone_all(*player_id, action_info);
                    self.generate_all_constraints();
                } else {
                    self.add_move_clone_public(*player_id, action_info);
                    self.generate_all_constraints();
                }
            }
            ActionObservation::ExchangeDraw { player_id, card } => {
                let action_info = ActionInfo::ExchangeDraw {
                    draw: card.to_vec(),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
                self.generate_all_constraints();
            }
            ActionObservation::ExchangeChoice {
                player_id,
                relinquish,
            } => {
                let action_info = ActionInfo::ExchangeChoice {
                    relinquish: relinquish.to_vec(),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
                self.generate_all_constraints();
            }
            _ => {}
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }
    /// Entrypoint for any action done, updates history accordingly
    /// Assumes knowledge of private information
    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        // Handle different move types
        match action {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                self.add_move_discard(*player_id, card, *no_cards);
            }
            ActionObservation::RevealRedraw {
                player_id,
                reveal,
                redraw,
            } => {
                let action_info = ActionInfo::RevealRedraw {
                    reveal: *reveal,
                    redraw: Some(*redraw),
                    relinquish: None,
                };
                log::trace!("Adding move RevealRedraw");
                self.add_move_clone_public(*player_id, action_info);
            }
            ActionObservation::ExchangeDraw { player_id, card } => {
                let action_info = ActionInfo::ExchangeDraw {
                    draw: card.to_vec(),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
            }
            ActionObservation::ExchangeChoice {
                player_id,
                relinquish,
            } => {
                let action_info = ActionInfo::ExchangeChoice {
                    relinquish: relinquish.to_vec(),
                };
                log::trace!("Adding move ExchangeChoice");
                self.add_move_clone_public(*player_id, action_info);
            }
            _ => {}
        }
        // shove move_no into CollectiveConstraint
        // post_increment: move_no is now the number of the next move
        self.move_no += 1;
    }

    fn pop(&mut self) {
        if self.move_no > 1 {
            self.move_no -= 1;
            while self.move_no_history.last() == Some(&self.move_no) {
                self.constraint_history.pop();
                self.move_no_history.pop();
            }
        }
    }
}

impl<T: InfoArrayTrait> CoupPossibilityAnalysis for BackTrackCardCountManager<T> {
    fn public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().public_constraints()
    }

    fn sorted_public_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().sorted_public_constraints()
    }

    fn inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().inferred_constraints()
    }

    fn sorted_inferred_constraints(&mut self) -> &Vec<Vec<Card>> {
        self.latest_constraint_mut().sorted_inferred_constraints()
    }

    fn player_impossible_constraints(&mut self) -> [[bool; 5]; 7] {
        self.latest_constraint_mut().player_impossible_constraints()
    }

    fn player_impossible_constraints_paired(&mut self) -> [[[bool; 5]; 5]; 7] {
        self.latest_constraint_mut()
            .player_impossible_constraints_paired()
    }

    fn player_impossible_constraints_triple(&mut self) -> [[[bool; 5]; 5]; 5] {
        self.latest_constraint_mut()
            .player_impossible_constraints_triple()
    }

    fn player_can_have_card_alive(&mut self, player: usize, card: Card) -> bool {
        self.latest_constraint_mut()
            .player_can_have_card_alive(player, card)
    }

    fn player_can_have_card_alive_lazy(&mut self, player: usize, card: Card) -> bool {
        let (mut public_constraints, mut inferred_constraints) = Self::create_buffer();
        inferred_constraints[player].push(card);
        self.possible_to_have_cards_latest(&mut public_constraints, &mut inferred_constraints)
    }

    fn player_can_have_cards_alive(&mut self, player: usize, cards: &[Card]) -> bool {
        self.latest_constraint_mut()
            .player_can_have_cards_alive(player, cards)
    }
    fn player_can_have_cards_alive_lazy(&mut self, player: usize, cards: &[Card]) -> bool {
        // TODO: [OPTIMIZE] check if latest state is updated!
        let (mut public_constraints, mut inferred_constraints) = Self::create_buffer();
        inferred_constraints[player].extend_from_slice(cards);
        self.possible_to_have_cards_latest(&mut public_constraints, &mut inferred_constraints)
    }
    fn is_legal_move_public(&mut self, action_observation: &ActionObservation) -> bool {
        match action_observation {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                if *no_cards == 1 {
                    self.player_can_have_card_alive_lazy(*player_id, card[0])
                } else {
                    self.player_can_have_cards_alive_lazy(*player_id, card)
                }
            }
            ActionObservation::RevealRedraw {
                player_id, reveal, ..
            } => self.player_can_have_card_alive_lazy(*player_id, *reveal),
            _ => true,
        }
    }
    fn is_legal_move_private(&mut self, action_observation: &ActionObservation) -> bool {
        match action_observation {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                if *no_cards == 1 {
                    self.player_can_have_card_alive_lazy(*player_id, card[0])
                } else {
                    self.player_can_have_cards_alive_lazy(*player_id, card)
                }
            }
            ActionObservation::RevealRedraw {
                player_id,
                reveal,
                redraw,
            } => {
                self.player_can_have_card_alive_lazy(*player_id, *reveal)
                    && (*reveal == *redraw
                        || self.player_can_have_card_alive_lazy(INDEX_PILE, *redraw))
            }
            ActionObservation::ExchangeDraw { card, .. } => {
                self.player_can_have_cards_alive_lazy(INDEX_PILE, card)
            }
            ActionObservation::ExchangeChoice {
                player_id,
                relinquish,
            } => {
                let player_dead = self.public_constraints()[*player_id].len() as u8;
                let mut required = [0u8; MAX_CARD_PERMS_ONE];
                relinquish.iter().for_each(|c| required[*c as usize] += 1);
                // println!("relinquish: {:?}", relinquish);
                // println!("required: {:?}", required);
                if let ActionInfo::ExchangeDraw { draw } =
                    self.constraint_history[self.constraint_history.len() - 1].action_info()
                {
                    // println!("draw: {:?}", draw);
                    draw.iter().for_each(|c| {
                        if required[*c as usize] > 0 {
                            required[*c as usize] -= 1
                        }
                    });
                }
                // println!("required: {:?}", required);
                let total_cards = required.iter().sum::<u8>();
                // println!("total_cards: {:?}", total_cards);
                if total_cards == 0 {
                    true
                } else if total_cards + player_dead > 2 {
                    false
                } else {
                    // if updated {..} just check the state
                    let mut cards = Vec::with_capacity(2);
                    for (c, &req_count) in required.iter().enumerate() {
                        let card = Card::try_from(c as u8).unwrap();
                        cards.extend(std::iter::repeat_n(card, req_count as usize));
                    }
                    self.player_can_have_cards_alive_lazy(*player_id, &cards)
                }
            }
            _ => true,
        }
    }
}
