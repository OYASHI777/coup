use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct StandardActionsCoup {
    player_id: usize,
}

impl StandardActionsCoup {
    pub fn new(player_id: usize) -> Self{
        StandardActionsCoup{
            player_id,
        }
    }
}

impl DefaultBehaviour for StandardActionsCoup {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Presenting Standard Actions + Coup to Player {}", self.player_id));
        let standard_options: [Action; 7] = [Action::Income, Action::ForeignAid, Action::Tax, Action::Assassinate, Action::Exchange, Action::Steal, Action::Coup];
        
        let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.player_id, 17 as usize, &standard_options);
        box_default_behaviour.execute(game);

        log::trace!("End StandardActionsCoup");
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