use crate::models::{Card, CounterResult, DefaultBehaviour, Action};
use crate::game::Game;
use super::ActionFactory::ActionFactory;
pub struct Challenge {
    player_id: usize,
    opposing_player_id: usize,
    result: CounterResult,
    card: Card
}

impl Challenge {
    pub fn new(player_id: usize, opposing_player_id: usize, card: Card) -> Self{
        // player_id challenges opposing_player_id
        // Opposing_player_id claims to have a Duke
        // player_id does not think opposing_player_id has a Duke
        Challenge{
            player_id,
            opposing_player_id,
            result: CounterResult::Failure,
            card,
        }
    }
}
// === TODO === Make this a general challenge with self.card defined
impl DefaultBehaviour for Challenge {
    fn execute(&mut self, game: &mut Game) {
        log::info!("{}", format!("Player {} challenges Player {}'s {:?}", self.player_id, self.opposing_player_id, self.card));
        // Checks if opposing player actually has Duke card
        // Change to match statement
        if game.player_has_card(self.opposing_player_id, &self.card) {
            // Initiating Player loses challenge as opposing player has Duke
            self.result = CounterResult::Failure;
        } else {
            // Initiating Player wins challenge as opposing player does not have Duke
            self.result = CounterResult::Success;
        }
        
        // Resolve transactions for challenge
        match self.result {
            CounterResult::Success => {
                // Player initiating the Challenge Succeeds
                // Poll Block
                log::info!("{}", format!("Challenge Succeeded! Player {} does not have the {:?}", self.opposing_player_id, self.card));
                let discard_options: [Action; 1] = [Action::Discard];
                let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.opposing_player_id, self.player_id, &discard_options);
                box_default_behaviour.execute(game);
                              
                // Run discard choice
            },
            CounterResult::Failure => {
                // // Connect to RevealShuffle

                let revealaction: Action = match self.card {
                    Card::Ambassador => Action::RevealShuffleAmbassador,
                    Card::Assassin => Action::RevealShuffleAssassin,
                    Card::Captain => Action::RevealShuffleCaptain,
                    Card::Contessa => Action::RevealShuffleContessa,
                    Card::Duke => Action::RevealShuffleDuke,
                };
                let mut box_default_behaviour: Box<dyn DefaultBehaviour> = ActionFactory::create_action(&revealaction,self.opposing_player_id, 17);
                box_default_behaviour.execute(game);

                let discard_options: [Action; 1] = [Action::Discard];
                // Player receiving the Challenge Succeeds
                let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.present_legal_moves(self.player_id, self.opposing_player_id, &discard_options);
                box_default_behaviour.execute(game);
            
            },
        }
    }
    fn can_be_blocked(&self) -> bool {
        false
    }
    fn can_be_challenged(&self) -> bool {
        false
    }
    fn get_result(&self) -> CounterResult {
        self.result
    }
}
