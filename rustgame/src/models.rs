use crate::game::Game;
pub mod Assassinate;
pub mod Block;
pub mod Challenge;
pub mod Coup;
pub mod Discard;
pub mod ForeignAid;
pub mod Income;
pub mod StandardActions;
pub mod StandardActionsCoup;
pub mod Steal;
pub mod Tax;
pub mod TurnStart;
pub mod ActionFactory;
pub mod Pass;
pub mod RevealShuffle;
pub mod Exchange;
pub mod Receive;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, PartialOrd, Ord)]
pub enum Card {
    Ambassador,
    Assassin,
    Captain,
    Duke,
    Contessa,
}

pub enum Phase {
    Action,
    Attacking,
    Blocking,
    Discard,
    DoubleDiscard,
    Exchange,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CounterResult{
    Success,
    Failure,
}
// Action Enums do not contain numbering information, only the structs do
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Income,
    ForeignAid,
    Coup,
    Tax,
    Assassinate,
    Exchange,
    Steal,
    BlockForeignAid,
    BlockStealAmbassador,
    BlockStealCaptain,
    BlockAssassinate,
    ChallengeDuke,
    ChallengeAssassin,
    ChallengeAmbassador,
    ChallengeCaptain,
    ChallengeContessa,
    Pass,
    Receive,
    DiscardAmbassador,
    DiscardAssassin,
    DiscardCaptain,
    DiscardContessa,
    DiscardDuke,
    Discard,
    StandardActions,
    StandardActionsCoup,
    TurnStart,
    // Lose First card, gained Second card
    RevealShuffleAmbassador,
    RevealShuffleAssassin,
    RevealShuffleCaptain,
    RevealShuffleContessa,
    RevealShuffleDuke,
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ActionName {
    Income,
    ForeignAid,
    Coup,
    Tax,
    Assassinate,
    Exchange,
    Steal,
    Block,
    Challenge,
    Pass,
    Receive,
    Discard,
    RevealShuffle,
}
// for NN
pub enum PolicyAction {
    Income,
    ForeignAid,
    CoupP0,
    CoupP1,
    CoupP2,
    CoupP3,
    CoupP4,
    CoupP5,
    Tax,
    AssassinateP0,
    AssassinateP1,
    AssassinateP2,
    AssassinateP3,
    AssassinateP4,
    AssassinateP5,
    Exchange,
    StealP0,
    StealP1,
    StealP2,
    StealP3,
    StealP4,
    StealP5,
    BlockForeignAidP0,
    BlockForeignAidP1,
    BlockForeignAidP2,
    BlockForeignAidP3,
    BlockForeignAidP4,
    BlockForeignAidP5,
    BlockStealAmbassadorP0,
    BlockStealAmbassadorP1,
    BlockStealAmbassadorP2,
    BlockStealAmbassadorP3,
    BlockStealAmbassadorP4,
    BlockStealAmbassadorP5,
    BlockStealCaptainP0,
    BlockStealCaptainP1,
    BlockStealCaptainP2,
    BlockStealCaptainP3,
    BlockStealCaptainP4,
    BlockStealCaptainP5,
    BlockAssassinateP0,
    BlockAssassinateP1,
    BlockAssassinateP2,
    BlockAssassinateP3,
    BlockAssassinateP4,
    BlockAssassinateP5,
    ChallengeDukeP0,
    ChallengeDukeP1,
    ChallengeDukeP2,
    ChallengeDukeP3,
    ChallengeDukeP4,
    ChallengeDukeP5,
    ChallengeAssassinP0,
    ChallengeAssassinP1,
    ChallengeAssassinP2,
    ChallengeAssassinP3,
    ChallengeAssassinP4,
    ChallengeAssassinP5,
    ChallengeAmbassadorP0,
    ChallengeAmbassadorP1,
    ChallengeAmbassadorP2,
    ChallengeAmbassadorP3,
    ChallengeAmbassadorP4,
    ChallengeAmbassadorP5,
    ChallengeCaptainP0,
    ChallengeCaptainP1,
    ChallengeCaptainP2,
    ChallengeCaptainP3,
    ChallengeCaptainP4,
    ChallengeCaptainP5,
    ChallengeContessaP0,
    ChallengeContessaP1,
    ChallengeContessaP2,
    ChallengeContessaP3,
    ChallengeContessaP4,
    ChallengeContessaP5,
    Pass,
    // TO change receive to Amb first!!
    ReceiveAssassinAssassin,
    ReceiveAssassinAmbassador,
    ReceiveAssassinCaptain,
    ReceiveAssassinContessa,
    ReceiveAssassinDuke,
    ReceiveAmbassadorAmbassador,
    ReceiveAmbassadorCaptain,
    ReceiveAmbassadorContessa,
    ReceiveAmbassadorDuke,
    ReceiveCaptainCaptain,
    ReceiveCaptainContessa,
    ReceiveCaptainDuke,
    ReceiveContessaContessa,
    ReceiveContessaDuke,
    ReceiveDukeDuke,
    DiscardAmbassador,
    DiscardAssassin,
    DiscardCaptain,
    DiscardContessa,
    DiscardDuke,
}

// I need to handle, execution of action, blocking of action, challengeing of action or blocks, discarding of cards
// Actions should change the game state and let the game engine know what the next move to handle is.
// Actions can be challenged or blocked
// Blocks can be challenged
// Challenges cannot be stopped
pub trait DefaultBehaviour{
    // We can handle everything in execute
    fn execute(&mut self, game: &mut Game) {}
    fn can_be_blocked(&self) -> bool;
    fn can_be_challenged(&self) -> bool;
    fn get_result(&self) -> CounterResult;
}
#[derive(Debug)]
pub struct ActionPacket {
    turn_no: usize,
    action_name: ActionName,
    player_id: usize,
    opposing_player_id: usize,
    card_data: Vec<Card>,
}

impl Clone for ActionPacket {
    fn clone(&self) -> Self{
        ActionPacket{
            turn_no: self.turn_no,
            action_name: self.action_name.clone(),
            player_id: self.player_id,
            opposing_player_id: self.opposing_player_id,
            card_data: self.card_data.clone(),
        }
    }
}

impl ActionPacket {
    pub fn new(turn_no: usize, action_name: ActionName, player_id: usize, opposing_player_id: usize, card_data: &Vec<Card>) -> Self{
        ActionPacket{
            turn_no,
            action_name,
            player_id,
            opposing_player_id,
            card_data: card_data.clone(),
        }
    }
    pub fn get_turn_no(&self) -> usize {
        self.turn_no
    }
    pub fn get_action_name(&self) -> ActionName {
        self.action_name
    }
    pub fn get_player_id(&self) -> usize {
        self.player_id
    }
    pub fn get_opposing_player_id(&self) -> usize {
        self.opposing_player_id
    }
    pub fn get_card_data(&self) -> Vec<Card> {
        self.card_data.clone()
    }
    pub fn get_card(&self, index: usize) -> Card {
        if let Some(card) = self.card_data.get(index){
            return *card;
        } else {
            panic!("card does not exist at index {}!", {index});
        }
    }
}
