use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct Tax {
    player_id: usize,
    result: CounterResult,
}
impl Tax {
    pub fn new(player_id: usize) -> Self{
        Tax {
            player_id,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for Tax {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} uses Tax", self.player_id));
        // Poll Challenge
        let challenge_options: [Action; 2] = [Action::Pass, Action::ChallengeDuke];
        let challenge_result: CounterResult;

        challenge_result = game.round_robin_poll_counterplay(self.player_id, &challenge_options);
        
        match challenge_result {
            CounterResult::Success => {
                // Challenge Succeeded and Tax Fails
                self.result = CounterResult::Failure;
                log::info!("{}", format!("Player {}'s Tax was blocked!", self.player_id));
            },
            CounterResult::Failure => {
                // Challenge Fails and Tax Succeeds
                game.player_add_coins(self.player_id, 3);
                self.result = CounterResult::Success;
                log::info!("{}", format!("Player {}'s Tax Succeeds, they gain 3 coins!", self.player_id));
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