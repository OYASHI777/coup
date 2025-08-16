use std::fs::OpenOptions;
use std::io::Write;
use env_logger::{Builder, Env, Target};
use log::LevelFilter;
use rand::{seq::SliceRandom, thread_rng};
use rustapp::{history_public::Card, prob_manager::{engine::fsm_engine::FSMEngine, tracker::informed_tracker::InformedTracker}, traits::prob_manager::coup_analysis::CoupTraversal};

const LOG_FILE_NAME: &str = "./logs/fsm_val_rand.log";

fn main() {
    logger(LevelFilter::Info);

    let game_no = 1000;
    for _ in 0..game_no {
        let mut engine = FSMEngine::new();
        let mut tracker = InformedTracker::new();
        // TODO: RANDOMIZE
        let starting_cards = vec![
            vec![Card::Ambassador, Card::Ambassador],
            vec![Card::Ambassador, Card::Assassin],
            vec![Card::Assassin, Card::Assassin],
            vec![Card::Captain, Card::Captain],
            vec![Card::Captain, Card::Duke],
            vec![Card::Duke, Card::Duke],
            vec![Card::Contessa, Card::Contessa, Card::Contessa],
        ];
        // TODO: RANDOMIZE
        let player = 0;
        engine.start_private(
            player,
            &[starting_cards[player][0], starting_cards[player][1]],
        );
        tracker.start_known(&starting_cards);
        // TODO: test if game actually ends
        let mut turn_no = 0;
        log::info!("Game State, turn: {turn_no}");
        log::info!("FSM State {:?}", engine.state);
        log::info!("Details {:?}", engine.state);
        log::info!("");
        while !engine.game_end() {
            let suggested_moves = engine.generate_legal_moves(&tracker);

            turn_no += 1;
            log::info!("Game State, turn: {turn_no}");
            log::info!("Suggested moves: {:?}", &suggested_moves);
            let mut rng = thread_rng();
            if let Some(action) = suggested_moves.choose(&mut rng) {
                log::info!("Move chosen: {action:?}");
                engine.push_ao_private(action);
                log::trace!("engine push_ao done!");
                tracker.push_ao_private(action);
                log::trace!("tracker push_ao done!");
            } else {
                panic!("suggested_moves is empty");
            }
            log::info!("FSM Public Constraints {:?}", tracker.public_constraints);
            log::info!("FSM Inferred Constraints {:?}", tracker.inferred_constraints);
            log::info!("Details {:?}", engine.state);
            log::info!("");
        }
    }

}
pub fn clear_log() -> std::io::Result<()> {
    // Open file with truncate flag to clear contents
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(LOG_FILE_NAME)?;
    Ok(())
}
pub fn logger(level: LevelFilter) {
    // Clear log file before initializing logger
    
    let _ = clear_log();
    let log_file = OpenOptions::new()
        .create(true)
        
        .append(true)
        .open(LOG_FILE_NAME)
        .expect("Failed to open log file");


    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, level)
        .target(Target::Pipe(Box::new(log_file)))
        .init();
}