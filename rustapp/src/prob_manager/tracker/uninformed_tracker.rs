use crate::traits::prob_manager::coup_analysis::CoupGeneration;

pub struct UninformedTracker;

impl CoupGeneration for UninformedTracker {
    fn on_turn_start(&self, state: &crate::prob_manager::engine::models::turn_start::TurnStart, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_end(&self, state: &crate::prob_manager::engine::models::end::End, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_coup_hit(&self, state: &crate::prob_manager::engine::models::coup::CoupHit, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_invites_block(&self, state: &crate::prob_manager::engine::models::foreign_aid::ForeignAidInvitesBlock, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_invites_challenge(&self, state: &crate::prob_manager::engine::models::foreign_aid::ForeignAidBlockInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_challenged(&self, state: &crate::prob_manager::engine::models::foreign_aid::ForeignAidBlockChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_foreign_aid_block_challenger_failed(&self, state: &crate::prob_manager::engine::models::foreign_aid::ForeignAidBlockChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_tax_invites_challenge(&self, state: &crate::prob_manager::engine::models::tax::TaxInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_tax_challenged(&self, state: &crate::prob_manager::engine::models::tax::TaxChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_tax_challenger_failed(&self, state: &crate::prob_manager::engine::models::tax::TaxChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_invites_challenge(&self, state: &crate::prob_manager::engine::models::steal::StealInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_challenged(&self, state: &crate::prob_manager::engine::models::steal::StealChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_challenger_failed(&self, state: &crate::prob_manager::engine::models::steal::StealChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_invites_block(&self, state: &crate::prob_manager::engine::models::steal::StealInvitesBlock, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_block_invites_challenge(&self, state: &crate::prob_manager::engine::models::steal::StealBlockInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_block_challenged(&self, state: &crate::prob_manager::engine::models::steal::StealBlockChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_steal_block_challenger_failed(&self, state: &crate::prob_manager::engine::models::steal::StealBlockChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_exchange_invites_challenge(&self, state: &crate::prob_manager::engine::models::exchange::ExchangeInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_exchange_drawing(&self, state: &crate::prob_manager::engine::models::exchange::ExchangeDrawing, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_exchange_drawn(&self, state: &crate::prob_manager::engine::models::exchange::ExchangeDrawn, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_exchange_challenged(&self, state: &crate::prob_manager::engine::models::exchange::ExchangeChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_exchange_challenger_failed(&self, state: &crate::prob_manager::engine::models::exchange::ExchangeChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_invites_challenge(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_invites_block(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateInvitesBlock, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_invites_challenge(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateBlockInvitesChallenge, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_challenged(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateBlockChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_block_challenger_failed(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateBlockChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_succeeded(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateSucceeded, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_challenged(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateChallenged, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }

    fn on_assassinate_challenger_failed(&self, state: &crate::prob_manager::engine::models::assassinate::AssassinateChallengerFailed, data: &crate::prob_manager::engine::models::game_state::GameData) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}