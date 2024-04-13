mod models;
mod game;
mod player;
mod gamestate;
use crate::game::Game;
// mod replay;


//TODO: Game should do a check on the actionpacket received from the player, that it is within those sent out!
// Then ping an error

//TODO: Replay => Create replay that stores Action_Packets and GameStates
// Functionality to backtrack will be needed for AI, so removing past moves
//TODO: models => adjust challenge to use revealshuffle functionality
//TODO: models => see notes for changes to structs


//TODO: Game => Msg passing of opposing_player_id and or player id 
//              and gamestate (those alive) 1. send gamestate information and 2. pass legal moves
//              returns action, opposing_player_id

//TODO: Game reset
//TODO: Make challenge just 2 rounds, first round is just shared information to adjust probabilities, second round is the actual effective one

// ==== Replay to be done with RustApp due to synergy ====
//TODO: Game => Replay storage
//TODO: Replay => make storage of all moves so far {Income, Player_id, opposing_player_id}
//              Game replay will store Vec of [Player_id, PolicyMove]
//TODO: Game => Msging with Players
//TONOTE: Make different players who all have the same trait: choose action

//Game to pull from rustapp as player
//rustapp should be independent

use std::fs::File;
use std::io::Write;
use log::{info, LevelFilter};
use env_logger::{Builder, Env, Target};
fn main() {

    logger();
    let mut max_turn: usize = 0; 
    for i in 0..1{
        info!("These are the LOGS for Coup: The Resistance!! Proudly Presented by Camland Inc.");
    
        let mut game: Game = Game::new();
        game.run_game();
        max_turn = max_turn.max(game.get_turn_no());
        if i % 1000000 == 0 {
            println!("Iteration {} complete", i);
        }
    }
    println!("{}", format!("End, Turn_no of Longest Game: {}",max_turn));

}

pub fn logger(){
    // let log_file = File::create("app.log").unwrap();

    let log_file = File::create("app.log").expect("Failed to create log file");

    // Initialize the env_logger builder with custom format
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            // Custom format: Timestamp, Level, and Message
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        // Set log level filter; this line is optional if using default_filter_or in from_env
        .filter(None, LevelFilter::Info) // Adjust the log level as needed
        // Direct logs to the file
        .target(Target::Pipe(Box::new(log_file)))
        // Apply the configuration
        .init();
}