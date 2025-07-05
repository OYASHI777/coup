use crate::prob_manager::constants::MAX_GAME_LENGTH;
use crate::prob_manager::engine::constants::{MAX_CARD_PERMS_ONE, MAX_PLAYERS_EXCL_PILE};
use crate::traits::prob_manager::coup_analysis::{CoupGeneration, CoupTraversal};
use crate::prob_manager::engine::models_prelude::*;
use crate::prob_manager::engine::models::game_state::GameData;
use crate::history_public::{ActionObservation, Card};

pub const TEMP_DUMMY_STEAL_AMT: u8 = 77;

// TODO: Unique ExchangeDraw vs Agent Choice
// TODO: Unique ExchangeDraw is implemented
// Functionality needed:
//      - CFR Traversal
//          - [d] Create another that uses card counter
//      - Actual Game Traversal
//          - [a] Check and modify legal moves accordingly
//      - Agent interface (move ingress/egress to process chance stuff and round robin)
//          - [b] Add filter/round robin interfaces
//          - [c] Chance node drawing interface
//              - will need generic for UniqueMove (generation of all possible moves)
//              - will need generic for Receiving move and providing random chance
// TODO: Receive Collective Indicator vs All Final Actioners vs All Responses
// TODO: Receive Action and decide the chance stuff
// TODO: Create trait to process incoming moves!
/// This is a class created purely for documentation purposes
/// It outlines all the possible moves legal without considering any information
/// other than coins a player has and their lives.
pub struct InformedTracker {
    history: Vec<ActionObservation>,
    public_constraints: Vec<Vec<Card>>,
    inferred_constraints: Vec<Vec<Card>>,
    card_counts: [u8; 5],
}

impl InformedTracker {
    pub fn new() -> Self {
        InformedTracker { 
            history: Vec::with_capacity(MAX_GAME_LENGTH),
            public_constraints: vec![Vec::with_capacity(2); 7], 
            inferred_constraints: vec![Vec::with_capacity(4); 7],
            card_counts: [3; 5],
        }
    }
    #[inline(always)]
    pub fn discard(&self, player: usize) -> Vec<ActionObservation> {
        self.inferred_constraints[player]
        .iter()
        .copied()
        .map(|c| ActionObservation::Discard { player_id: player, card: [c, c], no_cards: 1 })
        .collect()
    }
    #[inline(always)]
    pub fn challenge_invite(&self, player: usize, data: &GameData) -> Vec<ActionObservation> {
        // participants here indicates the players alive that can block
        // final_actioner here is just a place holder
        let participants = std::array::from_fn(|p| data.influence()[p] > 0);
        vec![
            ActionObservation::CollectiveChallenge { participants, opposing_player_id: player, final_actioner: player }
        ]
    }
    pub fn block_invite(&self, player: usize, data: &GameData) -> Vec<ActionObservation> {
        // participants here indicates the players alive that can block
        // final_actioner here is just a place holder
        let participants = std::array::from_fn(|p| data.influence()[p] > 0);
        vec![
            ActionObservation::CollectiveBlock { participants, opposing_player_id: player, final_actioner: player }
        ]
    }
    // TODO: This current implementation provides all Unique moves,
    // TODO: for actual gameplay it should be done in the move intake interface
    pub fn reveal_or_discard(&self, player: usize, card_reveal: Card) -> Vec<ActionObservation> {
        // TODO: Consider Random Sample vs Single Sample vs Unique Samples
        let mut output = Vec::with_capacity(5);
        let mut player_cards = self.inferred_constraints[player].clone();
        let mut pile_cards = self.inferred_constraints[player].clone();
        player_cards.sort_unstable();
        player_cards.dedup();
        for card in player_cards.iter() {
            if *card == card_reveal {
                pile_cards.push(card_reveal);
                pile_cards.sort_unstable();
                pile_cards.dedup();
                output.extend(
                    pile_cards
                    .iter()
                    .map(|c| ActionObservation::RevealRedraw { player_id: player, reveal: card_reveal, redraw: *c })
                );
            } else {
                output.push(ActionObservation::Discard { player_id: player, card: [*card; 2], no_cards: 1 })
            }
        }
        output
    }
}

impl CoupTraversal for InformedTracker {
    fn start_public(&mut self, _player: usize) {
        unimplemented!();
    }

    fn start_private(&mut self, _player: usize, _cards: &[Card; 2]) {
        unimplemented!();
    }

    fn start_known(&mut self, player_cards: &Vec<Vec<Card>>) {
        self.public_constraints
        .iter_mut()
        .for_each(|v| v.clear());
        self.inferred_constraints
        .iter_mut()
        .for_each(|v| v.clear());
        player_cards
        .iter()
        .enumerate()
        .for_each(|(i, v)| self.inferred_constraints[i].extend(v));
    }

    fn push_ao_public(&mut self, _action: &ActionObservation) {
        unimplemented!("Informed Tracker is only intended to support moves with private information!");
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }

    fn push_ao_private(&mut self, action: &ActionObservation) {
        match action {
            ActionObservation::Discard { player_id, card, no_cards } => {
                for i in 0..*no_cards {
                    if let Some(pos) = self.inferred_constraints[*player_id].iter().rposition(|c| *c == card[i]) {
                        self.inferred_constraints[*player_id].swap_remove(pos);
                        self.public_constraints[*player_id].push(card[i]);
                        self.card_counts[card[i] as usize] -= 1;
                    } else {
                        debug_assert!(false, "Card not found!");
                    }
                }
            },
            ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                if let Some(pos_i) = self.inferred_constraints[*player_id].iter().rposition(|c| *c == *reveal) {
                    if let Some(pos_j) = self.inferred_constraints[6].iter().rposition(|c| *c == *redraw) {
                        self.inferred_constraints[*player_id].swap_remove(pos_i);
                        self.inferred_constraints[6].swap_remove(pos_j);
                        self.inferred_constraints[6].push(*reveal);
                        self.inferred_constraints[*player_id].push(*redraw);
                    } else {
                        debug_assert!(false, "Card not found!");
                    }
                } else {
                    debug_assert!(false, "Card not found!");
                }
            },
            ActionObservation::ExchangeDraw { player_id, card } => {
                for d in card.iter() {
                    if let Some(pos) = self.inferred_constraints[6].iter().rposition(|c| *c == *d) {
                        self.inferred_constraints[6].swap_remove(pos);
                        self.inferred_constraints[*player_id].push(*d);
                    }
                }
            },
            ActionObservation::ExchangeChoice { player_id, relinquish } => {
                for d in relinquish.iter() {
                    if let Some(pos) = self.inferred_constraints[*player_id].iter().rposition(|c| *c == *d) {
                        self.inferred_constraints[*player_id].swap_remove(pos);
                        self.inferred_constraints[6].push(*d);
                    }
                }
            },
            _ => {},
        }
    }

    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_private(action);
    }

    fn pop(&mut self) {
        if let Some(action) = self.history.pop() {
            match action {
                ActionObservation::Discard { player_id, card, no_cards } => {
                    for i in 0..no_cards {
                        if let Some(pos) = self.public_constraints[player_id].iter().rposition(|c| *c == card[i]) {
                            self.public_constraints[player_id].swap_remove(pos);
                            self.inferred_constraints[player_id].push(card[i]);
                            self.card_counts[card[i] as usize] += 1;
                        } else {
                            debug_assert!(false, "Card not found!");
                        }
                    }
                },
                ActionObservation::RevealRedraw { player_id, reveal, redraw } => {
                    if let Some(pos_i) = self.inferred_constraints[6].iter().rposition(|c| *c == reveal) {
                        if let Some(pos_j) = self.inferred_constraints[player_id].iter().rposition(|c| *c == redraw) {
                            self.inferred_constraints[6].swap_remove(pos_i);
                            self.inferred_constraints[player_id].swap_remove(pos_j);
                            self.inferred_constraints[player_id].push(reveal);
                            self.inferred_constraints[6].push(redraw);
                        } else {
                            debug_assert!(false, "Card not found!");
                        }
                    } else {
                        debug_assert!(false, "Card not found!");
                    }
                },
                ActionObservation::ExchangeDraw { player_id, card } => {
                    for d in card.iter() {
                        if let Some(pos) = self.inferred_constraints[player_id].iter().rposition(|c| *c == *d) {
                            self.inferred_constraints[player_id].swap_remove(pos);
                            self.inferred_constraints[6].push(*d);
                        }
                    }
                },
                ActionObservation::ExchangeChoice { player_id, relinquish } => {
                    for d in relinquish.iter() {
                        if let Some(pos) = self.inferred_constraints[6].iter().rposition(|c| *c == *d) {
                            self.inferred_constraints[6].swap_remove(pos);
                            self.inferred_constraints[player_id].push(*d);
                        }
                    }
                },
                _ => {},
            }
        }
    }
}

// TODO: Add another trait for Coup move collection
// TODO: Refactor Steal to register remove the amount!
impl CoupGeneration for InformedTracker {
    fn on_turn_start(&self, state: &TurnStart, data: &GameData) -> Vec<ActionObservation> {
        match data.coins()[state.player_turn] {
            0..=6 => {
                let mut output = Vec::with_capacity(1 + 1 + 1 + 1 + 5 + 5);
                output.push(ActionObservation::Income { player_id: state.player_turn });
                output.push(ActionObservation::ForeignAid { player_id: state.player_turn });
                output.push(ActionObservation::Tax { player_id: state.player_turn });
                output.push(ActionObservation::Exchange { player_id: state.player_turn });
                output.extend(
                    data
                    .player_targets_steal(state.player_turn)
                    .map(|p| ActionObservation::Steal { player_id: state.player_turn, opposing_player_id: p, amount: TEMP_DUMMY_STEAL_AMT })
                    // Steal amount is handled in engine not in move!
                );
                output.extend(
                    data
                    .player_targets_alive(state.player_turn)
                    .map(|p| ActionObservation::Assassinate { player_id: state.player_turn, opposing_player_id: p })
                );
                output
            },
            7..=9 => {
                let mut output = Vec::with_capacity(1 + 1 + 1 + 1 + 5 + 5 + 5);
                output.push(ActionObservation::Income { player_id: state.player_turn });
                output.push(ActionObservation::ForeignAid { player_id: state.player_turn });
                output.push(ActionObservation::Tax { player_id: state.player_turn });
                output.push(ActionObservation::Exchange { player_id: state.player_turn });
                output.extend(
                    data
                    .player_targets_steal(state.player_turn)
                    .map(|p| ActionObservation::Steal { player_id: state.player_turn, opposing_player_id: p, amount: TEMP_DUMMY_STEAL_AMT })
                    // Steal amount is handled in engine not in move!
                );
                output.extend(
                    data
                    .player_targets_alive(state.player_turn)
                    .map(|p| ActionObservation::Assassinate { player_id: state.player_turn, opposing_player_id: p })
                );
                output.extend(
                    data
                    .player_targets_alive(state.player_turn)
                    .map(|p| ActionObservation::Coup { player_id: state.player_turn, opposing_player_id: p })
                );
                output
            },
            10.. => {
                let mut output = Vec::with_capacity(5);
                output.extend(
                    data
                    .player_targets_alive(state.player_turn)
                    .map(|p| ActionObservation::Coup{ player_id: state.player_turn, opposing_player_id: p})
                );
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

    fn on_foreign_aid_invites_block(&self, state: &ForeignAidInvitesBlock, data: &GameData) -> Vec<ActionObservation> {
        self.block_invite(state.player_turn, data)
    }

    fn on_foreign_aid_block_invites_challenge(&self, state: &ForeignAidBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_foreign_aid_block_challenged(&self, state: &ForeignAidBlockChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_blocking, Card::Duke)
    }

    fn on_foreign_aid_block_challenger_failed(&self, state: &ForeignAidBlockChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_tax_invites_challenge(&self, state: &TaxInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_turn, data)
    }

    fn on_tax_challenged(&self, state: &TaxChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Duke)
    }

    fn on_tax_challenger_failed(&self, state: &TaxChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_steal_invites_challenge(&self, state: &StealInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_turn, data)
    }

    fn on_steal_challenged(&self, state: &StealChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Captain)
    }

    fn on_steal_challenger_failed(&self, state: &StealChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_steal_invites_block(&self, state: &StealInvitesBlock, _data: &GameData) -> Vec<ActionObservation> {
        vec![
            ActionObservation::BlockSteal { player_id: state.player_blocking, opposing_player_id: state.player_turn, card: Card::Ambassador },
            ActionObservation::BlockSteal { player_id: state.player_blocking, opposing_player_id: state.player_turn, card: Card::Captain },
            // This represents not Blocking
            ActionObservation::BlockSteal { player_id: state.player_blocking, opposing_player_id: state.player_blocking, card: Card::Captain },
        ]
    }

    fn on_steal_block_invites_challenge(&self, state: &StealBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_steal_block_challenged(&self, state: &StealBlockChallenged, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2* MAX_CARD_PERMS_ONE - 1);
        let mut pile_cards = self.inferred_constraints[6].clone();
        pile_cards.sort_unstable();
        pile_cards.dedup();
        if self.inferred_constraints[state.player_blocking].contains(&state.card_blocker) {
            if !pile_cards.contains(&state.card_blocker) {
                output.push(ActionObservation::RevealRedraw { player_id: state.player_blocking, reveal: state.card_blocker, redraw: state.card_blocker });
            }
            for card in pile_cards.iter() {
                output.push(ActionObservation::RevealRedraw { player_id: state.player_blocking, reveal: state.card_blocker, redraw: *card });
            }
        }
        output.extend(
            self.inferred_constraints[state.player_blocking]
            .iter()
            .filter_map(|player_card| (*player_card != state.card_blocker).then_some(
                ActionObservation::Discard { player_id: state.player_blocking, card: [*player_card; 2], no_cards: 1 }
            ))
        );
        output
    }

    fn on_steal_block_challenger_failed(&self, state: &StealBlockChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_exchange_invites_challenge(&self, state: &ExchangeInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_turn, data)
    }

    fn on_exchange_drawing(&self, state: &ExchangeDrawing, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(3);
        for i in 0..self.inferred_constraints[6].len() as u8 {
            for j in (i + 1)..self.inferred_constraints[6].len() as u8 {
                let action = ActionObservation::ExchangeDraw { player_id: state.player_turn, card: [Card::try_from(i).unwrap(), Card::try_from(j).unwrap()] };
                if !output.contains(&action) {
                    output.push(action);
                }
            }
        }
        output
    }
    
    fn on_exchange_drawn(&self, state: &ExchangeDrawn, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(6);
        for i in 0..self.inferred_constraints[state.player_turn].len() as u8 {
            for j in (i + 1)..self.inferred_constraints[state.player_turn].len() as u8 {
                let action = ActionObservation::ExchangeChoice { player_id: state.player_turn, relinquish: [Card::try_from(i).unwrap(), Card::try_from(j).unwrap()] };
                if !output.contains(&action) {
                    output.push(action);
                }
            }
        }
        output
    }

    fn on_exchange_challenged(&self, state: &ExchangeChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Ambassador)
    }

    fn on_exchange_challenger_failed(&self, state: &ExchangeChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_assassinate_invites_challenge(&self, state: &AssassinateInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_turn, data)
    }

    fn on_assassinate_invites_block(&self, state: &AssassinateInvitesBlock, _data: &GameData) -> Vec<ActionObservation> {
        vec![
            ActionObservation::BlockAssassinate { player_id: state.player_blocking, opposing_player_id: state.player_turn },
            ActionObservation::BlockAssassinate { player_id: state.player_blocking, opposing_player_id: state.player_blocking },
        ]
    }

    fn on_assassinate_block_invites_challenge(&self, state: &AssassinateBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_assassinate_block_challenged(&self, state: &AssassinateBlockChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_blocking, Card::Contessa)
    }

    fn on_assassinate_block_challenger_failed(&self, state: &AssassinateBlockChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_assassinate_succeeded(&self, state: &AssassinateSucceeded, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(MAX_CARD_PERMS_ONE);
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Ambassador, Card::Ambassador], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Assassin, Card::Assassin], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Captain, Card::Captain], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Duke, Card::Duke], no_cards: 1 });
        output
    }

    fn on_assassinate_challenged(&self, state: &AssassinateChallenged, _data: &GameData) -> Vec<ActionObservation> {
        self.reveal_or_discard(state.player_turn, Card::Assassin)
    }
    
    fn on_assassinate_challenger_failed(&self, state: &AssassinateChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        // In this case if the player_challenger == player_blocking, they can still discard Contessa if they have double Contessa
        // Although it should be [PRUNE] if player only has 1 Contessa
        self.discard(state.player_challenger)
    }
}