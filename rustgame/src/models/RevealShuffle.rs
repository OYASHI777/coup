use crate::models::{Card, CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct RevealShuffle{
    player_id: usize,
    card: Card,
    result: CounterResult
}

impl RevealShuffle {
    pub fn new(player_id: usize, card: Card) -> Self{
        RevealShuffle{
            player_id,
            card,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for RevealShuffle {
    fn execute(&mut self, game: &mut Game){
        log::info!("{}", format!("Challenge Failed! Player {} Reveals the {:?}", self.player_id, self.card));
        // Change to send game Actions This is just temporary and is not correct

        log::info!("{}", format!("Player {}'s {:?} gets reshuffled into the deck and they receive a new card", self.player_id, self.card));
        game.shuffle_redraw(self.player_id, &self.card);
        self.result = CounterResult::Success;

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