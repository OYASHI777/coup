use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;

const STEAL_AMT: u8 = 2;
pub struct Steal {
    player_id: usize,
    opposing_player_id: usize,
    result: CounterResult,
}
impl Steal {
    pub fn new(player_id: usize, opposing_player_id: usize) -> Self{
        Steal {
            player_id,
            opposing_player_id,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for Steal {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} uses Steal on Player {}", self.player_id, self.opposing_player_id));
        let challenge_options: [Action; 2] = [Action::Pass, Action::ChallengeCaptain];
        let challenge_result_a: CounterResult; 
        let opposing_coins: u8 = game.player_get_coins(self.opposing_player_id);

        challenge_result_a = game.round_robin_poll_counterplay(self.player_id, &challenge_options);
        // IF Stealing Player and Def player are both alive and player_id indeed has Captain
        // if game.is_alive(self.player_id) && game.is_alive(self.opposing_player_id) && challenge_result_a == CounterResult::Failure {
        if game.is_alive(self.opposing_player_id) && challenge_result_a == CounterResult::Failure {
            log::info!("{}", format!("Player {} has the option to block!", self.opposing_player_id));
            let block_options: [Action; 3] = [Action::Pass, Action::BlockStealAmbassador, Action::BlockStealCaptain];
            // Challenge Fails
            let mut block_result: CounterResult;
            // Poll Block
            let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.opposing_player_id, self.player_id, &block_options);
            box_default_behaviour.execute(game);
            block_result = box_default_behaviour.get_result();

            match block_result {
                CounterResult::Success => {
                    // Block Succeeds, Steal Fails
                    self.result = CounterResult::Failure;
                },
                CounterResult::Failure => {
                    // Block Fails, Steal Succeeds
                    if game.is_alive(self.opposing_player_id){
                        if opposing_coins >= 2 {
                            log::info!("{}", format!("Player {} takes {} of Player {}'s coins", self.player_id, STEAL_AMT,self.opposing_player_id));
                            game.player_add_coins(self.player_id, STEAL_AMT);
                            game.player_minus_coins(self.opposing_player_id, STEAL_AMT);
                        } else if opposing_coins == 1{
                            log::info!("{}", format!("Player {} takes 1 of Player {}'s coins", self.player_id,self.opposing_player_id));
                            game.player_add_coins(self.player_id, 1 as u8);
                            game.player_minus_coins(self.opposing_player_id, 1 as u8);
                        } else {
                            log::info!("{}", format!("Player {} takes none of Player {}'s coins as Player {} had no coins to begin with", self.player_id, self.opposing_player_id, self.opposing_player_id));
                        }
                    } else {
                        if opposing_coins >= 2{
                            log::info!("{}", format!("Player {} takes {} of Player {}'s coins", self.player_id, STEAL_AMT,self.opposing_player_id));
                            game.player_add_coins(self.player_id, 2 as u8);
                        } else if opposing_coins == 1 {
                            log::info!("{}", format!("Player {} takes 1 of Player {}'s coins", self.player_id,self.opposing_player_id));
                            game.player_add_coins(self.player_id, 1 as u8);
                        } else {
                            log::info!("{}", format!("Player {} takes none of Player {}'s coins as Player {} had no coins to begin with", self.player_id, self.opposing_player_id, self.opposing_player_id));
                        }
                    }
                    self.result = CounterResult::Success;
                }
            }
        } else if challenge_result_a == CounterResult::Failure {
        // } else if game.is_alive(self.player_id) && challenge_result_a == CounterResult::Failure {
            if opposing_coins >= 2{
                log::info!("{}", format!("Player {} takes {} of Player {}'s coins", self.player_id, STEAL_AMT,self.opposing_player_id));
                game.player_add_coins(self.player_id, 2 as u8);
            } else if opposing_coins == 1 {
                log::info!("{}", format!("Player {} takes 1 of Player {}'s coins", self.player_id,self.opposing_player_id));
                game.player_add_coins(self.player_id, 1 as u8);
            } else {
                log::info!("{}", format!("Player {} takes none of Player {}'s coins as Player {} had no coins to begin with", self.player_id, self.opposing_player_id, self.opposing_player_id));
            }
        }
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