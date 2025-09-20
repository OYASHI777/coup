use env_logger::{Builder, Env, Target};
use log::LevelFilter;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rustapp::history_public::ActionObservation;
use rustapp::prob_manager::engine::models::engine_state::EngineState;
use rustapp::{
    history_public::Card,
    prob_manager::{
        engine::fsm_engine::FSMEngine,
        tracker::{collater::Unique, informed_tracker::InformedTracker},
    },
    traits::prob_manager::coup_analysis::CoupTraversal,
};
use std::io::Write;
use std::{collections::HashSet, fs::OpenOptions};

const LOG_FILE_NAME: &str = "./logs/fsm_val_rand.log";
const CLEAR_LOG: bool = true;
const STORE_LOG: bool = true;
fn main() {
    // rand_game();
    gen_unique_paths();
}

/// This function is used to generate all possible paths a game can flow from
/// the start of a turn.
pub fn gen_unique_paths() {
    if STORE_LOG {
        logger(LevelFilter::Info);
        if CLEAR_LOG {
            let _ = clear_log();
        }
    }
    let game_no = 10000000;
    let standard_turn_player = 0;
    let standard_target_player = 5;
    let standard_other_player = 1;
    let standard_card = Card::Duke;

    let mut paths: HashSet<Vec<ActionObservation>> = HashSet::new();

    // Every turn store move into an intermediate vec
    // Every Turnstart or End state shove into hashset
    for i in 0..game_no {
        if i % 10000 == 0 {
            println!("Game_no: {i}");
        }
        let mut path_vec: Vec<ActionObservation> = Vec::with_capacity(20);

        let mut engine = FSMEngine::new();
        let mut tracker: InformedTracker<Unique> = InformedTracker::new();
        let mut replay = Vec::with_capacity(120);

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
        let mut turn_player = rng.gen_range(0..=5);
        let mut target_player = None;
        engine.start_private(
            turn_player,
            &[
                starting_cards[turn_player][0],
                starting_cards[turn_player][1],
            ],
        );
        tracker.start_known(&starting_cards);
        while !engine.game_end() {
            let suggested_moves = engine.generate_legal_moves(&tracker);

            let mut rng = thread_rng();
            if let Some(action) = suggested_moves.choose(&mut rng) {
                engine.push_ao_private(action);
                tracker.push_ao_private(action);
                replay.push(action.clone());
                match action {
                    ActionObservation::Steal {
                        opposing_player_id, ..
                    }
                    | ActionObservation::Assassinate {
                        opposing_player_id, ..
                    }
                    | ActionObservation::Coup {
                        opposing_player_id, ..
                    } => {
                        target_player = Some(*opposing_player_id);
                    }
                    _ => (),
                }
                let cleaned_action = clean_action(
                    *action,
                    turn_player,
                    target_player,
                    standard_turn_player,
                    standard_other_player,
                    standard_target_player,
                    standard_card,
                );
                path_vec.push(cleaned_action);
            } else {
                panic!("suggested_moves is empty");
            }
            match engine.state.engine_state {
                EngineState::TurnStart(turn_start) => {
                    if paths.insert(path_vec.clone()) {
                        log::info!("{:?}", path_vec);
                    }
                    path_vec.clear();
                    turn_player = turn_start.player_turn;
                    target_player = None;
                }
                EngineState::End(_) => {
                    if paths.insert(path_vec.clone()) {
                        log::info!("{:?}", path_vec);
                    }
                    path_vec.clear();
                }
                _ => {}
            }
        }
    }
}

pub fn clean_action(
    action: ActionObservation,
    turn_player: usize,
    target_player: Option<usize>,
    standard_turn_player: usize,
    standard_other_player: usize,
    standard_target_player: usize,
    standard_card: Card,
) -> ActionObservation {
    fn allocate_player(
        player: usize,
        turn_player: usize,
        target_player: Option<usize>,
        standard_turn_player: usize,
        standard_other_player: usize,
        standard_target_player: usize,
    ) -> usize {
        if player == turn_player {
            standard_turn_player
        } else if Some(player) == target_player {
            standard_target_player
        } else {
            standard_other_player
        }
    }
    match action {
        ActionObservation::ChallengeAccept {
            player_id,
            opposing_player_id,
        } => ActionObservation::ChallengeAccept {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::Income { player_id } => ActionObservation::Income {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::ForeignAid { player_id } => ActionObservation::ForeignAid {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::Tax { player_id } => ActionObservation::Tax {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::Steal {
            player_id,
            opposing_player_id,
            amount,
        } => ActionObservation::Steal {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            amount,
        },
        ActionObservation::Assassinate {
            player_id,
            opposing_player_id,
        } => ActionObservation::Assassinate {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::Coup {
            player_id,
            opposing_player_id,
        } => ActionObservation::Coup {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::CollectiveChallenge {
            opposing_player_id,
            final_actioner,
            ..
        } => ActionObservation::CollectiveChallenge {
            participants: [false; 6],
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            final_actioner: allocate_player(
                final_actioner,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::CollectiveBlock {
            opposing_player_id,
            final_actioner,
            ..
        } => ActionObservation::CollectiveBlock {
            participants: [false; 6],
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            final_actioner: allocate_player(
                final_actioner,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::BlockSteal {
            player_id,
            opposing_player_id,
            ..
        } => ActionObservation::BlockSteal {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            card: Card::Captain,
        },
        ActionObservation::BlockAssassinate {
            player_id,
            opposing_player_id,
        } => ActionObservation::BlockAssassinate {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            opposing_player_id: allocate_player(
                opposing_player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::Discard {
            player_id,
            no_cards,
            ..
        } => ActionObservation::Discard {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            card: [standard_card; 2],
            no_cards,
        },
        ActionObservation::RevealRedraw {
            player_id, reveal, ..
        } => ActionObservation::RevealRedraw {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            reveal,
            redraw: standard_card,
        },
        ActionObservation::Exchange { player_id } => ActionObservation::Exchange {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
        },
        ActionObservation::ExchangeDraw { player_id, .. } => ActionObservation::ExchangeDraw {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            card: [standard_card; 2],
        },
        ActionObservation::ExchangeChoice { player_id, .. } => ActionObservation::ExchangeChoice {
            player_id: allocate_player(
                player_id,
                turn_player,
                target_player,
                standard_turn_player,
                standard_other_player,
                standard_target_player,
            ),
            relinquish: [standard_card; 2],
        },
        ActionObservation::ChallengeDeny => action,
        _ => action,
    }
}

pub fn rand_game() {
    if STORE_LOG {
        logger(LevelFilter::Info);
    }

    let game_no = 1;
    for i in 0..game_no {
        if STORE_LOG && CLEAR_LOG {
            let _ = clear_log();
        }
        if i % 10000 == 0 {
            println!("Game_no: {i}");
        }
        let mut engine = FSMEngine::new();
        let mut tracker: InformedTracker<Unique> = InformedTracker::new();
        let mut replay = Vec::with_capacity(120);

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
                replay.push(action.clone());
            } else {
                panic!("suggested_moves is empty");
            }
            log::info!("");
            log::info!("Game State, turn: {turn_no}");
            log::info!("FSM Public Constraints {:?}", tracker.public_constraints);
            log::info!(
                "FSM Inferred Constraints {:?}",
                tracker.inferred_constraints
            );
            log::info!("Details {:?}", engine.state);
        }
        log::warn!("Replay: {:?}", replay);
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
