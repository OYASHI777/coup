use crate::traits::prob_manager::coup_analysis::CoupGeneration;
use crate::prob_manager::engine::models_prelude::*;
use crate::prob_manager::engine::models::game_state::GameData;
use crate::history_public::ActionObservation;
pub struct UninformedTracker;
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
                    .player_targets_kill(state.player_turn)
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
                    .player_targets_kill(state.player_turn)
                    .map(|p| ActionObservation::Assassinate { player_id: state.player_turn, opposing_player_id: p })
                );
                output.extend(
                    data
                    .player_targets_kill(state.player_turn)
                    .map(|p| ActionObservation::Coup { player_id: state.player_turn, opposing_player_id: p })
                );
                output
            },
            10.. => {
                let mut output = Vec::with_capacity(5);
                output.extend(
                    data
                    .player_targets_kill(state.player_turn)
                    .map(|p| ActionObservation::Coup{ player_id: state.player_turn, opposing_player_id: p})
                );
                output
            }
        }
    }

    fn on_end(&self, state: &End, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_coup_hit(&self, state: &CoupHit, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_invites_block(&self, state: &ForeignAidInvitesBlock, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_invites_challenge(&self, state: &ForeignAidBlockInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_challenged(&self, state: &ForeignAidBlockChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_challenger_failed(&self, state: &ForeignAidBlockChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_tax_invites_challenge(&self, state: &TaxInvitesChallenge, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_tax_challenged(&self, state: &TaxChallenged, data: &GameData) -> Vec<ActionObservation> {
        todo!()
    }

    fn on_tax_challenger_failed(&self, state: &TaxChallengerFailed, data: &GameData) -> Vec<ActionObservation> {
        todo!()
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