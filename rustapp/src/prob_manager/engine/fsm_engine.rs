use crate::traits::prob_manager::coup_analysis::CoupTraversal;
use crate::history_public::{ActionObservation, Card};
use super::models::turn_start::TurnStart;

enum EngineState {
    // Assassin => (-3 Coins)
    // Coup => (-7 Coins)
    TurnStart(TurnStart),
    // Assassin => (-3 Coins)
    // Coup => (-7 Coins)
    TurnStartCoup,
    // Player Chosen => TurnStart/TurnStartCoup/ForcedCoup (-7 Coins)
    ForcedCoup,
    // Discard => TurnStart/TurnStartCoup/ForcedCoup
    CoupHit,
    // No Block => TurnStart/TurnStartCoup/ForcedCoup (+2 Coins)
    // Block => ForeignAidBlockInvitesChallenge
    ForeignAidInvitesBlock,
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    ForeignAidBlockInvitesChallenge{player_blocking: u8},
    // Challenged RevealRedraw => ForeignAidBlockChallengerFailed
    // Challenged Discard (Not Duke) => TurnStart/TurnStartCoup/ForcedCoup (+2 Coins)
    ForeignAidBlockChallenged{player_challenger: u8, player_blocking: u8}, // TODO: [OPTIMIZE] Can find player_blocking in index before
    // Discard => TurnStart/TurnStartCoup/ForcedCoup
    ForeignAidBlockChallengerFailed{player_challenger: u8},
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup (+3 Coins)
    // Challenge => TaxChallenged
    TaxInvitesChallenge, 
    // Challenged RevealRedraw => TaxChallengerFailed (+3 Coins)
    // Challenged Discard (Not Duke)=> TurnStart/TurnStartCoup/ForcedCoup
    TaxChallenged{player_challenger: u8},
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    TaxChallengerFailed{player_challenger: u8},
    // No Challenge => StealInvitesBlock
    // Challenge => StealChallenged
    StealInvitesChallenge,
    // Challenged RevealRedraw => StealChallengerFailed
    // Challenged Discard (Not Captain) => TurnStart/TurnStartCoup/ForcedCoup
    StealChallenged{player_challenger: u8, player_blocking: u8},
    // Challenger Discard => StealInvitesBlock
    StealChallengerFailed{player_challenger: u8, player_blocking: u8},
    // Blocker Dead (from challenge) => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | =0 Coins)
    // Blocker Alive
    //      Block (CAP/AMB) => StealBlockInvitesChallenge
    //      No Block => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | +min(2,x) Coins)
    StealInvitesBlock{player_blocking: u8},
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    // Challenge => StealBlockChallenged
    StealBlockInvitesChallenge{card_blocker: Card, player_blocking: u8},
    // Challenged RevealRedraw => StealBlockChallengerFailed
    // Challenged Discard => TurnStart/TurnStartCoup/ForcedCoup (+min(2,x) Coins | -min(2,x) Coins)
    StealBlockChallenged{card_blocker: Card, player_challenger: u8, player_blocking: u8},
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    StealBlockChallengerFailed{player_challenger: u8},
    // No Challenge => AmbassadorDrawn
    // Challenge => AmbassadorChallenged
    AmbassadorInvitesChallenge,
    // ExchangeChoice => TurnStart/TurnStartCoup/ForcedCoup
    AmbassadorDrawn,
    // Challenged RevealRedraw => AmbassadorChallengerFailed
    // Challenged Discard (Not Ambassador) => TurnStart/TurnStartCoup/ForcedCoup
    AmbassadorChallenged{player_challenger: u8},
    // Challenger Discard => AmbassadorDrawn
    AmbassadorChallengerFailed{player_challenger: u8},
    // No Challenge => AssassinateInvitesBlock
    // Challenge => AssassinateChallenged
    AssassinateInvitesChallenge,
    // Blocker Dead (from challenge) => TurnStart/TurnStartCoup/ForcedCoup
    // Blocker Alive
    //      Block (CON) => AssassinateBlockInvitesChallenge
    //      No Block => AssassinateSucceeded
    AssassinateInvitesBlock{player_blocking: u8},
    // No Challenge => TurnStart/TurnStartCoup/ForcedCoup
    // Challenge => AssassinateBlockChallenged
    AssassinateBlockInvitesChallenge{player_blocking: u8},
    // Challenged RevealRedraw => AssassinateBlockChallengerFailed
    // Challenged Discard (ALL CARDS && NOT Contessa) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateBlockChallenged{player_challenger: u8, player_blocking: u8},
    // Challenger Discard => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateBlockChallengerFailed{player_challenger: u8},
    // Blocker Discard (NOT Contessa) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateSucceeded{player_blocking: u8},
    // Challenged RevealRedraw => AssassinateChallengerFailed
    // Challenged Discard (Not Assassin) => TurnStart/TurnStartCoup/ForcedCoup
    AssassinateChallenged{player_challenger: u8},
    // Challenger Discard => AssassinateInvitesBlock
    AssassinateChallengerFailed{player_challenger: u8},
}
pub trait Node {
    fn dispatch(&self) -> bool;
}
// TODO: Write test for same resources after push() then pop()
pub struct FSMEngine {
    store: Vec<EngineState>,
    player_turn: Vec<u8>,
    influence: [u8; 6],
    coins: [u8; 6],
}
impl FSMEngine {
    /// Generates an FSMEngine
    pub fn new() -> Self {
        FSMEngine { 
            store: Vec::with_capacity(128), 
            player_turn: Vec::with_capacity(128),
            influence: [0; 6], 
            coins: [0; 6], 
        }
    }
    /// returns which player's turn it is next in a round robin fashion
    /// Assumes latest influence is used and updated accurately
    pub fn next_player(&self, player_id: usize) -> usize {
        let mut current_turn: usize = (player_id + 1) % 6;
        // while self.latest_influence()[current_turn] == 0 {
        while self.influence[current_turn] == 0 {
            current_turn = (current_turn + 1) % 6;
        }
        current_turn
    }
    pub fn dispatch_example(&self) -> bool {
        if let Some(engine_state) = self.store.last() {
            match engine_state {
                EngineState::TurnStart(inner) => {
                    return inner.dispatch()
                },
                EngineState::TurnStartCoup => todo!(),
                EngineState::ForcedCoup => todo!(),
                EngineState::CoupHit => todo!(),
                EngineState::ForeignAidInvitesBlock => todo!(),
                EngineState::ForeignAidBlockInvitesChallenge { player_blocking } => todo!(),
                EngineState::ForeignAidBlockChallenged { player_challenger, player_blocking } => todo!(),
                EngineState::ForeignAidBlockChallengerFailed { player_challenger } => todo!(),
                EngineState::TaxInvitesChallenge => todo!(),
                EngineState::TaxChallenged { player_challenger } => todo!(),
                EngineState::TaxChallengerFailed { player_challenger } => todo!(),
                EngineState::StealInvitesChallenge => todo!(),
                EngineState::StealChallenged { player_challenger, player_blocking } => todo!(),
                EngineState::StealChallengerFailed { player_challenger, player_blocking } => todo!(),
                EngineState::StealInvitesBlock { player_blocking } => todo!(),
                EngineState::StealBlockInvitesChallenge { card_blocker, player_blocking } => todo!(),
                EngineState::StealBlockChallenged { card_blocker, player_challenger, player_blocking } => todo!(),
                EngineState::StealBlockChallengerFailed { player_challenger } => todo!(),
                EngineState::AmbassadorInvitesChallenge => todo!(),
                EngineState::AmbassadorDrawn => todo!(),
                EngineState::AmbassadorChallenged { player_challenger } => todo!(),
                EngineState::AmbassadorChallengerFailed { player_challenger } => todo!(),
                EngineState::AssassinateInvitesChallenge => todo!(),
                EngineState::AssassinateInvitesBlock { player_blocking } => todo!(),
                EngineState::AssassinateBlockInvitesChallenge { player_blocking } => todo!(),
                EngineState::AssassinateBlockChallenged { player_challenger, player_blocking } => todo!(),
                EngineState::AssassinateBlockChallengerFailed { player_challenger } => todo!(),
                EngineState::AssassinateSucceeded { player_blocking } => todo!(),
                EngineState::AssassinateChallenged { player_challenger } => todo!(),
                EngineState::AssassinateChallengerFailed { player_challenger } => todo!(),
            }
        }
        false
    }
}
impl CoupTraversal for FSMEngine {
    fn start_public(&mut self) {
        todo!()
    }

    fn start_private(&mut self, player: usize, cards: &[crate::history_public::Card; 2]) {
        todo!()
    }
    /// Update's Engine's state
    fn push_ao_public(&mut self, action: &ActionObservation) {
        todo!()
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        todo!()
    }
    /// Update's Engine's state with private information
    fn push_ao_private(&mut self, action: &ActionObservation) {
        todo!()
    }

    fn push_ao_private_lazy(&mut self, action: &ActionObservation) {
        todo!()
    }

    fn pop(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}