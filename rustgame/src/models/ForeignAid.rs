use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct ForeignAid {
    player_id: usize,
    result: CounterResult
}
impl ForeignAid {
    pub fn new(player_id: usize) -> Self{
        ForeignAid{
            player_id,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for ForeignAid {
    fn execute(&mut self, game: &mut Game) {

        log::info!("{}", format!("Player {} uses ForeignAid", self.player_id));
        let block_options: [Action; 2] = [Action::Pass, Action::BlockForeignAid];
        let mut block_result: CounterResult;

        block_result = game.round_robin_poll_counterplay(self.player_id, &block_options);
        
        match block_result {
            CounterResult::Success => {
                // Another Player's Block Succeeded Foreign Aid Fails
                self.result = CounterResult::Failure;
                log::info!("{}", format!("Player {}'s ForeignAid Fails", self.player_id));
            },
            CounterResult::Failure => {
                // Nobody Blocks or the Block Fails so Foreign Aid goes through
                self.result = CounterResult::Success;
                // Applying Foreign Aid Transactions
                game.player_add_coins(self.player_id, 2);
                log::info!("{}", format!("Player {}'s ForeignAid Succeeds, they gain 2 coins!", self.player_id));
            }
        }
    }
    fn can_be_blocked(&self) -> bool {
        true
    }
    fn can_be_challenged(&self) -> bool {
        false
    }
    fn get_result(&self) -> CounterResult {
        self.result
    }
}

