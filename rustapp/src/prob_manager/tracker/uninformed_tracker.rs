use crate::history_public::{ActionObservation, Card};
use crate::prob_manager::engine::constants::{MAX_CARD_PERMS_ONE, MAX_PLAYERS_EXCL_PILE};
use crate::prob_manager::engine::models::game_state::GameData;
use crate::prob_manager::engine::models_prelude::*;
use crate::traits::prob_manager::coup_analysis::CoupGeneration;

pub const TEMP_DUMMY_STEAL_AMT: u8 = 77;

/// This is a class created purely for documentation purposes
/// It outlines all the possible moves legal without considering any information
/// other than coins a player has and their lives.
pub struct UninformedTracker;

impl UninformedTracker {
    pub fn new() -> Self {
        UninformedTracker
    }
    #[inline(always)]
    pub fn discard(&self, player: usize) -> Vec<ActionObservation> {
        vec![
            ActionObservation::Discard {
                player_id: player,
                card: [Card::Ambassador, Card::Ambassador],
                no_cards: 1,
            },
            ActionObservation::Discard {
                player_id: player,
                card: [Card::Assassin, Card::Assassin],
                no_cards: 1,
            },
            ActionObservation::Discard {
                player_id: player,
                card: [Card::Captain, Card::Captain],
                no_cards: 1,
            },
            ActionObservation::Discard {
                player_id: player,
                card: [Card::Duke, Card::Duke],
                no_cards: 1,
            },
            ActionObservation::Discard {
                player_id: player,
                card: [Card::Contessa, Card::Contessa],
                no_cards: 1,
            },
        ]
    }
    #[inline(always)]
    pub fn challenge_invite(&self, player: usize, data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(MAX_PLAYERS_EXCL_PILE);
        output.extend(
            data.players_alive()
                .map(|p| ActionObservation::CollectiveChallenge {
                    participants: [false; 6],
                    opposing_player_id: player,
                    final_actioner: p,
                }),
        );
        output
    }
    pub fn block_invite(&self, player: usize, data: &GameData) -> Vec<ActionObservation> {
        // participants is all false here indicating that we aren't trying every combination of players who wish to block
        // opposing_player == final_actioner indicates nobody blocks
        let mut output = Vec::with_capacity(MAX_PLAYERS_EXCL_PILE);
        output.extend(
            data.players_alive()
                .map(|p| ActionObservation::CollectiveBlock {
                    participants: [false; 6],
                    opposing_player_id: player,
                    final_actioner: p,
                }),
        );
        output
    }
}

// TODO: Refactor Steal to register remove the amount!
impl CoupGeneration for UninformedTracker {
    fn on_turn_start(&self, state: &TurnStart, data: &GameData) -> Vec<ActionObservation> {
        match data.coins()[state.player_turn] {
            0..=6 => {
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
                            amount: TEMP_DUMMY_STEAL_AMT,
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
                            amount: TEMP_DUMMY_STEAL_AMT,
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
        self.block_invite(state.player_turn, data)
    }

    fn on_foreign_aid_block_invites_challenge(
        &self,
        state: &ForeignAidBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_foreign_aid_block_challenged(
        &self,
        state: &ForeignAidBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Duke,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Duke,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Duke,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Duke,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Duke,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
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
        self.challenge_invite(state.player_turn, data)
    }

    fn on_tax_challenged(&self, state: &TaxChallenged, _data: &GameData) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Duke,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Duke,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Duke,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Duke,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Duke,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
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
        self.challenge_invite(state.player_turn, data)
    }

    fn on_steal_challenged(
        &self,
        state: &StealChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Captain,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Captain,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Captain,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Captain,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Captain,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
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
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_steal_block_challenged(
        &self,
        state: &StealBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        match state.card_blocker {
            Card::Ambassador => {
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Ambassador,
                    redraw: Card::Ambassador,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Ambassador,
                    redraw: Card::Assassin,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Ambassador,
                    redraw: Card::Captain,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Ambassador,
                    redraw: Card::Duke,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Ambassador,
                    redraw: Card::Contessa,
                });
                output.push(ActionObservation::Discard {
                    player_id: state.player_blocking,
                    card: [Card::Captain, Card::Captain],
                    no_cards: 1,
                });
            }
            Card::Captain => {
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Captain,
                    redraw: Card::Ambassador,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Captain,
                    redraw: Card::Assassin,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Captain,
                    redraw: Card::Captain,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Captain,
                    redraw: Card::Duke,
                });
                output.push(ActionObservation::RevealRedraw {
                    player_id: state.player_blocking,
                    reveal: Card::Captain,
                    redraw: Card::Contessa,
                });
                output.push(ActionObservation::Discard {
                    player_id: state.player_blocking,
                    card: [Card::Ambassador, Card::Ambassador],
                    no_cards: 1,
                });
            }
            _ => {
                panic!("Illegal Move!")
            }
        }
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
    }

    fn on_steal_block_challenger_failed(
        &self,
        state: &StealBlockChallengerFailed,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        // In this case if the player_challenger == player_blocking, they can still discard blocking card if they have 2 blocking cards
        // Although it should be [PRUNE] if player only has 1 blocking card
        self.discard(state.player_challenger)
    }

    fn on_exchange_invites_challenge(
        &self,
        state: &ExchangeInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_turn, data)
    }

    fn on_exchange_drawing(
        &self,
        state: &ExchangeDrawing,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        vec![
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Ambassador, Card::Ambassador],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Ambassador, Card::Assassin],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Ambassador, Card::Captain],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Ambassador, Card::Duke],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Ambassador, Card::Contessa],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Assassin, Card::Assassin],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Assassin, Card::Captain],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Assassin, Card::Duke],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Assassin, Card::Contessa],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Captain, Card::Captain],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Captain, Card::Duke],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Captain, Card::Contessa],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Duke, Card::Duke],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Duke, Card::Contessa],
            },
            ActionObservation::ExchangeDraw {
                player_id: state.player_turn,
                card: [Card::Contessa, Card::Contessa],
            },
        ]
    }

    fn on_exchange_drawn(&self, state: &ExchangeDrawn, _data: &GameData) -> Vec<ActionObservation> {
        vec![
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Ambassador, Card::Ambassador],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Ambassador, Card::Assassin],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Ambassador, Card::Captain],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Ambassador, Card::Duke],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Ambassador, Card::Contessa],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Assassin, Card::Assassin],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Assassin, Card::Captain],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Assassin, Card::Duke],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Assassin, Card::Contessa],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Captain, Card::Captain],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Captain, Card::Duke],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Captain, Card::Contessa],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Duke, Card::Duke],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Duke, Card::Contessa],
            },
            ActionObservation::ExchangeChoice {
                player_id: state.player_turn,
                relinquish: [Card::Contessa, Card::Contessa],
            },
        ]
    }

    fn on_exchange_challenged(
        &self,
        state: &ExchangeChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Ambassador,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Ambassador,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Ambassador,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Ambassador,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Ambassador,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
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
        self.challenge_invite(state.player_turn, data)
    }

    fn on_assassinate_invites_block(
        &self,
        state: &AssassinateInvitesBlock,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        vec![
            ActionObservation::BlockAssassinate {
                player_id: state.player_blocking,
                opposing_player_id: state.player_turn,
            },
            ActionObservation::BlockAssassinate {
                player_id: state.player_blocking,
                opposing_player_id: state.player_blocking,
            },
        ]
    }

    fn on_assassinate_block_invites_challenge(
        &self,
        state: &AssassinateBlockInvitesChallenge,
        data: &GameData,
    ) -> Vec<ActionObservation> {
        self.challenge_invite(state.player_blocking, data)
    }

    fn on_assassinate_block_challenged(
        &self,
        state: &AssassinateBlockChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Contessa,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Contessa,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Contessa,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Contessa,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_blocking,
            reveal: Card::Contessa,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output
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
        let mut output = Vec::with_capacity(MAX_CARD_PERMS_ONE);
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Assassin, Card::Assassin],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_blocking,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output
    }

    fn on_assassinate_challenged(
        &self,
        state: &AssassinateChallenged,
        _data: &GameData,
    ) -> Vec<ActionObservation> {
        let mut output = Vec::with_capacity(2 * MAX_CARD_PERMS_ONE - 1);
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Assassin,
            redraw: Card::Ambassador,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Assassin,
            redraw: Card::Assassin,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Assassin,
            redraw: Card::Captain,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Assassin,
            redraw: Card::Duke,
        });
        output.push(ActionObservation::RevealRedraw {
            player_id: state.player_turn,
            reveal: Card::Assassin,
            redraw: Card::Contessa,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Ambassador, Card::Ambassador],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Captain, Card::Captain],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Duke, Card::Duke],
            no_cards: 1,
        });
        output.push(ActionObservation::Discard {
            player_id: state.player_turn,
            card: [Card::Contessa, Card::Contessa],
            no_cards: 1,
        });
        output
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
