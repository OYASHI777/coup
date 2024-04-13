use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;
use log;
const ASSASSINATE_COST: u8 = 3;
pub struct Assassinate {
    player_id: usize,
    opposing_player_id: usize,
    result: CounterResult,
}
impl Assassinate {
    pub fn new(player_id: usize, opposing_player_id: usize) -> Self{
        Assassinate{
            player_id,
            opposing_player_id,
            result: CounterResult::Failure,
        }
    }
}


// POll opposing Challenge
// Assassin -> Challenge -> Reveal Assassin -> Block -> Challenge -> Success
// Assassin -> No Challenge -> No Block
// Assassin -> No Challenge -> Block

impl DefaultBehaviour for Assassinate {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}",format!("Player {} Assassinates Player {}", self.player_id, self.opposing_player_id));
        // Move Costs 3 coins just to use
        game.player_minus_coins(self.player_id, ASSASSINATE_COST);
        
        let challenge_options: [Action; 2] = [Action::Pass, Action::ChallengeAssassin];
        let challenge_result_a: CounterResult;
        // Checks if any Player wishes to challenge Assassin

        challenge_result_a = game.round_robin_poll_counterplay(self.player_id, &challenge_options);
        // if A player and Def Player are both alive and player_id indeed has Assassin
        // Ask Def player if they want to block (challenging a block is within block logic)
        if game.is_alive(self.player_id) && game.is_alive(self.opposing_player_id) && challenge_result_a == CounterResult::Failure {
            log::info!("{}", format!("Player {} has the option to block!", self.opposing_player_id));
            let block_options: [Action; 2] = [Action::Pass, Action::BlockAssassinate];
            let mut block_result: CounterResult;
            // Poll Block
            let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.opposing_player_id, self.player_id, &block_options);
            box_default_behaviour.execute(game);
            block_result = box_default_behaviour.get_result();
                        
            match block_result {
                CounterResult::Success => {
                    // Block Succeeds, Assassinate Fails
                    self.result = CounterResult::Failure;
                },
                CounterResult::Failure => {
                    // Block Fails, Assassinate Succeeds
                    if game.is_alive(self.opposing_player_id){
                        let discard_options: [Action; 1] = [Action::Discard];
                        let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.opposing_player_id, self.player_id,  &discard_options);
                        box_default_behaviour.execute(game);
                        // game.present_discard_moves(self.opposing_player_id).execute(game);
                    }   
                    self.result = CounterResult::Success;
                }   
            }
        }
                    
        if game.is_alive(self.player_id) && challenge_result_a == CounterResult::Success {
            self.result = CounterResult::Failure;
        } else if game.is_alive(self.player_id) {
            self.result = CounterResult::Success;
        } else {
            self.result = CounterResult::Failure;
        }
        log::trace!("{}", format!("End Player {}'s Assassinate", self.player_id));
    }

    fn can_be_blocked(&self) -> bool {
        true
    }
    fn can_be_challenged(&self) -> bool {
        true
    }
    fn get_result(&self) -> CounterResult {
        self.result
    }
}