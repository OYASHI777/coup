use crate::traits::prob_manager::coup_analysis::CoupTraversal;
use crate::history_public::{ActionObservation, Card};

enum EngineState {
    TurnStart,
    TurnStartCoup,
    // Player Chosen => TurnStart/TurnStartCoup/ForcedCoup
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
}

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