use crate::models::{Card, CounterResult, DefaultBehaviour};
use crate::game::Game;
use log;
pub struct Discard{
    player_id: usize,
    card: Card,
    result: CounterResult
}


impl Discard {
    pub fn new(player_id: usize, card: Card) -> Self{
        Discard{
            player_id,
            card,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for Discard {
    fn execute(&mut self, game: &mut Game){

        log::info!("{}",format!("Player {} Discards {:?}", self.player_id, self.card));
        if game.player_remove_card(self.player_id, &self.card) == CounterResult::Success {
            self.result = CounterResult::Success;
        } else {
            self.result = CounterResult::Failure;
            panic!("Discard did not work properly!")
        }
        log::trace!("{}",format!("End Player {} Discard", self.player_id));
    }
    fn can_be_blocked(&self) -> bool {
        false
    }
    fn can_be_challenged(&self) -> bool {
        false
    }
    fn get_result(&self) -> CounterResult{
        self.result
    }
}