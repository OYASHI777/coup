// TODO: Keeps track of path name when added or dropped
// TODO: Initialises path name at whatever the root node is at
use crate::history_public::{ActionObservation, Card};

pub struct DefaultEmbedding;

pub trait ActionEmbedding {
    fn serialise_name_key(&self, action: &ActionObservation) -> String;
    fn add_action(&self, path: &str, action: &ActionObservation) -> String;
    fn remove_action(&self, path: &str) -> String;
}

impl ActionEmbedding for DefaultEmbedding {
    fn serialise_name_key(&self, action: &ActionObservation) -> String {
        // Public moves only!
        match action {
            ActionObservation::EmptyAO => panic!("root"),
            ActionObservation::Income { player_id } => format!("IC{}", player_id),
            ActionObservation::ForeignAid { player_id } => format!("FA{}", player_id),
            ActionObservation::Tax { player_id } => format!("TX{}", player_id),
            ActionObservation::Steal { player_id, opposing_player_id, ..  } => format!("SL{}|{}", player_id, opposing_player_id),
            ActionObservation::Assassinate { player_id, opposing_player_id, ..  } => format!("AS{}|{}", player_id, opposing_player_id),
            ActionObservation::Coup { player_id, opposing_player_id, ..  } => format!("CO{}|{}", player_id, opposing_player_id),
            ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => 
                format!("CH{}|{}", opposing_player_id, final_actioner),
            ActionObservation::CollectiveBlock { participants, opposing_player_id, final_actioner } => 
                format!("BC{}|{}", opposing_player_id, final_actioner),
            // Not recombining version
            // ActionObservation::CollectiveChallenge { participants, opposing_player_id, final_actioner } => 
            //     format!("CH{}{}{}", opposing_player_id, participants.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>(), final_actioner),
            // ActionObservation::CollectiveBlock { participants, opposing_player_id, final_actioner } => 
            //     format!("BC{}{}{}", opposing_player_id, participants.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>(), final_actioner),
            ActionObservation::BlockSteal { player_id, opposing_player_id, card } => format!("BS{}|{}|{}",if *card == Card::Captain {"C"} else {"A"}, player_id, opposing_player_id ),
            ActionObservation::BlockAssassinate { player_id, opposing_player_id, ..  } => format!("BA{}|{}", player_id, opposing_player_id),
            ActionObservation::Discard { player_id, card, no_cards } => {
                match no_cards {
                    1 => format!("DO{}{}", player_id, card[0].card_to_string()),
                    2 => format!("DA{}{}{}", player_id, card[0].card_to_string(), card[1].card_to_string()),
                    _ => panic!("Invalid number of cards"),
                }
            },
            ActionObservation::RevealRedraw { player_id, card } => format!("RR{}{}", player_id, card.card_to_string()),
            ActionObservation::Exchange { player_id } => format!("EX{}", player_id),
            ActionObservation::ExchangeDraw { player_id , card } => format!("ED{}{}{}", player_id, card[0].card_to_string(), card[1].card_to_string()),
            ActionObservation::ExchangeChoice { player_id , no_cards, .. } => format!("EC{}{}", no_cards, player_id),
            _ => panic!("bad kind"),
        }
    }
    
    fn add_action(&self, path: &str, action: &ActionObservation) -> String {
        let node_str: String = self.serialise_name_key(action);
        let new_str: String  = format!("{path}_{node_str}");
        new_str
    }
    fn remove_action(&self, path: &str) -> String {
        if let Some(index) = path.rfind("_") {
            path[..index].to_string()
        } else {
            path.to_string()
        }
    }
}

