use std::marker::PhantomData;

use crate::history_public::{ActionObservation, Card};
use crate::prob_manager::constants::MAX_GAME_LENGTH;
use crate::prob_manager::engine::constants::{
    DEFAULT_PLAYER_LIVES, INDEX_PILE, MAX_CARDS_DISCARD, MAX_CARD_PERMS_ONE, MAX_HAND_SIZE_PLAYER,
    MAX_PLAYERS_INCL_PILE,
};
use crate::prob_manager::engine::models::game_state::GameData;
use crate::prob_manager::engine::models_prelude::*;
use crate::prob_manager::tracker::collater::Collator;
use crate::traits::prob_manager::coup_analysis::{CoupGeneration, CoupTraversal};

// TODO: Clean amount stolen out from the ActionObservation
const THIS_VALUE_DOES_NOT_MATTER: u8 = 77; // When the state transitions to challenge, amount stolen is not stored

// TODO: Unique ExchangeDraw vs Agent Choice
// TODO: Unique ExchangeDraw is implemented
// Functionality needed:
//      - CFR Traversal
//          - [d] Create another that uses card counter
//              - Make backtracking_prob_hybrid over InfoArray and InfoArrayBit
//                  - Use some kind of indexing instead of passing out the array
//                  - backtracking_prob_hybrid.generate_inferred_constraints() will require some iteration
//      - Actual Game Traversal
//          - [a] Check and modify legal moves accordingly
//      - Agent interface (move ingress/egress to process chance stuff and round robin)
//          - [b] Add filter/round robin interfaces
//          - [c] Chance node drawing interface
//              - will need generic for UniqueMove (generation of all possible moves)
//              - will need generic for Receiving move and providing random chance
//      - Move targetting players modes
//          - All unique actions (for UI)
//              - Controller can then compress and suggest Steal then choose Player
//      - Suggested Move Handler (Controller)
//          - Somehow can accomodate different challenge types with an added interface?
// TODO: Receive Collective Indicator vs All Final Actioners vs All Responses
// TODO: Receive Action and decide the chance stuff
// TODO: Create trait to process incoming moves!
// TODO: Consider making backtracking_prob_hybrid card_count_manager take history as an input
/// This is a class outlines all the possible legal moves based on revealled cards?
/// Well the test is with perfect information (4), but it seems the implementation was really
/// for a naive game (2) based on revealed cards only without card counting
/// TODO: Split this into (4) & (2)
///     - The test fails because its implemented as (2)
/// TODO: 4 stages of info
///     1) Uninformed -> Player lives & Coins
///     2) Naive -> (1) + Dead Cards
///     3) Counting -> (2) + Card counting
///     4) Informed -> Perfect knowledge of all cards
#[derive(Debug)]
pub struct InformedTracker<C>
where
    C: Collator,
{
    history: Vec<ActionObservation>,
    pub public_constraints: Vec<Vec<Card>>,
    pub inferred_constraints: Vec<Vec<Card>>,
    card_counts: [u8; MAX_CARD_PERMS_ONE],
    marker_collator: PhantomData<C>,
}

impl<C> Default for InformedTracker<C>
where
    C: Collator,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> InformedTracker<C>
where
    C: Collator,
{
    pub fn new() -> Self {
        InformedTracker::<C> {
            history: Vec::with_capacity(MAX_GAME_LENGTH),
            public_constraints: (0..MAX_PLAYERS_INCL_PILE)
                .map(|_| Vec::with_capacity(2))
                .collect::<Vec<_>>(),
            inferred_constraints: (0..MAX_PLAYERS_INCL_PILE)
                .map(|_| Vec::with_capacity(4))
                .collect::<Vec<_>>(),
            card_counts: [3; MAX_CARD_PERMS_ONE],
            marker_collator: PhantomData,
        }
    }
    #[inline(always)]
    pub fn discard(&self, player: usize) -> Vec<ActionObservation> {
        self.inferred_constraints[player]
            .iter()
            .copied()
            .map(|c| ActionObservation::Discard {
                player_id: player,
                card: [c, c],
                no_cards: 1,
            })
            .collect()
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
    pub fn reveal_or_discard(&self, player: usize, card_reveal: Card) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(4);
        if self.inferred_constraints[player].contains(&card_reveal) {
            let mut pile_cards = self.inferred_constraints[INDEX_PILE].clone();
            pile_cards.push(card_reveal);
            pile_cards.sort_unstable();
            pile_cards.dedup();
            output.extend(pile_cards.iter().map(|c| ActionObservation::RevealRedraw {
                player_id: player,
                reveal: card_reveal,
                redraw: *c,
            }));
        } else {
            for card in self.inferred_constraints[player].iter() {
                output.push(ActionObservation::Discard {
                    player_id: player,
                    card: [*card; 2],
                    no_cards: 1,
                })
            }
        }
        output
    }
    pub fn reveal_or_discard_all(
        &self,
        player: usize,
        card_reveal: Card,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(4);
        if self.inferred_constraints[player].contains(&card_reveal) {
            let mut pile_cards = self.inferred_constraints[INDEX_PILE].clone();
            pile_cards.push(card_reveal);
            pile_cards.sort_unstable();
            pile_cards.dedup();
            output.extend(pile_cards.iter().map(|c| ActionObservation::RevealRedraw {
                player_id: player,
                reveal: card_reveal,
                redraw: *c,
            }));
        } else {
            // Discard all
            if self.inferred_constraints[player].len() == 1 {
                output.push(ActionObservation::Discard {
                    player_id: player,
                    card: [self.inferred_constraints[player][0]; MAX_CARDS_DISCARD],
                    no_cards: 1,
                })
            } else {
                output.push(ActionObservation::Discard {
                    player_id: player,
                    card: [
                        self.inferred_constraints[player][0],
                        self.inferred_constraints[player][1],
                    ],
                    no_cards: 2,
                })
            }
        }
        output
    }
}

impl<C> CoupTraversal for InformedTracker<C>
where
    C: Collator,
{
    fn start_public(&mut self, _player: usize) {
        unimplemented!();
    }

    fn start_private(&mut self, _player: usize, _cards: &[Card; MAX_HAND_SIZE_PLAYER]) {
        unimplemented!();
    }

    fn start_known(&mut self, player_cards: &[Vec<Card>]) {
        self.public_constraints.iter_mut().for_each(|v| v.clear());
        self.inferred_constraints.iter_mut().for_each(|v| v.clear());
        player_cards
            .iter()
            .enumerate()
            .for_each(|(i, v)| self.inferred_constraints[i].extend(v));
    }

    fn push_ao_public(&mut self, _action: &ActionObservation) {
        unimplemented!(
            "Informed Tracker is only intended to support moves with private information!"
        );
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }

    fn push_ao_private(&mut self, action: &ActionObservation) {
        match action {
            ActionObservation::Discard {
                player_id,
                card,
                no_cards,
            } => {
                for i in 0..*no_cards {
                    if let Some(pos) = self.inferred_constraints[*player_id]
                        .iter()
                        .rposition(|c| *c == card[i])
                    {
                        self.inferred_constraints[*player_id].swap_remove(pos);
                        self.public_constraints[*player_id].push(card[i]);
                        self.card_counts[card[i] as usize] -= 1;
                    } else {
                        debug_assert!(false, "Card not found!");
                    }
                }
            }
            ActionObservation::RevealRedraw {
                player_id,
                reveal,
                redraw,
            } => {
                if let Some(pos_i) = self.inferred_constraints[*player_id]
                    .iter()
                    .rposition(|c| *c == *reveal)
                {
                    if let Some(pos_j) = self.inferred_constraints[INDEX_PILE]
                        .iter()
                        .rposition(|c| *c == *redraw)
                    {
                        self.inferred_constraints[*player_id].swap_remove(pos_i);
                        self.inferred_constraints[6].swap_remove(pos_j);
                        self.inferred_constraints[6].push(*reveal);
                        self.inferred_constraints[*player_id].push(*redraw);
                    } else {
                        debug_assert!(*reveal == *redraw, "Card not found!");
                    }
                } else {
                    debug_assert!(false, "Card not found!");
                }
            }
            ActionObservation::ExchangeDraw { player_id, card } => {
                for d in card.iter() {
                    if let Some(pos) = self.inferred_constraints[6].iter().rposition(|c| *c == *d) {
                        self.inferred_constraints[6].swap_remove(pos);
                        self.inferred_constraints[*player_id].push(*d);
                    }
                }
            }
            ActionObservation::ExchangeChoice {
                player_id,
                relinquish,
            } => {
                for d in relinquish.iter() {
                    if let Some(pos) = self.inferred_constraints[*player_id]
                        .iter()
                        .rposition(|c| *c == *d)
                    {
                        self.inferred_constraints[*player_id].swap_remove(pos);
                        self.inferred_constraints[INDEX_PILE].push(*d);
                    }
                }
            }
            _ => {}
        }
    }

    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_private(action);
    }

    fn pop(&mut self) {
        if let Some(action) = self.history.pop() {
            match action {
                ActionObservation::Discard {
                    player_id,
                    card,
                    no_cards,
                } => {
                    for i in 0..no_cards {
                        if let Some(pos) = self.public_constraints[player_id]
                            .iter()
                            .rposition(|c| *c == card[i])
                        {
                            self.public_constraints[player_id].swap_remove(pos);
                            self.inferred_constraints[player_id].push(card[i]);
                            self.card_counts[card[i] as usize] += 1;
                        } else {
                            debug_assert!(false, "Card not found!");
                        }
                    }
                }
                ActionObservation::RevealRedraw {
                    player_id,
                    reveal,
                    redraw,
                } => {
                    if let Some(pos_i) = self.inferred_constraints[INDEX_PILE]
                        .iter()
                        .rposition(|c| *c == reveal)
                    {
                        if let Some(pos_j) = self.inferred_constraints[player_id]
                            .iter()
                            .rposition(|c| *c == redraw)
                        {
                            self.inferred_constraints[INDEX_PILE].swap_remove(pos_i);
                            self.inferred_constraints[player_id].swap_remove(pos_j);
                            self.inferred_constraints[player_id].push(reveal);
                            self.inferred_constraints[INDEX_PILE].push(redraw);
                        } else {
                            debug_assert!(false, "Card not found!");
                        }
                    } else {
                        debug_assert!(false, "Card not found!");
                    }
                }
                ActionObservation::ExchangeDraw { player_id, card } => {
                    for d in card.iter() {
                        if let Some(pos) = self.inferred_constraints[player_id]
                            .iter()
                            .rposition(|c| *c == *d)
                        {
                            self.inferred_constraints[player_id].swap_remove(pos);
                            self.inferred_constraints[INDEX_PILE].push(*d);
                        }
                    }
                }
                ActionObservation::ExchangeChoice {
                    player_id,
                    relinquish,
                } => {
                    for d in relinquish.iter() {
                        if let Some(pos) = self.inferred_constraints[INDEX_PILE]
                            .iter()
                            .rposition(|c| *c == *d)
                        {
                            self.inferred_constraints[INDEX_PILE].swap_remove(pos);
                            self.inferred_constraints[player_id].push(*d);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// TODO: Add another trait for Coup move collection
// TODO: Refactor Steal to register remove the amount!
impl<C> CoupGeneration for InformedTracker<C>
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
        state: &StealBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        let mut pile_cards = self.inferred_constraints[INDEX_PILE].clone();
        pile_cards.sort_unstable();
        pile_cards.dedup();
        if self.inferred_constraints[state.player_blocking].contains(&state.card_blocker) {
            if !pile_cards.contains(&state.card_blocker) {
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: state.card_blocker,
                    redraw: state.card_blocker,
                });
            }
            for card in pile_cards.iter() {
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: state.card_blocker,
                    redraw: *card,
                });
            }
        }
        output.extend(
            self.inferred_constraints[state.player_blocking]
                .iter()
                .filter_map(|player_card| {
                    (*player_card != state.card_blocker).then_some(ActionObservation::Discard {
                        player_id: state.player_blocking,
                        card: [*player_card; MAX_CARDS_DISCARD],
                        no_cards: 1,
                    })
                }),
        );
        output
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
        let mut output = Vec::with_capacity(3);
        for i in 0..self.inferred_constraints[INDEX_PILE].len() {
            for j in (i + 1)..self.inferred_constraints[INDEX_PILE].len() {
                let action = ActionObservation::ExchangeDraw {
                    player_id: state.player_turn,
                    card: [
                        self.inferred_constraints[INDEX_PILE][i],
                        self.inferred_constraints[INDEX_PILE][j],
                    ],
                };
                if !output.contains(&action) {
                    output.push(action);
                }
            }
        }
        output
    }

    fn on_exchange_drawn(&self, state: &ExchangeDrawn, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(6);
        for i in 0..self.inferred_constraints[state.player_turn].len() {
            for j in (i + 1)..self.inferred_constraints[state.player_turn].len() {
                let action = ActionObservation::ExchangeChoice {
                    player_id: state.player_turn,
                    relinquish: [
                        self.inferred_constraints[state.player_turn][i],
                        self.inferred_constraints[state.player_turn][j],
                    ],
                };
                if !output.contains(&action) {
                    output.push(action);
                }
            }
        }
        output
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
        state: &AssassinateInvitesBlock,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        // TODO: Consider if this should be BlockAssassinate + Discard?
        let mut output = Vec::with_capacity(2);
        output.push(ActionObservation::BlockAssassinate {
            player_id: state.player_blocking,
            opposing_player_id: state.player_turn,
        });
        if self.inferred_constraints[state.player_blocking]
            .iter()
            .any(|c| *c != Card::Contessa)
        {
            output.push(ActionObservation::BlockAssassinate {
                player_id: state.player_blocking,
                opposing_player_id: state.player_blocking,
            });
        }
        output
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
        state: &AssassinateSucceeded,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(DEFAULT_PLAYER_LIVES);
        for card in self.inferred_constraints[state.player_blocking]
            .iter()
            .copied()
        {
            if card != Card::Contessa {
                output.push(ActionObservation::Discard {
                    player_id: state.player_blocking,
                    card: [card; MAX_CARDS_DISCARD],
                    no_cards: 1,
                });
            }
        }
        output
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
        // In this case if the player_challenger == player_blocking, they can still discard Contessa if they have double Contessa
        // Although it should be [PRUNE] if player only has 1 Contessa
        self.discard(state.player_challenger)
    }
}

#[cfg(test)]
mod tests {
    #[warn(unused_imports)]
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    use crate::prob_manager::{
        engine::{
            constants::{COST_ASSASSINATE, COST_COUP, COUNT_PER_CHARACTER, MAX_CARDS_IN_GAME},
            fsm_engine::FSMEngine,
        },
        tracker::collater::Indicate,
    };

    // Implement 100 random games and test if all items returned by CoupGeneration are valid
    fn all_cards_legal(
        inferred_constraints: &Vec<Vec<Card>>,
        suggested_moves: &Vec<ActionObservation>,
    ) {
        suggested_moves.iter().for_each(|action| {
            match action {
                ActionObservation::Discard {
                    player_id, card, ..
                } => assert!(card
                    .iter()
                    .all(|c| inferred_constraints[*player_id].contains(c))),
                ActionObservation::RevealRedraw {
                    player_id,
                    reveal,
                    redraw,
                } => {
                    assert!(inferred_constraints[*player_id].contains(reveal));
                    assert!(inferred_constraints[INDEX_PILE].contains(redraw));
                }
                ActionObservation::ExchangeDraw { card, .. } => {
                    // This is a weak test.. it needs to have BOTH
                    assert!(
                        inferred_constraints[INDEX_PILE].contains(&card[0])
                            || inferred_constraints[INDEX_PILE].contains(&card[1])
                    );
                }
                ActionObservation::ExchangeChoice {
                    player_id,
                    relinquish,
                } => {
                    // This is a weak check...
                    assert!(relinquish.iter().all(|c| {
                        inferred_constraints[INDEX_PILE].contains(&c)
                            || inferred_constraints[*player_id].contains(&c)
                    }));
                }
                _ => {}
            }
        })
    }
    fn all_coins_legal(data: &GameData, suggested_moves: &Vec<ActionObservation>) {
        suggested_moves.iter().for_each(|action| match action {
            ActionObservation::Assassinate { player_id, .. } => {
                assert!(data.coins[*player_id] >= COST_ASSASSINATE)
            }
            ActionObservation::Coup { player_id, .. } => {
                assert!(data.coins[*player_id] >= COST_COUP)
            }
            _ => {}
        })
    }
    fn all_cards_constraint_valid(
        public_constraints: &Vec<Vec<Card>>,
        inferred_constraints: &Vec<Vec<Card>>,
    ) {
        let mut card_counts: [u8; MAX_CARD_PERMS_ONE] = [0; MAX_CARD_PERMS_ONE];
        public_constraints
            .iter()
            .flatten()
            .for_each(|c| card_counts[*c as usize] += 1);
        inferred_constraints
            .iter()
            .flatten()
            .for_each(|c| card_counts[*c as usize] += 1);
        assert!(card_counts.iter().sum::<u8>() == MAX_CARDS_IN_GAME);
        assert!(card_counts
            .iter()
            .all(|count| *count == COUNT_PER_CHARACTER));
    }
    #[test]
    fn test_random_games() {
        const RANDOM_GAME_COUNT: usize = 1000;
        for _ in 0..RANDOM_GAME_COUNT {
            let mut engine = FSMEngine::new();
            let mut tracker: InformedTracker<Indicate> = InformedTracker::new();
            // TODO: RANDOMIZE
            let starting_cards = vec![
                vec![Card::Ambassador, Card::Ambassador],
                vec![Card::Ambassador, Card::Assassin],
                vec![Card::Assassin, Card::Assassin],
                vec![Card::Captain, Card::Captain],
                vec![Card::Captain, Card::Duke],
                vec![Card::Duke, Card::Duke],
                vec![Card::Contessa, Card::Contessa, Card::Contessa],
            ];
            // TODO: RANDOMIZE
            let player = 0;
            engine.start_private(
                player,
                &[starting_cards[player][0], starting_cards[player][1]],
            );
            tracker.start_known(&starting_cards);
            // TODO: test if game actually ends
            while !engine.game_end() {
                let suggested_moves = engine.generate_legal_moves(&tracker);
                all_cards_legal(&tracker.inferred_constraints, &suggested_moves);
                all_coins_legal(&engine.state.game_data, &suggested_moves);
                all_cards_constraint_valid(
                    &tracker.public_constraints,
                    &tracker.inferred_constraints,
                );

                let mut rng = thread_rng();
                if let Some(action) = suggested_moves.choose(&mut rng) {
                    engine.push_ao_private(action);
                    tracker.push_ao_private(action);
                } else {
                    panic!("suggested_moves is empty");
                }
            }
        }
    }
}
