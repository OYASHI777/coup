use crate::models::{Action, CounterResult, DefaultBehaviour, Card};
use crate::game::Game;
use log;
pub struct Block {
    player_id: usize,
    opposing_player_id: usize,
    card: Card,
    result: CounterResult,
}

impl Block {
    pub fn new(player_id: usize, opposing_player_id: usize, card: Card) -> Self{
        Block{
            player_id,
            opposing_player_id,
            card,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for Block {
    fn execute(&mut self, game: &mut Game) {

        log::info!("{}",format!("Player {} Blocks Player {} with a {:?}", self.player_id, self.opposing_player_id, self.card));
        // Poll Challenge
        let challenge_options: [Action; 2];
        challenge_options = match self.card {
            Card::Contessa => [Action::Pass, Action::ChallengeContessa],
            Card::Duke => [Action::Pass, Action::ChallengeDuke],
            Card::Captain => [Action::Pass, Action::ChallengeCaptain],
            Card::Ambassador => [Action::Pass, Action::ChallengeAmbassador],
            _ => panic!("Invalid Option")
        };
        let challenge_result: CounterResult;

        challenge_result = game.round_robin_poll_counterplay(self.player_id, &challenge_options);
        match challenge_result {
            CounterResult::Success => {
                // Challenge Succeeded and Block Fails
                log::info!("{}",format!("Player {} Fails to Block Player {} with the {:?}", self.player_id, self.opposing_player_id, self.card));
                self.result = CounterResult::Failure;
            },
            CounterResult::Failure => {
                // Challenge Fails and Block Succeeds
                log::info!("{}",format!("Player {} Succeeds in Blocking Player {} with the {:?}", self.player_id, self.opposing_player_id, self.card));
                self.result = CounterResult::Success;
            },
        }
    }
    fn can_be_blocked(&self) -> bool {
        false
    }
    fn can_be_challenged(&self) -> bool {
        true
    }
    fn get_result(&self) -> CounterResult {
        self.result
    }
}
