use super::models::engine_state::EngineState;
use crate::history_public::{ActionObservation, Card};
use crate::prob_manager::engine::models::engine_state::CoupTransition;
use crate::prob_manager::engine::models::game_state::GameState;
use crate::prob_manager::engine::models_prelude::End;
use crate::traits::prob_manager::coup_analysis::{CoupGeneration, CoupTraversal};

pub trait Node {
    fn dispatch(&self) -> bool;
}
// TODO: Write test for same resources after push() then pop()
// TODO: Write mode for simulation and mode for registering player moves and randomly choosing?
// TODO: Make a better api pls omg, just write a random game to see how bad it is
pub struct FSMEngine {
    pub history: Vec<ActionObservation>,
    pub history_state: Vec<EngineState>,
    pub state: GameState,
}

impl Default for FSMEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FSMEngine {
    /// Generates an FSMEngine
    pub fn new() -> Self {
        FSMEngine {
            history: Vec::with_capacity(128),
            history_state: Vec::with_capacity(128),
            state: GameState::new(0),
        }
    }

    pub fn generate_legal_moves<T>(&self, tracker: &T) -> Vec<ActionObservation>
    where
        T: CoupGeneration,
    {
        match self.state.engine_state {
            EngineState::TurnStart(state) => tracker.on_turn_start(&state, &self.state.game_data),
            EngineState::End(state) => tracker.on_end(&state, &self.state.game_data),
            EngineState::CoupHit(state) => tracker.on_coup_hit(&state, &self.state.game_data),
            EngineState::ForeignAidInvitesBlock(state) => {
                tracker.on_foreign_aid_invites_block(&state, &self.state.game_data)
            }
            EngineState::ForeignAidBlockInvitesChallenge(state) => {
                tracker.on_foreign_aid_block_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::ForeignAidBlockChallenged(state) => {
                tracker.on_foreign_aid_block_challenged(&state, &self.state.game_data)
            }
            EngineState::ForeignAidBlockChallengerFailed(state) => {
                tracker.on_foreign_aid_block_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::TaxInvitesChallenge(state) => {
                tracker.on_tax_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::TaxChallenged(state) => {
                tracker.on_tax_challenged(&state, &self.state.game_data)
            }
            EngineState::TaxChallengerFailed(state) => {
                tracker.on_tax_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::StealInvitesChallenge(state) => {
                tracker.on_steal_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::StealChallenged(state) => {
                tracker.on_steal_challenged(&state, &self.state.game_data)
            }
            EngineState::StealChallengerFailed(state) => {
                tracker.on_steal_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::StealInvitesBlock(state) => {
                tracker.on_steal_invites_block(&state, &self.state.game_data)
            }
            EngineState::StealBlockInvitesChallenge(state) => {
                tracker.on_steal_block_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::StealBlockChallenged(state) => {
                tracker.on_steal_block_challenged(&state, &self.state.game_data)
            }
            EngineState::StealBlockChallengerFailed(state) => {
                tracker.on_steal_block_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::ExchangeInvitesChallenge(state) => {
                tracker.on_exchange_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::ExchangeDrawing(state) => {
                tracker.on_exchange_drawing(&state, &self.state.game_data)
            }
            EngineState::ExchangeDrawn(state) => {
                tracker.on_exchange_drawn(&state, &self.state.game_data)
            }
            EngineState::ExchangeChallenged(state) => {
                tracker.on_exchange_challenged(&state, &self.state.game_data)
            }
            EngineState::ExchangeChallengerFailed(state) => {
                tracker.on_exchange_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::AssassinateInvitesChallenge(state) => {
                tracker.on_assassinate_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::AssassinateInvitesBlock(state) => {
                tracker.on_assassinate_invites_block(&state, &self.state.game_data)
            }
            EngineState::AssassinateBlockInvitesChallenge(state) => {
                tracker.on_assassinate_block_invites_challenge(&state, &self.state.game_data)
            }
            EngineState::AssassinateBlockChallenged(state) => {
                tracker.on_assassinate_block_challenged(&state, &self.state.game_data)
            }
            EngineState::AssassinateBlockChallengerFailed(state) => {
                tracker.on_assassinate_block_challenger_failed(&state, &self.state.game_data)
            }
            EngineState::AssassinateSucceeded(state) => {
                tracker.on_assassinate_succeeded(&state, &self.state.game_data)
            }
            EngineState::AssassinateChallenged(state) => {
                tracker.on_assassinate_challenged(&state, &self.state.game_data)
            }
            EngineState::AssassinateChallengerFailed(state) => {
                tracker.on_assassinate_challenger_failed(&state, &self.state.game_data)
            }
        }
    }

    pub fn game_end(&self) -> bool {
        self.state.engine_state == EngineState::End(End {})
    }
}
impl CoupTraversal for FSMEngine {
    fn start_public(&mut self, player: usize) {
        self.history.clear();
        self.history_state.clear();
        self.state = GameState::start(player);
        self.history_state.push(self.state.engine_state);
    }
    /// Currently engine does not store private information, but tracker does
    /// so default to public
    fn start_private(&mut self, player: usize, _cards: &[crate::history_public::Card; 2]) {
        self.start_public(player);
    }

    fn start_known(&mut self, _cards: &[Vec<Card>]) {
        unimplemented!()
    }

    /// Update's Engine's state
    fn push_ao_public(&mut self, action: &ActionObservation) {
        self.state.engine_state = self
            .state
            .engine_state
            .state_leave_update(action, &mut self.state.game_data);
        EngineState::action_update(action, &mut self.state.game_data);
        self.state
            .engine_state
            .state_enter_update(&mut self.state.game_data);
        self.history_state.push(self.state.engine_state);
        self.history.push(*action);
    }

    fn push_ao_public_lazy(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }
    /// Update's Engine's state with private information
    /// Currently engine does not store private information, but tracker does
    /// so default to public
    fn push_ao_private(&mut self, action: &ActionObservation) {
        self.push_ao_public(action);
    }

    fn push_ao_private_lazy(&mut self, _action: &ActionObservation) {
        todo!()
    }

    fn pop(&mut self) {
        // Case when history is empty => Off or ignore
        if let Some(action) = self.history.pop() {
            self.history_state.pop();
            // This must come first so Discard can add the influence back
            self.state
                .engine_state
                .state_enter_reverse(&mut self.state.game_data);
            EngineState::action_reverse(&action, &mut self.state.game_data);
            if let Some(prev_state) = self.history_state.last() {
                self.state.engine_state = *prev_state;
                // This is affected by Discard and must come after
                self.state
                    .engine_state
                    .state_leave_reverse(&action, &mut self.state.game_data);
            } else {
                panic!("Pop not working")
            }
        }
    }
}
