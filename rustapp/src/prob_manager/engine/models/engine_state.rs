use crate::history_public::ActionObservation;
use crate::prob_manager::engine::constants::COST_ASSASSINATE;
use crate::prob_manager::engine::constants::COST_COUP;
use crate::prob_manager::engine::constants::GAIN_INCOME;
use crate::prob_manager::engine::models::game_state::GameData;

use super::turn_start::*;
use super::coup::*;
use super::end::*;
use super::exchange::*;
use super::foreign_aid::*;
use super::steal::*;
use super::tax::*;
use super::assassinate::*;
#[derive(Copy, Clone)]
pub enum EngineState {
    // Assassin => (-3 Coins)
    // Coup => (-7 Coins)
    TurnStart(TurnStart),
    // End of Game
    End(End),
    // Discard => TurnStart/TurnStartCoup/ForcedCoup
    CoupHit(CoupHit),
    // No Block => TurnStart/TurnStartCoup/ForcedCoup (+2 Coins)
    // Block => ForeignAidBlockInvitesChallenge
    ForeignAidInvitesBlock(ForeignAidInvitesBlock),
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    // Challenged => ForeignAidBlockChallenged
    ForeignAidBlockInvitesChallenge(ForeignAidBlockInvitesChallenge),
    // Challenged RevealRedraw => ForeignAidBlockChallengerFailed
    // Challenged Discard (Not Duke) => TurnStart/TurnStartCoup/ForcedCoup (+2 Coins)
    ForeignAidBlockChallenged(ForeignAidBlockChallenged),
    // Discard => TurnStart/TurnStartCoup/ForcedCoup
    ForeignAidBlockChallengerFailed(ForeignAidBlockChallengerFailed),
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup (+3 Coins)
    // Challenge => TaxChallenged
    TaxInvitesChallenge(TaxInvitesChallenge), 
    // Challenged RevealRedraw => TaxChallengerFailed (+3 Coins)
    // Challenged Discard (Not Duke)=> TurnStart/TurnStartCoup/ForcedCoup
    TaxChallenged(TaxChallenged),
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    TaxChallengerFailed(TaxChallengerFailed),
    // No Challenge => StealInvitesBlock
    // Challenge => StealChallenged
    StealInvitesChallenge(StealInvitesChallenge),
    // Challenged RevealRedraw => StealChallengerFailed
    // Challenged Discard (Not Captain) => TurnStart/TurnStartCoup/ForcedCoup
    StealChallenged(StealChallenged),
    // Challenger Discard => StealInvitesBlock
    StealChallengerFailed(StealChallengerFailed),
    // Blocker Dead (from challenge) => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | =0 Coins)
    // Blocker Alive
    //      Block (CAP/AMB) => StealBlockInvitesChallenge
    //      No Block => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | +min(2,x) Coins)
    StealInvitesBlock(StealInvitesBlock),
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    // Challenge => StealBlockChallenged
    StealBlockInvitesChallenge(StealBlockInvitesChallenge),
    // Challenged RevealRedraw => StealBlockChallengerFailed
    // Challenged Discard => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | -min(2,x) Coins)
    StealBlockChallenged(StealBlockChallenged),
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    StealBlockChallengerFailed(StealBlockChallengerFailed),
    // No Challenge => ExchangeDrawn
    // Challenge => ExchangeChallenged
    ExchangeInvitesChallenge(ExchangeInvitesChallenge),
    // ExchangeDraw => ExchangeDrawn
    ExchangeDrawing(ExchangeDrawing),
    // ExchangeChoice => TurnStart/TurnStartCoup/ForcedCoup
    ExchangeDrawn(ExchangeDrawn),
    // Challenged RevealRedraw => ExchangeChallengerFailed
    // Challenged Discard (Not Exchange) => TurnStart/TurnStartCoup/ForcedCoup
    ExchangeChallenged(ExchangeChallenged),
    // Challenger Discard => ExchangeDrawn
    ExchangeChallengerFailed(ExchangeChallengerFailed),
    // No Challenge => AssassinateInvitesBlock
    // Challenge => AssassinateChallenged
    AssassinateInvitesChallenge(AssassinateInvitesChallenge),
    // Blocker Dead (from challenge) => TurnStart/TurnStartCoup/ForcedCoup
    // Blocker Alive
    //      Block (CON) => AssassinateBlockInvitesChallenge
    //      No Block => AssassinateSucceeded
    AssassinateInvitesBlock(AssassinateInvitesBlock),
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    // Challenge => AssassinateBlockChallenged
    AssassinateBlockInvitesChallenge(AssassinateBlockInvitesChallenge),
    // Challenged RevealRedraw => AssassinateBlockChallengerFailed
    // Challenged Discard (ALL CARDS && NOT Contessa) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateBlockChallenged(AssassinateBlockChallenged),
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateBlockChallengerFailed(AssassinateBlockChallengerFailed),
    // Blocker Discard (NOT Contessa) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateSucceeded(AssassinateSucceeded),
    // Challenged RevealRedraw => AssassinateChallengerFailed
    // Challenged Discard (Not Assassin) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateChallenged(AssassinateChallenged),
    // Challenger Discard => AssassinateInvitesBlock
    AssassinateChallengerFailed(AssassinateChallengerFailed),
}

pub trait CoupTransition {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState;
    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData);
    fn action_update(action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Income { player_id } => {
                game_data.coins[*player_id] += GAIN_INCOME;
                // game_data.next_player();
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                game_data.coins[*player_id] -= COST_ASSASSINATE;
            },
            ActionObservation::Coup { player_id, opposing_player_id } => {
                game_data.coins[*player_id] -= COST_COUP;
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.influence[*player_id] -= *no_cards as u8;
            },
            _ => {}
        }
    }
    fn action_reverse(action: &ActionObservation, game_data: &mut GameData) {
        match action {
            ActionObservation::Income { player_id } => {
                game_data.coins[*player_id] -= GAIN_INCOME;
                // game_data.prev_player();
            },
            ActionObservation::Assassinate { player_id, opposing_player_id } => {
                game_data.coins[*player_id] += COST_ASSASSINATE;
            },
            ActionObservation::Coup { player_id, opposing_player_id } => {
                game_data.coins[*player_id] += COST_COUP;
            },
            ActionObservation::Discard { player_id, card, no_cards } => {
                game_data.influence[*player_id] += *no_cards as u8;
            },
            _ => {}
        }
    }
}

impl CoupTransition for EngineState {
    fn state_leave_update(&self, action: &ActionObservation, game_data: &mut GameData) -> EngineState {
        match self {
            EngineState::TurnStart(turn_start) => turn_start.state_leave_update(action, game_data),
            EngineState::End(end) => end.state_leave_update(action, game_data),
            EngineState::CoupHit(coup_hit) => coup_hit.state_leave_update(action, game_data),
            EngineState::ForeignAidInvitesBlock(foreign_aid_invites_block) => foreign_aid_invites_block.state_leave_update(action, game_data),
            EngineState::ForeignAidBlockInvitesChallenge(foreign_aid_block_invites_challenge) => foreign_aid_block_invites_challenge.state_leave_update(action, game_data),
            EngineState::ForeignAidBlockChallenged(foreign_aid_block_challenged) => foreign_aid_block_challenged.state_leave_update(action, game_data),
            EngineState::ForeignAidBlockChallengerFailed(foreign_aid_block_challenger_failed) => foreign_aid_block_challenger_failed.state_leave_update(action, game_data),
            EngineState::TaxInvitesChallenge(tax_invites_challenge) => tax_invites_challenge.state_leave_update(action, game_data),
            EngineState::TaxChallenged(tax_challenged) => tax_challenged.state_leave_update(action, game_data),
            EngineState::TaxChallengerFailed(tax_challenger_failed) => tax_challenger_failed.state_leave_update(action, game_data),
            EngineState::StealInvitesChallenge(steal_invites_challenge) => steal_invites_challenge.state_leave_update(action, game_data),
            EngineState::StealChallenged(steal_challenged) => steal_challenged.state_leave_update(action, game_data),
            EngineState::StealChallengerFailed(steal_challenger_failed) => steal_challenger_failed.state_leave_update(action, game_data),
            EngineState::StealInvitesBlock(steal_invites_block) => steal_invites_block.state_leave_update(action, game_data),
            EngineState::StealBlockInvitesChallenge(steal_block_invites_challenge) => steal_block_invites_challenge.state_leave_update(action, game_data),
            EngineState::StealBlockChallenged(steal_block_challenged) => steal_block_challenged.state_leave_update(action, game_data),
            EngineState::StealBlockChallengerFailed(steal_block_challenger_failed) => steal_block_challenger_failed.state_leave_update(action, game_data),
            EngineState::ExchangeInvitesChallenge(exchange_invites_challenge) => exchange_invites_challenge.state_leave_update(action, game_data),
            EngineState::ExchangeDrawing(exchange_drawing) => exchange_drawing.state_leave_update(action, game_data),
            EngineState::ExchangeDrawn(exchange_drawn) => exchange_drawn.state_leave_update(action, game_data),
            EngineState::ExchangeChallenged(exchange_challenged) => exchange_challenged.state_leave_update(action, game_data),
            EngineState::ExchangeChallengerFailed(exchange_challenger_failed) => exchange_challenger_failed.state_leave_update(action, game_data),
            EngineState::AssassinateInvitesChallenge(assassinate_invites_challenge) => assassinate_invites_challenge.state_leave_update(action, game_data),
            EngineState::AssassinateInvitesBlock(assassinate_invites_block) => assassinate_invites_block.state_leave_update(action, game_data),
            EngineState::AssassinateBlockInvitesChallenge(assassinate_block_invites_challenge) => assassinate_block_invites_challenge.state_leave_update(action, game_data),
            EngineState::AssassinateBlockChallenged(assassinate_block_challenged) => assassinate_block_challenged.state_leave_update(action, game_data),
            EngineState::AssassinateBlockChallengerFailed(assassinate_block_challenger_failed) => assassinate_block_challenger_failed.state_leave_update(action, game_data),
            EngineState::AssassinateSucceeded(assassinate_succeeded) => assassinate_succeeded.state_leave_update(action, game_data),
            EngineState::AssassinateChallenged(assassinate_challenged) => assassinate_challenged.state_leave_update(action, game_data),
            EngineState::AssassinateChallengerFailed(assassinate_challenger_failed) => assassinate_challenger_failed.state_leave_update(action, game_data),
        }
    }

    fn state_leave_reverse(&self, action: &ActionObservation, game_data: &mut GameData) {
        match self {
            EngineState::TurnStart(turn_start) => turn_start.state_leave_reverse(action, game_data),
            EngineState::End(end) => end.state_leave_reverse(action, game_data),
            EngineState::CoupHit(coup_hit) => coup_hit.state_leave_reverse(action, game_data),
            EngineState::ForeignAidInvitesBlock(foreign_aid_invites_block) => foreign_aid_invites_block.state_leave_reverse(action, game_data),
            EngineState::ForeignAidBlockInvitesChallenge(foreign_aid_block_invites_challenge) => foreign_aid_block_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::ForeignAidBlockChallenged(foreign_aid_block_challenged) => foreign_aid_block_challenged.state_leave_reverse(action, game_data),
            EngineState::ForeignAidBlockChallengerFailed(foreign_aid_block_challenger_failed) => foreign_aid_block_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::TaxInvitesChallenge(tax_invites_challenge) => tax_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::TaxChallenged(tax_challenged) => tax_challenged.state_leave_reverse(action, game_data),
            EngineState::TaxChallengerFailed(tax_challenger_failed) => tax_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::StealInvitesChallenge(steal_invites_challenge) => steal_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::StealChallenged(steal_challenged) => steal_challenged.state_leave_reverse(action, game_data),
            EngineState::StealChallengerFailed(steal_challenger_failed) => steal_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::StealInvitesBlock(steal_invites_block) => steal_invites_block.state_leave_reverse(action, game_data),
            EngineState::StealBlockInvitesChallenge(steal_block_invites_challenge) => steal_block_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::StealBlockChallenged(steal_block_challenged) => steal_block_challenged.state_leave_reverse(action, game_data),
            EngineState::StealBlockChallengerFailed(steal_block_challenger_failed) => steal_block_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::ExchangeInvitesChallenge(exchange_invites_challenge) => exchange_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::ExchangeDrawing(exchange_drawing) => exchange_drawing.state_leave_reverse(action, game_data),
            EngineState::ExchangeDrawn(exchange_drawn) => exchange_drawn.state_leave_reverse(action, game_data),
            EngineState::ExchangeChallenged(exchange_challenged) => exchange_challenged.state_leave_reverse(action, game_data),
            EngineState::ExchangeChallengerFailed(exchange_challenger_failed) => exchange_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::AssassinateInvitesChallenge(assassinate_invites_challenge) => assassinate_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::AssassinateInvitesBlock(assassinate_invites_block) => assassinate_invites_block.state_leave_reverse(action, game_data),
            EngineState::AssassinateBlockInvitesChallenge(assassinate_block_invites_challenge) => assassinate_block_invites_challenge.state_leave_reverse(action, game_data),
            EngineState::AssassinateBlockChallenged(assassinate_block_challenged) => assassinate_block_challenged.state_leave_reverse(action, game_data),
            EngineState::AssassinateBlockChallengerFailed(assassinate_block_challenger_failed) => assassinate_block_challenger_failed.state_leave_reverse(action, game_data),
            EngineState::AssassinateSucceeded(assassinate_succeeded) => assassinate_succeeded.state_leave_reverse(action, game_data),
            EngineState::AssassinateChallenged(assassinate_challenged) => assassinate_challenged.state_leave_reverse(action, game_data),
            EngineState::AssassinateChallengerFailed(assassinate_challenger_failed) => assassinate_challenger_failed.state_leave_reverse(action, game_data),
        }
    }
}