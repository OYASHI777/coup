use crate::models::{CounterResult, DefaultBehaviour, Card};
use crate::game::Game;

pub struct Receive {
    player_id: usize,
    card_data: Vec<Card>,
}
impl Receive {
    pub fn new(player_id: usize, card_data: Vec<Card>) -> Self{
        Receive{
            player_id,
            card_data,
        }
    }
}

impl DefaultBehaviour for Receive {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} privately receives the cards {:?} from the Pile", self.player_id, self.card_data));
        // gets game to take conduct the swap
        game.swap_pooled(self.player_id, &self.card_data);
        if self.card_data.sort() != game.player_get_hand(self.player_id).sort(){
            panic!("swap_pooled went wrong!");
        }
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