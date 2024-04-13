use crate::models::{Card, CounterResult, DefaultBehaviour, Action, ActionPacket};
use crate::game::Game;
use crate::models::ActionFactory::ActionFactory;
use std::collections::HashMap;

pub struct Exchange {
    player_id: usize,
    card_data: Vec<Card>,
    result: CounterResult
}

impl Exchange {
    pub fn new(player_id: usize, card_data: Vec<Card>) -> Self{
        Exchange{
            player_id,
            card_data,
            result: CounterResult::Failure,
        }
    }
}

impl DefaultBehaviour for Exchange {
    fn execute(&mut self, game: &mut Game){
        log::info!("{}", format!("Player {} uses Exchange to Peek in the Center Pile", self.player_id));
        log::info!("{}", format!("GameState before Exchange"));
        game.logcc();
        let challenge_options: [Action; 2] = [Action::Pass, Action::ChallengeAmbassador];
        let challenge_result_a: CounterResult; 

        challenge_result_a = game.round_robin_poll_counterplay(self.player_id, &challenge_options);

        // Exchange ability usable if challenge fails!
        if challenge_result_a == CounterResult::Failure {

            // Get Randomly choose 2 cards here, then send the Receive combinations to game
            // ActionPacket should be called Reveal

            //Cant use array because moves not known at compile time. Instead just create the packets and send them directly
            
            // get random 2 cards from deck
            let two_random_cards: Vec<Card> = game.choose_random_cards(6 as usize, 2 as usize);
            log::info!("{}", format!("Player {} privately draws the cards: {:?}", self.player_id, two_random_cards));

            let mut combined: Vec<Card> = game.player_get_hand(self.player_id);
            combined.extend(two_random_cards);

            let mut card_map: HashMap<Card, u8> = HashMap::new();
            for card in &[Card::Ambassador, Card::Assassin, Card::Captain, Card::Contessa, Card::Duke]{
                card_map.entry(*card).or_insert(0);
            }
            for card in &combined {
                *card_map.entry(*card).or_insert(0) += 1;
            }

            // This will be legal and another legal check will not be done
            // Generate action packets
            let mut legal_pkts: Vec<ActionPacket> = Vec::new();
            if game.player_get_influence(self.player_id) == 2 {

                for card in &[Card::Ambassador, Card::Assassin, Card::Captain, Card::Contessa, Card::Duke]{
                    if card_map[&card] == 2 {
                        legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![*card, *card], game.get_turn_no()));
                    }
                }
                if card_map[&Card::Ambassador] > 0 && card_map[&Card::Assassin] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Ambassador, Card::Assassin], game.get_turn_no()));
                }
                if card_map[&Card::Ambassador] > 0 && card_map[&Card::Captain] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Ambassador, Card::Captain], game.get_turn_no()));
                }
                if card_map[&Card::Ambassador] > 0 && card_map[&Card::Contessa] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Ambassador, Card::Contessa], game.get_turn_no()));
                }
                if card_map[&Card::Ambassador] > 0 && card_map[&Card::Duke] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Ambassador, Card::Duke], game.get_turn_no()));
                }
                if card_map[&Card::Assassin] > 0 && card_map[&Card::Captain] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Assassin, Card::Captain], game.get_turn_no()));
                }
                if card_map[&Card::Assassin] > 0 && card_map[&Card::Contessa] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Assassin, Card::Contessa], game.get_turn_no()));
                }
                if card_map[&Card::Assassin] > 0 && card_map[&Card::Duke] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Assassin, Card::Duke], game.get_turn_no()));
                }
                if card_map[&Card::Captain] > 0 && card_map[&Card::Contessa] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Captain, Card::Contessa], game.get_turn_no()));
                }
                if card_map[&Card::Captain] > 0 && card_map[&Card::Duke] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Captain, Card::Duke], game.get_turn_no()));
                }
                if card_map[&Card::Contessa] > 0 && card_map[&Card::Duke] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Contessa, Card::Duke], game.get_turn_no()));
                }
            } else {
                if card_map[&Card::Ambassador] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Ambassador], game.get_turn_no()));
                }
                if card_map[&Card::Assassin] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Assassin], game.get_turn_no()));
                }
                if card_map[&Card::Captain] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Captain], game.get_turn_no()));
                }
                if card_map[&Card::Contessa] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Contessa], game.get_turn_no()));
                }
                if card_map[&Card::Duke] > 0{
                    legal_pkts.push(ActionFactory::create_action_packet(&Action::Receive, self.player_id, 17, &vec![Card::Duke], game.get_turn_no()));
                }
            }
                
            // send to sendlegalmoves (it will check if legal)
            let mut box_default_behaviour: Box<dyn DefaultBehaviour> = game.send_move_pkts(self.player_id, legal_pkts);
            box_default_behaviour.execute(game);
        }
        log::trace!("End of Exchange GameState:");
        game.logcc();
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