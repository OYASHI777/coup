use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;
use crate::models::ActionFactory::ActionFactory;
use crate::models::Coup::COUP_COST;


const FORCE_COUP_ABOVE: u8 = 9;
// Consider that you don't even need player_id here
pub struct TurnStart {
    player_id: usize,
}
impl TurnStart {
    pub fn new(player_id: usize) -> Self{
        TurnStart{
            player_id,
        }
    }
}

impl DefaultBehaviour for TurnStart {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("It is now Player {}'s turn!", self.player_id));
        log::info!("Cards at Start of the turn: ");
        game.logcc();
        if game.player_get_coins(self.player_id) > FORCE_COUP_ABOVE {
            let coup_options: [Action; 1] = [Action::Coup];
            let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.player_id, 17 as usize, &coup_options);
            box_default_behaviour.execute(game);

            } else if game.player_get_coins(self.player_id) >= COUP_COST {
                let mut box_standard_actions: Box<dyn DefaultBehaviour> = ActionFactory::create_action(&Action::StandardActionsCoup, self.player_id, 0);
                box_standard_actions.execute(game);
            } else {
                let mut box_standard_actions: Box<dyn DefaultBehaviour> = ActionFactory::create_action(&Action::StandardActions, self.player_id, 0);
                box_standard_actions.execute(game);
            }
        log::info!("{}", format!("It is the End of Player {}'s turn!", self.player_id));

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