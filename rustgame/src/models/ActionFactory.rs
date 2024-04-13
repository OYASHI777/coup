use crate::models::{Action, Card, DefaultBehaviour};
// use crate::dmodels::{Income, Assassinate, ForeignAid, BlockForeignAid, Block, Challenge, Discard, Coup, StandardActions, StandardActionsCoup, Steal, Tax, TurnStart};

use crate::models::Assassinate::Assassinate;
use crate::models::Block::Block;
// use crate::dmodels::BlockForeignAid::BlockForeignAid;
use crate::models::Challenge::Challenge;
use crate::models::Coup::Coup;
use crate::models::Discard::Discard;
use crate::models::ForeignAid::ForeignAid;
use crate::models::Pass::Pass;
use crate::models::Income::Income;
use crate::models::StandardActions::StandardActions;
use crate::models::StandardActionsCoup::StandardActionsCoup;
use crate::models::Steal::Steal;
use crate::models::Tax::Tax;
use crate::models::TurnStart::TurnStart;
use crate::models::Receive::Receive;
use crate::models::Exchange::Exchange;
use crate::models::ActionName;
use crate::models::ActionPacket;
use crate::models::RevealShuffle::RevealShuffle;
pub struct ActionFactory;

impl ActionFactory {

    pub fn create_action(action_type: &Action, player_id: usize, opposing_player_id: usize) -> Box<dyn DefaultBehaviour> {
        match action_type {
            Action::Pass => Box::new(Pass::new(player_id)),
            Action::Income => Box::new(Income::new(player_id)),
            Action::ForeignAid => Box::new(ForeignAid::new(player_id)),
            Action::TurnStart => Box::new(TurnStart::new(player_id)),
            Action::StandardActions => Box::new(StandardActions::new(player_id)),
            Action::StandardActionsCoup => Box::new(StandardActionsCoup::new(player_id)),
            Action::Assassinate => Box::new(Assassinate::new(player_id, opposing_player_id)),
            Action::Coup => Box::new(Coup::new(player_id, opposing_player_id)),
            Action::Tax => Box::new(Tax::new(player_id)),
            Action::Steal => Box::new(Steal::new(player_id, opposing_player_id)),
            Action::DiscardAmbassador => Box::new(Discard::new(player_id, Card::Ambassador)),
            Action::DiscardAssassin => Box::new(Discard::new(player_id, Card::Assassin)),
            Action::DiscardCaptain => Box::new(Discard::new(player_id, Card::Captain)),
            Action::DiscardContessa => Box::new(Discard::new(player_id, Card::Contessa)),
            Action::DiscardDuke => Box::new(Discard::new(player_id, Card::Duke)),
            Action::ChallengeAssassin => Box::new(Challenge::new(player_id, opposing_player_id, Card::Assassin)),
            Action::ChallengeAmbassador => Box::new(Challenge::new(player_id, opposing_player_id, Card::Ambassador)),
            Action::ChallengeCaptain => Box::new(Challenge::new(player_id, opposing_player_id, Card::Captain)),
            Action::ChallengeContessa => Box::new(Challenge::new(player_id, opposing_player_id, Card::Contessa)),
            Action::ChallengeDuke => Box::new(Challenge::new(player_id, opposing_player_id, Card::Duke)),
            Action::BlockAssassinate => Box::new(Block::new(player_id, opposing_player_id, Card::Contessa)),
            Action::BlockStealAmbassador => Box::new(Block::new(player_id, opposing_player_id, Card::Ambassador)),
            Action::BlockStealCaptain => Box::new(Block::new(player_id, opposing_player_id, Card::Captain)),
            Action::BlockForeignAid => Box::new(Block::new(player_id, opposing_player_id, Card::Duke)),
            Action::RevealShuffleAssassin => Box::new(RevealShuffle::new(player_id, Card::Assassin)),
            Action::RevealShuffleAmbassador => Box::new(RevealShuffle::new(player_id, Card::Ambassador)),
            Action::RevealShuffleCaptain => Box::new(RevealShuffle::new(player_id, Card::Captain)),
            Action::RevealShuffleContessa => Box::new(RevealShuffle::new(player_id, Card::Contessa)),
            Action::RevealShuffleDuke => Box::new(RevealShuffle::new(player_id, Card::Duke)),
            // ACTIONS ABOVE ARE IMPLEMENTED
            // Action::Exchange => Box::new(Exchange::new(player_id)),
            // Action::ReceiveAssassinAssassin => Box::new(ReceiveAssassinAssassin::new(player_id)),
            // Action::ReceiveAssassinAmbassador => Box::new(ReceiveAssassinAmbassador::new(player_id)),
            // Action::ReceiveAssassinCaptain => Box::new(ReceiveAssassinCaptain::new(player_id)),
            // Action::ReceiveAssassinContessa => Box::new(ReceiveAssassinContessa::new(player_id)),
            // Action::ReceiveAssassinDuke => Box::new(ReceiveAssassinDuke::new(player_id)),
            // Action::ReceiveAmbassadorAmbassador => Box::new(ReceiveAmbassadorAmbassador::new(player_id)),
            // Action::ReceiveAmbassadorCaptain => Box::new(ReceiveAmbassadorCaptain::new(player_id)),
            // Action::ReceiveAmbassadorContessa => Box::new(ReceiveAmbassadorContessa::new(player_id)),
            // Action::ReceiveAmbassadorDuke => Box::new(ReceiveAmbassadorDuke::new(player_id)),
            // Action::ReceiveCaptainCaptain => Box::new(ReceiveCaptainCaptain::new(player_id)),
            // Action::ReceiveCaptainContessa => Box::new(ReceiveCaptainContessa::new(player_id)),
            // Action::ReceiveCaptainDuke => Box::new(ReceiveCaptainDuke::new(player_id)),
            // Action::ReceiveContessaContessa => Box::new(ReceiveContessaContessa::new(player_id)),
            // Action::ReceiveContessaDuke => Box::new(ReceiveContessaDuke::new(player_id)),
            // Action::ReceiveDukeDuke => Box::new(ReceiveDukeDuke::new(player_id)),
            Action::Pass => panic!("Pass type should not be created"),
            _ => panic!("Unknown action type"),
        }
    }

    pub fn create_action_packet(action: &Action, player_id: usize, opposing_player_id: usize, card_data: &Vec<Card>, turn_no: usize) -> ActionPacket {
        match action {
            // No Card data and No opposing player
            Action::Pass => ActionPacket::new(turn_no, ActionName::Pass, player_id, 17, &vec![]),
            Action::Income => ActionPacket::new(turn_no, ActionName::Income, player_id, 17, &vec![]),
            Action::ForeignAid => ActionPacket::new(turn_no, ActionName::ForeignAid, player_id, 17, &vec![]),
            // Has All card data is either the card being challenged or the card being used to block
            Action::ChallengeAmbassador => ActionPacket::new(turn_no, ActionName::Challenge, player_id, opposing_player_id, &vec![Card::Ambassador]),
            Action::ChallengeAssassin => ActionPacket::new(turn_no, ActionName::Challenge, player_id, opposing_player_id, &vec![Card::Assassin]),
            Action::ChallengeCaptain => ActionPacket::new(turn_no, ActionName::Challenge, player_id, opposing_player_id, &vec![Card::Captain]),
            Action::ChallengeContessa => ActionPacket::new(turn_no, ActionName::Challenge, player_id, opposing_player_id, &vec![Card::Contessa]),
            Action::ChallengeDuke => ActionPacket::new(turn_no, ActionName::Challenge, player_id, opposing_player_id, &vec![Card::Duke]),
            
            // Block card_data[0] with card_data[1]
            Action::BlockAssassinate => ActionPacket::new(turn_no, ActionName::Block, player_id, opposing_player_id, &vec![Card::Contessa]),
            Action::BlockStealAmbassador => ActionPacket::new(turn_no, ActionName::Block, player_id, opposing_player_id, &vec![Card::Ambassador]),
            Action::BlockStealCaptain => ActionPacket::new(turn_no, ActionName::Block, player_id, opposing_player_id, &vec![Card::Captain]),
            Action::BlockForeignAid => ActionPacket::new(turn_no, ActionName::Block, player_id, opposing_player_id, &vec![Card::Duke]),
            Action::Steal => ActionPacket::new(turn_no, ActionName::Steal, player_id, opposing_player_id, &vec![Card::Captain]),
            Action::Assassinate => ActionPacket::new(turn_no, ActionName::Assassinate, player_id, opposing_player_id, &vec![Card::Assassin]),
            Action::Coup => ActionPacket::new(turn_no, ActionName::Coup, player_id, opposing_player_id, &vec![]),
            // No Opposing Player
            Action::Tax => ActionPacket::new(turn_no, ActionName::Tax, player_id, 17, &vec![Card::Duke]),
            Action::Exchange => ActionPacket::new(turn_no, ActionName::Exchange, player_id, 17, &vec![Card::Ambassador]),
            // Action::DiscardAssassin => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, vec![Card::Assassin]),
            // Action::DiscardAmbassador => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, vec![Card::Ambassador]),
            // Action::DiscardCaptain => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, vec![Card::Captain]),
            // Action::DiscardContessa => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, vec![Card::Contessa]),
            // Action::DiscardDuke => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, vec![Card::Duke]),
            Action::Discard => ActionPacket::new(turn_no, ActionName::Discard, player_id, 17, card_data),
            Action::Receive => ActionPacket::new(turn_no, ActionName::Receive, player_id, 17, card_data),
            // Reveal and Reveal_Shuffle
            // card_data[0] should be outgoing card
            // card_data[1] should be received card
            // Action::RevealShuffleAmbassador => ActionPacket::new(turn_no, ActionName::RevealShuffleAmbassador, player_id, 17, card_data),
            // Action::RevealShuffleAssassin => ActionPacket::new(turn_no, ActionName::RevealShuffleAssassin, player_id, 17, card_data),
            // Action::RevealShuffleCaptain => ActionPacket::new(turn_no, ActionName::RevealShuffleCaptain, player_id, 17, card_data),
            // Action::RevealShuffleContessa => ActionPacket::new(turn_no, ActionName::RevealShuffleContessa, player_id, 17, card_data),
            // Action::RevealShuffleDuke => ActionPacket::new(turn_no, ActionName::RevealShuffleDuke, player_id, 17, card_data),
            _ => panic!("Not implemented yet"),
            // Receive functionality and exchange
        }
    }

    pub fn create_action_packet_vec_blind(action: &[Action], player_id: usize, opposing_player_id: usize, turn_no: usize) -> Vec<ActionPacket> {
        let mut output:Vec<ActionPacket> = Vec::new();
        for act in action.iter(){
            if *act == Action::Steal || *act == Action::Coup || *act == Action::Assassinate {
                for i in 0..6 {
                    if i != player_id {
                        output.push(ActionFactory::create_action_packet(act, player_id, i, &vec![], turn_no));
                    }
                }
            } else if *act == Action::Discard {
                let all_cards: [Card; 5] = [Card::Ambassador, Card::Assassin, Card::Captain, Card::Contessa, Card::Duke];
                for card in all_cards.iter(){
                    output.push(ActionFactory::create_action_packet(act, player_id, opposing_player_id, &vec![*card], turn_no));
                }
            }
            else{
                // All other non reveal cards
                output.push(ActionFactory::create_action_packet(act, player_id, opposing_player_id, &vec![], turn_no));
            }
        }
        return output;
    }

    pub fn create_action_from_pkt(action_packet: &ActionPacket) -> Box<dyn DefaultBehaviour> {
        let player_id: usize = action_packet.get_player_id();
        let opposing_player_id: usize = action_packet.get_opposing_player_id();
        let card_data: Vec<Card> = action_packet.get_card_data();
        match action_packet.get_action_name() {
            ActionName::Pass => Box::new(Pass::new(player_id)),
            ActionName::Income => Box::new(Income::new(player_id)),
            ActionName::ForeignAid => Box::new(ForeignAid::new(player_id)),
            ActionName::Coup => Box::new(Coup::new(player_id, opposing_player_id)),
            ActionName::Tax => Box::new(Tax::new(player_id)),
            ActionName::Assassinate => Box::new(Assassinate::new(player_id, opposing_player_id)),
            ActionName::Challenge => Box::new(Challenge::new(player_id, opposing_player_id, card_data[0])),
            ActionName::Steal => Box::new(Steal::new(player_id, opposing_player_id)),
            ActionName::Block => Box::new(Block::new(player_id, opposing_player_id, card_data[0])),
            ActionName::Discard => Box::new(Discard::new(player_id, card_data[0])),
            ActionName::RevealShuffle => panic!("Reveal Shuffle cannot be Initiated by the Player!"),
            ActionName::Exchange => Box::new(Exchange::new(player_id, card_data)),
            ActionName::Receive => Box::new(Receive::new(player_id, card_data)),
        }
    }

}
