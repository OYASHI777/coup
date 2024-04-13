use crate::models::{CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct Pass {
    player_id: usize,
}
impl Pass {
    pub fn new(player_id: usize) -> Self{
        Pass{
            player_id,
        }
    }
}

impl DefaultBehaviour for Pass {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} Passes and abstains from any counterplay", self.player_id));
    }
    fn can_be_blocked(&self) -> bool {
        false
    }
    fn can_be_challenged(&self) -> bool {
        false
    }
    fn get_result(&self) -> CounterResult {
        // Pass result is a failure because Pass is always the negative of another action, if they do not block or challenge, the result is a Failure
        CounterResult::Failure
    }
}