use std::marker::PhantomData;

use crate::history_public::{ActionObservation, Card};
use crate::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager;
use crate::prob_manager::constants::MAX_GAME_LENGTH;
use crate::prob_manager::engine::constants::{
    INDEX_PILE, MAX_CARDS_DISCARD, MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PLAYER,
};
use crate::prob_manager::engine::models::game_state::GameData;
use crate::prob_manager::engine::models_prelude::*;
use crate::prob_manager::models::backtrack::info_array::InfoArray;
use crate::prob_manager::tracker::collater::Collator;
use crate::traits::prob_manager::coup_analysis::{CoupGeneration, CoupTraversal};

const THIS_VALUE_DOES_NOT_MATTER: u8 = 77; // When the state transitions to challenge, amount stolen is not stored

/// Card counting tracker that uses public information only to track card counts
/// and generate appropriate moves using backtracking probability analysis.
/// Uses BackTrackCardCountManager for sophisticated card counting and probability inference.
pub struct CardCountTracker<C>
where
    C: Collator,
{
    history: Vec<ActionObservation>,
    pub backtracking_hybrid_prob: BackTrackCardCountManager<InfoArray>,
    card_counts: [u8; MAX_CARD_PERMS_ONE],
    cards_alive: Vec<Card>,
    marker_collator: PhantomData<C>,
}

impl<C> CardCountTracker<C>
where
    C: Collator,
{
    pub fn new() -> Self {
        CardCountTracker::<C> {
            history: Vec::with_capacity(MAX_GAME_LENGTH),
            backtracking_hybrid_prob: BackTrackCardCountManager::new(),
            card_counts: [3; MAX_CARD_PERMS_ONE], // 3 of each card initially
            cards_alive: vec![
                Card::Ambassador,
                Card::Assassin,
                Card::Captain,
                Card::Duke,
                Card::Contessa,
            ],
            marker_collator: PhantomData,
        }
    }

    #[inline(always)]
    pub fn discard(&self, player: usize) -> Vec<ActionObservation> {
        // Check all possible cards to see if the player could have them
        let mut possible_discards = Vec::with_capacity(MAX_CARD_PERMS_ONE);

        let (mut public_constraints, mut inferred_constraints) =
            BackTrackCardCountManager::<InfoArray>::create_buffer();
        for &card in &self.cards_alive {
            inferred_constraints[player].push(card);
            // Check if the player can have this card based on current constraints
            if self
                .backtracking_hybrid_prob
                .possible_to_have_cards_latest(&mut public_constraints, &mut inferred_constraints)
            {
                possible_discards.push(ActionObservation::Discard {
                    player_id: player,
                    card: [card, card],
                    no_cards: 1,
                });
            }
            BackTrackCardCountManager::<InfoArray>::clear_buffer(
                &mut public_constraints,
                &mut inferred_constraints,
            );
        }

        possible_discards
    }

    pub fn block_invite(&self, player: usize, data: &GameData) -> Vec<ActionObservation> {
        // participants here indicates the players alive that can block
        // final_actioner here is just a place holder
        let participants = std::array::from_fn(|p| data.influence()[p] > 0);
        vec![ActionObservation::CollectiveBlock {
            participants,
            opposing_player_id: player,
            final_actioner: player,
        }]
    }
    /// ASSUMPTION: Game rule indicates that if player has the card they MUST reveal it
    ///     => if a player discards a card, they cannot have had the card_reveal
    pub fn reveal_or_discard(&self, player: usize, card_reveal: Card) -> Vec<ActionObservation> {
        let mut cards_discard = self.cards_alive.clone();
        if let Some(pos) = cards_discard.iter().position(|c| *c == card_reveal) {
            cards_discard.swap_remove(pos);
        }
        let mut output = Vec::with_capacity(5 + 4); // 5: reveal_redraw, 4: discards
        let (mut public_constraints, mut inferred_constraints) =
            BackTrackCardCountManager::<InfoArray>::create_buffer();

        // Checking if possible for player to have card_reveal
        inferred_constraints[player].push(card_reveal);
        if self
            .backtracking_hybrid_prob
            .possible_to_have_cards_latest(&mut public_constraints, &mut inferred_constraints)
        {
            // CASE A
            // Since player returns card to pile, then redraws from pile (including card_reveal)
            // Player can always redraw the same card
            output.push(ActionObservation::RevealRedraw {
                player_id: player,
                reveal: card_reveal,
                redraw: card_reveal,
            });
            // CASE B
            // player redraws card that the pile has but is not card_reveal
            for &card in cards_discard.iter() {
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                // Checking if possible for player to have card_reveal and pile to have card
                // TODO: Handle case where player can redraw the same card
                inferred_constraints[player].push(card_reveal);
                inferred_constraints[INDEX_PILE].push(card);
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    output.push(ActionObservation::RevealRedraw {
                        player_id: player,
                        reveal: card_reveal,
                        redraw: card,
                    });
                }
            }
            // CASE C
            // Player's hand does not have card_reveal so may discard any card in hand
            // We check that the player can have a hand that does not include card_reveal
            let mut index_checked: Vec<usize> = Vec::with_capacity(MAX_CARD_PERMS_ONE - 1);
            debug_assert!(MAX_CARD_PERMS_ONE == 5);
            for (i, j) in [
                (0, 1),
                (2, 3),
                (0, 2),
                (1, 3),
                (0, 3),
                (1, 2),
                (0, 0),
                (1, 1),
                (2, 2),
                (3, 3),
            ] {
                if index_checked.len() == MAX_CARD_PERMS_ONE - 1 {
                    break;
                }
                let contains_i = index_checked.contains(&i);
                let contains_j = index_checked.contains(&j);
                if contains_i && contains_j {
                    continue;
                }
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                // Checking if possible for player to have cards_discard[i] and cards_discard[j]
                inferred_constraints[player]
                    .extend_from_slice(&[cards_discard[i], cards_discard[j]]);
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    if !contains_i {
                        index_checked.push(i);
                    }
                    if !contains_j {
                        index_checked.push(j);
                    }
                }
            }
            // This is done outside the loop above to maintain order of moves
            index_checked.sort_unstable();
            for i in index_checked {
                output.push(ActionObservation::Discard {
                    player_id: player,
                    card: [cards_discard[i], cards_discard[i]],
                    no_cards: 1,
                })
            }
        } else {
            // CASE D
            // Player's hand does not have card_reveal so may discard any card in hand
            for card in cards_discard {
                // Checking if possible for player to have card
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                // Checking if possible for player to have card
                inferred_constraints[player].push(card);
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    output.push(ActionObservation::Discard {
                        player_id: player,
                        card: [card, card],
                        no_cards: 1,
                    });
                }
            }
        }
        output
    }

    /// ASSUMPTION: Game rule indicates that if player has the card they MUST reveal it
    ///     => if a player discards a card, they cannot have had the card_reveal
    pub fn reveal_or_discard_all(
        &self,
        player: usize,
        card_reveal: Card,
    ) -> Vec<ActionObservation> {
        let mut cards_discard = self.cards_alive.clone();
        if let Some(pos) = cards_discard.iter().position(|c| *c == card_reveal) {
            cards_discard.swap_remove(pos);
        }
        let mut output = Vec::with_capacity(15); // 5: reveal_redraw, 10: discards
        let (mut public_constraints, mut inferred_constraints) =
            BackTrackCardCountManager::<InfoArray>::create_buffer();

        // Checking if possible for player to have card_reveal
        inferred_constraints[player].push(card_reveal);
        if self
            .backtracking_hybrid_prob
            .possible_to_have_cards_latest(&mut public_constraints, &mut inferred_constraints)
        {
            // CASE A
            // Since player returns card to pile, then redraws from pile (including card_reveal)
            // Player can always redraw the same card
            output.push(ActionObservation::RevealRedraw {
                player_id: player,
                reveal: card_reveal,
                redraw: card_reveal,
            });
            // CASE B
            // player redraws card that the pile has but is not card_reveal
            for &card in cards_discard.iter() {
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                // Checking if possible for player to have card_reveal and pile to have card
                // TODO: Handle case where player can redraw the same card
                inferred_constraints[player].push(card_reveal);
                inferred_constraints[INDEX_PILE].push(card);
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    output.push(ActionObservation::RevealRedraw {
                        player_id: player,
                        reveal: card_reveal,
                        redraw: card,
                    });
                }
            }
        }
        // CASE C
        // Player's hand does not have card_reveal so may discard any card in hand
        for i in 0..(MAX_CARD_PERMS_ONE - 1) {
            for j in i..(MAX_CARD_PERMS_ONE - 1) {
                // Checking if possible for player to have card
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
                inferred_constraints[player]
                    .extend_from_slice(&[cards_discard[i], cards_discard[j]]);
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    output.push(ActionObservation::Discard {
                        player_id: player,
                        card: [cards_discard[i], cards_discard[j]],
                        no_cards: 2,
                    });
                }
            }
        }
        output
    }

    /// Updates public information
    fn push_action_update(&mut self, action: &ActionObservation) {
        match action {
            ActionObservation::Discard { card, no_cards, .. } => {
                self.update_discard_card_counts(&card, *no_cards);
            }
            _ => {} // Other actions don't affect public card counts directly
        }
    }

    /// Reverts card counts based on public information
    fn pop_action_update(&mut self, action: &ActionObservation) {
        match action {
            ActionObservation::Discard { card, no_cards, .. } => {
                self.revert_discard_card_counts(&card, *no_cards);
            }
            _ => {}
        }
    }

    fn update_discard_card_counts(&mut self, card: &[Card; MAX_CARDS_DISCARD], no_cards: usize) {
        for i in 0..no_cards {
            debug_assert!(
                self.card_counts[card[i] as usize] > 0,
                "should only be able to discard an alive card"
            );
            self.card_counts[card[i] as usize] -= 1;
            if self.card_counts[card[i] as usize] == 0 {
                if let Some(pos) = self.cards_alive.iter().position(|c| *c == card[i]) {
                    self.cards_alive.swap_remove(pos);
                } else {
                    debug_assert!(false, "unable to find card!");
                }
            }
        }
    }

    fn revert_discard_card_counts(&mut self, card: &[Card; MAX_CARDS_DISCARD], no_cards: usize) {
        for i in 0..no_cards {
            if self.card_counts[card[i] as usize] == 0 {
                debug_assert!(!self.cards_alive.contains(&card[i]));
                self.cards_alive.push(card[i]);
            }
            self.card_counts[card[i] as usize] += 1;
        }
    }
}

impl<C> CoupTraversal for CardCountTracker<C>
where
    C: Collator,
{
    fn start_public(&mut self, player: usize) {
        // Reset state for public game start
        self.history.clear();
        self.card_counts = [3; MAX_CARD_PERMS_ONE];
        self.backtracking_hybrid_prob = BackTrackCardCountManager::new();
        // Delegate to the backtracking manager's start method
        self.backtracking_hybrid_prob.start_public(player);
    }

    fn start_private(&mut self, _player: usize, _cards: &[Card; MAX_HAND_SIZE_PLAYER]) {
        unimplemented!(
            "Card count tracker uses public information only and does not support private information!"
        );
    }

    fn start_known(&mut self, _player_cards: &Vec<Vec<Card>>) {
        unimplemented!("Card count tracker supports does not support this")
    }

    fn push_ao_public(&mut self, action: &ActionObservation) {
        self.history.push(action.clone());
        self.push_action_update(action);
        // Delegate to the backtracking manager
        self.backtracking_hybrid_prob.push_ao_public(action);
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.history.push(action.clone());
        self.push_action_update(action);
        // Delegate to the backtracking manager
        self.backtracking_hybrid_prob.push_ao_public_lazy(action);
    }

    fn push_ao_private(&mut self, _action: &ActionObservation) {
        unimplemented!(
            "Card count tracker uses public information only and does not support private actions!"
        );
    }

    fn push_ao_private_lazy(&mut self, _action: &ActionObservation) {
        unimplemented!(
            "Card count tracker uses public information only and does not support private actions!"
        );
    }

    fn pop(&mut self) {
        if let Some(action) = self.history.pop() {
            self.pop_action_update(&action);
            // Delegate to the backtracking manager
            self.backtracking_hybrid_prob.pop();
        }
    }
}

impl<C> CoupGeneration for CardCountTracker<C>
where
    C: Collator,
{
    fn on_turn_start(&self, state: &TurnStart, data: &GameData) -> Vec<ActionObservation> {
        match data.coins()[state.player_turn] {
            0..=2 => {
                let mut output = Vec::with_capacity(1 + 1 + 1 + 1 + 5 + 5);
                output.push(ActionObservation::Income {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::ForeignAid {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Tax {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Exchange {
                    player_id: state.player_turn,
                });
                output.extend(
                    data.player_targets_steal(state.player_turn).map(|p| {
                        ActionObservation::Steal {
                            player_id: state.player_turn,
                            opposing_player_id: p,
                            amount: THIS_VALUE_DOES_NOT_MATTER,
                        }
                    }), // Steal amount is handled in engine not in move!
                );
                output
            }
            3..=6 => {
                let mut output = Vec::with_capacity(1 + 1 + 1 + 1 + 5 + 5);
                output.push(ActionObservation::Income {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::ForeignAid {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Tax {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Exchange {
                    player_id: state.player_turn,
                });
                output.extend(
                    data.player_targets_steal(state.player_turn).map(|p| {
                        ActionObservation::Steal {
                            player_id: state.player_turn,
                            opposing_player_id: p,
                            amount: THIS_VALUE_DOES_NOT_MATTER,
                        }
                    }), // Steal amount is handled in engine not in move!
                );
                output.extend(data.player_targets_alive(state.player_turn).map(|p| {
                    ActionObservation::Assassinate {
                        player_id: state.player_turn,
                        opposing_player_id: p,
                    }
                }));
                output
            }
            7..=9 => {
                let mut output = Vec::with_capacity(1 + 1 + 1 + 1 + 5 + 5 + 5);
                output.push(ActionObservation::Income {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::ForeignAid {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Tax {
                    player_id: state.player_turn,
                });
                output.push(ActionObservation::Exchange {
                    player_id: state.player_turn,
                });
                output.extend(
                    data.player_targets_steal(state.player_turn).map(|p| {
                        ActionObservation::Steal {
                            player_id: state.player_turn,
                            opposing_player_id: p,
                            amount: THIS_VALUE_DOES_NOT_MATTER,
                        }
                    }), // Steal amount is handled in engine not in move!
                );
                output.extend(data.player_targets_alive(state.player_turn).map(|p| {
                    ActionObservation::Assassinate {
                        player_id: state.player_turn,
                        opposing_player_id: p,
                    }
                }));
                output.extend(data.player_targets_alive(state.player_turn).map(|p| {
                    ActionObservation::Coup {
                        player_id: state.player_turn,
                        opposing_player_id: p,
                    }
                }));
                output
            }
            10.. => {
                let mut output = Vec::with_capacity(5);
                output.extend(data.player_targets_alive(state.player_turn).map(|p| {
                    ActionObservation::Coup {
                        player_id: state.player_turn,
                        opposing_player_id: p,
                    }
                }));
                output
            }
        }
    }

    fn on_end(&self, _state: &End, _data: &GameData) -> Vec<ActionObservation> {
        vec![]
    }

    fn on_coup_hit(&self, state: &CoupHit, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_hit)
    }

    fn on_foreign_aid_invites_block(
        &self,
        state: &ForeignAidInvitesBlock,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::block(state.player_turn, data)
    }

    fn on_foreign_aid_block_invites_challenge(
        &self,
        state: &ForeignAidBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_blocking, data)
    }

    fn on_foreign_aid_block_challenged(
        &self,
        state: &ForeignAidBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_blocking, Card::Duke)
    }

    fn on_foreign_aid_block_challenger_failed(
        &self,
        state: &ForeignAidBlockChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_tax_invites_challenge(
        &self,
        state: &TaxInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_turn, data)
    }

    fn on_tax_challenged(&self, state: &TaxChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Duke)
    }

    fn on_tax_challenger_failed(
        &self,
        state: &TaxChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_steal_invites_challenge(
        &self,
        state: &StealInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_turn, data)
    }

    fn on_steal_challenged(
        &self,
        state: &StealChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Captain)
    }

    fn on_steal_challenger_failed(
        &self,
        state: &StealChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_steal_invites_block(
        &self,
        state: &StealInvitesBlock,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        vec![
            ActionObservation::BlockSteal {
                player_id: state.player_blocking,
                opposing_player_id: state.player_turn,
                card: Card::Ambassador,
            },
            ActionObservation::BlockSteal {
                player_id: state.player_blocking,
                opposing_player_id: state.player_turn,
                card: Card::Captain,
            },
            // This represents not Blocking
            ActionObservation::BlockSteal {
                player_id: state.player_blocking,
                opposing_player_id: state.player_blocking,
                card: Card::Captain,
            },
        ]
    }

    fn on_steal_block_invites_challenge(
        &self,
        state: &StealBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_blocking, data)
    }

    fn on_steal_block_challenged(
        &self,
        _state: &StealBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_block_challenger_failed(
        &self,
        state: &StealBlockChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_exchange_invites_challenge(
        &self,
        state: &ExchangeInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_turn, data)
    }

    fn on_exchange_drawing(
        &self,
        state: &ExchangeDrawing,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(15);
        let (mut public_constraints, mut inferred_constraints) =
            BackTrackCardCountManager::<InfoArray>::create_buffer();
        for i in 0..MAX_CARD_PERMS_ONE {
            for j in i..MAX_CARD_PERMS_ONE {
                if self.backtracking_hybrid_prob.possible_to_have_cards_latest(
                    &mut public_constraints,
                    &mut inferred_constraints,
                ) {
                    output.push(ActionObservation::ExchangeDraw {
                        player_id: state.player_turn,
                        card: [
                            Card::try_from(i as u8).unwrap(),
                            Card::try_from(j as u8).unwrap(),
                        ],
                    });
                }
                BackTrackCardCountManager::<InfoArray>::clear_buffer(
                    &mut public_constraints,
                    &mut inferred_constraints,
                );
            }
        }
        output
    }
    // TODO: Figure out if can just check as per normal? How does the backtracker work for exchangedraw?
    fn on_exchange_drawn(
        &self,
        _state: &ExchangeDrawn,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_challenged(
        &self,
        state: &ExchangeChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Ambassador)
    }

    fn on_exchange_challenger_failed(
        &self,
        state: &ExchangeChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_assassinate_invites_challenge(
        &self,
        state: &AssassinateInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_turn, data)
    }

    fn on_assassinate_invites_block(
        &self,
        _state: &AssassinateInvitesBlock,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_invites_challenge(
        &self,
        state: &AssassinateBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        C::challenge(state.player_blocking, data)
    }

    fn on_assassinate_block_challenged(
        &self,
        state: &AssassinateBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.reveal_or_discard_all(state.player_blocking, Card::Contessa)
    }

    fn on_assassinate_block_challenger_failed(
        &self,
        state: &AssassinateBlockChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_assassinate_succeeded(
        &self,
        _state: &AssassinateSucceeded,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_challenged(
        &self,
        state: &AssassinateChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Assassin)
    }

    fn on_assassinate_challenger_failed(
        &self,
        state: &AssassinateChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }
}
