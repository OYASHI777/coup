use crate::traits::prob_manager::coup_analysis::CoupGeneration;
use crate::prob_manager::engine::models_prelude::*;
use crate::prob_manager::engine::models::game_state::GameData;
use crate::history_public::{ActionObservation, Card};

/// This is a class created purely for documentation purposes
/// It outlines all the possible moves legal without considering any information
/// other than coins a player has and their lives.
pub struct UninformedTracker;

impl UninformedTracker {
    #[inline(always)]
    pub fn discard(&self, player: usize) -> Vec<ActionObservation> {
        vec![
            ActionObservation::Discard { player_id: player, card: [Card::Ambassador, Card::Ambassador], no_cards: 1 },
            ActionObservation::Discard { player_id: player, card: [Card::Assassin, Card::Assassin], no_cards: 1 },
            ActionObservation::Discard { player_id: player, card: [Card::Captain, Card::Captain], no_cards: 1 },
            ActionObservation::Discard { player_id: player, card: [Card::Duke, Card::Duke], no_cards: 1 },
            ActionObservation::Discard { player_id: player, card: [Card::Contessa, Card::Contessa], no_cards: 1 },
        ]
    }
}

// TODO: Refactor Steal to register remove the amount!
impl CoupGeneration for UninformedTracker {
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
                    .map(|p| ActionObservation::Steal { player_id: state.player_turn, opposing_player_id: p, amount: 7 })
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
                    .map(|p| ActionObservation::Steal { player_id: state.player_turn, opposing_player_id: p, amount: 7 })
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
        // participants is all false here indicating that we aren't trying every combination of players who wish to block
        // opposing_player == final_actioner indicates nobody blocks
        let mut output = Vec::with_capacity(6);
        output.extend(
            data
            .players_alive()
            .map(|p| ActionObservation::CollectiveBlock { participants: [false; 6], opposing_player_id: state.player_turn, final_actioner: p })
        );
        output
    }

    fn on_foreign_aid_block_invites_challenge(&self, state: &ForeignAidBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(6);
        output.extend(
            data
            .players_alive()
            .map(|p| ActionObservation::CollectiveChallenge { participants: [false; 6], opposing_player_id: state.player_blocking, final_actioner: p })
        );
        output
    }

    fn on_foreign_aid_block_challenged(&self, state: &ForeignAidBlockChallenged, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(5);
        output.push(ActionObservation::RevealRedraw { player_id: state.player_blocking, reveal: Card::Duke, redraw: Card::Duke });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Ambassador, Card::Ambassador], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Assassin, Card::Assassin], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Captain, Card::Captain], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_blocking, card: [Card::Contessa, Card::Contessa], no_cards: 1 });
        output
    }

    fn on_foreign_aid_block_challenger_failed(&self, state: &ForeignAidBlockChallengerFailed, _data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_tax_invites_challenge(&self, state: &TaxInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(6);
        output.extend(
            data
            .players_alive()
            .map(|p| ActionObservation::CollectiveChallenge { participants: [false; 6], opposing_player_id: state.player_turn, final_actioner: p })
        );
        output
    }

    fn on_tax_challenged(&self, state: &TaxChallenged, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(5);
        output.push(ActionObservation::RevealRedraw { player_id: state.player_turn, reveal: Card::Duke, redraw: Card::Duke });
        output.push(ActionObservation::Discard { player_id: state.player_turn, card: [Card::Ambassador, Card::Ambassador], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_turn, card: [Card::Assassin, Card::Assassin], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_turn, card: [Card::Captain, Card::Captain], no_cards: 1 });
        output.push(ActionObservation::Discard { player_id: state.player_turn, card: [Card::Contessa, Card::Contessa], no_cards: 1 });
        output
    }

    fn on_tax_challenger_failed(&self, state: &TaxChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        self.discard(state.player_challenger)
    }

    fn on_steal_invites_challenge(&self, state: &StealInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_challenged(&self, state: &StealChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_challenger_failed(&self, state: &StealChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_invites_block(&self, state: &StealInvitesBlock, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_block_invites_challenge(&self, state: &StealBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_block_challenged(&self, state: &StealBlockChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_steal_block_challenger_failed(&self, state: &StealBlockChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_invites_challenge(&self, state: &ExchangeInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_drawing(&self, state: &ExchangeDrawing, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_drawn(&self, state: &ExchangeDrawn, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_challenged(&self, state: &ExchangeChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_exchange_challenger_failed(&self, state: &ExchangeChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_invites_challenge(&self, state: &AssassinateInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_invites_block(&self, state: &AssassinateInvitesBlock, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_invites_challenge(&self, state: &AssassinateBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_challenged(&self, state: &AssassinateBlockChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_challenger_failed(&self, state: &AssassinateBlockChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_succeeded(&self, state: &AssassinateSucceeded, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_challenged(&self, state: &AssassinateChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_assassinate_challenger_failed(&self, state: &AssassinateChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }
}