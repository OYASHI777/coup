use rand::{thread_rng, seq::SliceRandom, Rng};
use crate::models::{Card, ActionPacket, ActionName};
pub struct Player {
    player_id: usize,
    coins: u32,
    cards: Vec<Card>,
    influence: u32,
}
// Player should handle msg passing
// Put Player in another file

impl Player {
    pub fn new(player_id: usize) -> Self {
        Player {
            player_id,
            coins: 2,
            cards: Vec::new(),
            influence: 2,
        }
    }

    pub fn new_hand(player_id: usize, hand: &Vec<Card>)-> Self {
        Player {
            player_id,
            coins: 2,
            cards: hand.clone(),
            influence: 2,
        }
    }

    pub fn change_hand(&mut self, hand: &Vec<Card>){
        self.cards = hand.clone();
    }

    pub fn is_alive(&self) -> bool {
        if self.influence > 0 {
            true
        } else {
            false
        }
    }

    pub fn get_coins(self) -> u32 {
        self.coins
    }
    pub fn change_coins(&mut self, new_amount: u32){
        self.coins = new_amount;
    }
    pub fn add_coins(&mut self, additional_amount: u32){
        self.coins += additional_amount;
    }
    pub fn discard_card(&mut self, discard_card: Card){
        if let Some(index) = self.cards.iter().position(|&x| x == discard_card){
            self.cards.remove(index);
            self.influence -= 1;
        }
    }
    pub fn has_card(&mut self, check_card: &Card) -> bool{
        self.cards.contains(check_card)
    }
}
pub trait PlayerInterface {
    fn choose_actions(&self, actions: Vec<ActionPacket>) -> ActionPacket;
}
impl PlayerInterface for Player {
    fn choose_actions(&self, actions: Vec<ActionPacket>) -> ActionPacket {
        // Choose Pass if possible
        let mut rng = thread_rng();
        let unif_rand: f32 = rng.gen_range(0.0..1.0);
        // Increasing chance of passing to 0.5*0.5 + 0.5 = 0.75
        if unif_rand > 0.5 {
            let pass_packet = actions.iter().find(|pkt| pkt.get_action_name() == ActionName::Pass);
            match pass_packet {
                Some(pkt) => {return pkt.clone();},
                None => {},
            }
        }

        if let Some(output) = actions.choose(&mut thread_rng()).cloned(){
            return output;
        } else {
            log::trace!("{}", format!("Player choices {:?}", actions));
            panic!("what nonsense player move is this");
        }
    }
}