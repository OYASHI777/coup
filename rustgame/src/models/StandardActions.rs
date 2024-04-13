use crate::models::{Action, CounterResult, DefaultBehaviour};
use crate::game::Game;

pub struct StandardActions {
    player_id: usize,
}

impl StandardActions {
    pub fn new(player_id: usize) -> Self{
        StandardActions{
            player_id,
        }
    }
}

impl DefaultBehaviour for StandardActions {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Presenting Standard Actions to Player {}", self.player_id));
        let standard_options: [Action; 6] = [Action::Income, Action::ForeignAid, Action::Tax, Action::Assassinate, Action::Exchange, Action::Steal];

        let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.player_id, 17, &standard_options);
        box_default_behaviour.execute(game);

        log::trace!("End StandardActions");
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