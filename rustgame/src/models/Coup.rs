use crate::models::{CounterResult, DefaultBehaviour, Action};
use crate::game::Game;


pub const COUP_COST: u8 = 7;
pub struct Coup {
    player_id: usize,
    opposing_player_id: usize,
}
impl Coup {
    pub fn new(player_id: usize, opposing_player_id: usize) -> Self{
        Coup{
            player_id,
            opposing_player_id,
        }
    }
}

impl DefaultBehaviour for Coup {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} uses Coup on Player {}", self.player_id, self.opposing_player_id));
        // Choice to coup assumes player has 7 coins
        game.player_minus_coins(self.player_id, COUP_COST);
        
        let discard_options: [Action; 1] = [Action::Discard];
        
        let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.opposing_player_id, self.player_id, &discard_options);
        box_default_behaviour.execute(game);
        
        log::trace!("End Coup");
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