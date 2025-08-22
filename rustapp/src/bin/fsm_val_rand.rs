use std::fs::OpenOptions;
use std::io::Write;
use env_logger::{Builder, Env, Target};
use log::LevelFilter;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rustapp::{history_public::Card, prob_manager::{engine::fsm_engine::FSMEngine, tracker::{collater::{Indicate, Unique}, informed_tracker::InformedTracker}}, traits::prob_manager::coup_analysis::CoupTraversal};

const LOG_FILE_NAME: &str = "./logs/fsm_val_rand.log";

fn main() {
    logger(LevelFilter::Trace);

    let game_no = 1000000;
    for _ in 0..game_no {
        let _ = clear_log();
        let mut engine = FSMEngine::new();
        let mut tracker: InformedTracker<Unique> = InformedTracker::new();
        
        let starting_cards = vec![
            vec![Card::Ambassador, Card::Ambassador],
            vec![Card::Ambassador, Card::Assassin],
            vec![Card::Assassin, Card::Assassin],
            vec![Card::Captain, Card::Captain],
            vec![Card::Captain, Card::Duke],
            vec![Card::Duke, Card::Duke],
            vec![Card::Contessa, Card::Contessa, Card::Contessa],
        ];
        let starting_cards = shuffled_same_shape(&starting_cards);

        let mut rng = thread_rng();
        let player = rng.gen_range(0..=5);
        engine.start_private(
            player,
            &[starting_cards[player][0], starting_cards[player][1]],
        );
        tracker.start_known(&starting_cards);
        let mut turn_no = 0;
        log::info!("Game State, turn: {turn_no}");
        log::info!("FSM State {:?}", engine.state);
        log::info!("Details {:?}", engine.state);
        while !engine.game_end() {
            let suggested_moves = engine.generate_legal_moves(&tracker);

            turn_no += 1;
            log::info!("Suggested moves: {:?}", &suggested_moves);
            let mut rng = thread_rng();
            if let Some(action) = suggested_moves.choose(&mut rng) {
                log::info!("Move chosen: {action:?}");
                engine.push_ao_private(action);
                tracker.push_ao_private(action);
            } else {
                panic!("suggested_moves is empty");
            }
            log::info!("");
            log::info!("Game State, turn: {turn_no}");
            log::info!("FSM Public Constraints {:?}", tracker.public_constraints);
            log::info!("FSM Inferred Constraints {:?}", tracker.inferred_constraints);
            log::info!("Details {:?}", engine.state);
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

// Flatten -> shuffle -> regroup to the SAME shape as the input.
fn shuffled_same_shape<T: Clone>(groups: &[Vec<T>]) -> Vec<Vec<T>> {
    let sizes: Vec<usize> = groups.iter().map(|g| g.len()).collect();

    let mut flat: Vec<T> = groups.iter().flat_map(|g| g.clone()).collect();

    flat.shuffle(&mut thread_rng());

    let mut out = Vec::with_capacity(sizes.len());
    let mut idx = 0;
    for sz in sizes {
        out.push(flat[idx..idx + sz].to_vec());
        idx += sz;
    }
    out
}