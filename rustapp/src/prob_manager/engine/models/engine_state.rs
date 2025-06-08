use crate::history_public::ActionObservation;

use super::turn_start::*;
use super::coup::*;
use super::exchange::*;
use super::foreign_aid::*;
use super::steal::*;
use super::tax::*;
use super::assassinate::*;
pub enum EngineState {
    // Assassin => (-3 Coins)
    // Coup => (-7 Coins)
    TurnStart(TurnStart),
    // Discard => TurnStart/TurnStartCoup/ForcedCoup
    CoupHit(CoupHit),
    // No Block => TurnStart/TurnStartCoup/ForcedCoup (+2 Coins)
    // Block => ForeignAidBlockInvitesChallenge
    ForeignAidInvitesBlock(ForeignAidInvitesBlock),
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
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
    fn next(self, action: &ActionObservation) -> EngineState;
    fn prev(self, action: &ActionObservation) -> EngineState;
}