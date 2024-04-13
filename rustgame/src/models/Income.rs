use crate::models::{CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct Income {
    player_id: usize,
}
impl Income {
    pub fn new(player_id: usize) -> Self{
        Income{
            player_id,
        }
    }
}

impl DefaultBehaviour for Income {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} uses Income", self.player_id));
        game.player_add_coins(self.player_id, 1);
        log::info!("{}", format!("Player {} gains a coin", self.player_id));
    }
    fn can_be_blocked(&self) -> bool {
        false
    }
    fn can_be_challenged(&self) -> bool {
        false
    }
    fn get_result(&self) -> CounterResult {
        CounterResult::Success
    }
}