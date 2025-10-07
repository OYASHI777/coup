use log::LevelFilter;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rustapp::history_public::{AOName, ActionObservation, Card, History};
use rustapp::prob_manager::models::backtrack::{InfoArray, InfoArrayTrait};
use rustapp::traits::prob_manager::coup_analysis::{CoupTraversal, ImpossibleConstraints};
use std::time::Instant;
pub const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
pub const LOG_FILE_NAME: &str = "just_test_replay_000000000.log";
// TODO: [REFACTOR] Lite to take history instead of store again and again
fn main() {
    benchmark::<InfoArray>();
}
fn benchmark<T: InfoArrayTrait>() {
    println!("InfoArray Complete Public");
    game_rnd_constraint_bt_bench::<T>(50000, false);
    println!("InfoArray Lazy Private");
    game_rnd_constraint_bt_bench_lazy::<T>(100000, false);
    println!("InfoArray Complete Private");
    game_rnd_constraint_bt_bench::<T>(50000, true);
    println!("InfoArray Lazy Private");
    game_rnd_constraint_bt_bench_lazy::<T>(100000, true);
}
pub fn game_rnd_constraint_bt_bench<I: InfoArrayTrait>(game_no: usize, bool_know_priv_info: bool) {
    let mut game: usize = 0;
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager<
        I,
    > = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let start_time = Instant::now();
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        let private_player: usize = 0;
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [
                Card::try_from(card_0).unwrap(),
                Card::try_from(card_1).unwrap(),
            ];
            // TODO: Fill those up
            bit_prob.start_private(private_player, &cards);
        } else {
            bit_prob.start_public(0);
        }
        while !hh.game_won() {
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);

            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned() {
                if output.name() == AOName::Discard {
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        bit_prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        bit_prob.player_can_have_cards_alive(output.player_id(), output.cards())
                    };
                    if !true_legality {
                        break;
                    }
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool =
                        bit_prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality {
                        break;
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool =
                        bit_prob.player_can_have_cards_alive(output.player_id(), output.cards());
                    if !true_legality {
                        break;
                    }
                }
                hh.push_ao(output);
                bit_prob.push_ao_public(&output);
                actions_processed += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            step += 1;
            if step > 1000 {
                break;
            }
            log::info!("");
        }
        game += 1;
    }
    let elapsed_time = start_time.elapsed();
    let process_per_action_us = elapsed_time.as_micros() as f64 / actions_processed as f64;
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!(
        "Estimated Time per nodes: {} micro seconds",
        process_per_action_us
    );
}
pub fn game_rnd_constraint_bt_bench_lazy<I: InfoArrayTrait>(
    game_no: usize,
    bool_know_priv_info: bool,
) {
    let mut game: usize = 0;
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager<
        I,
    > = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let start_time = Instant::now();
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        let private_player: usize = 0;
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [
                Card::try_from(card_0).unwrap(),
                Card::try_from(card_1).unwrap(),
            ];
            // TODO: Fill those up
            bit_prob.start_private(private_player, &cards);
        } else {
            bit_prob.start_public(0);
        }
        while !hh.game_won() {
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);

            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned() {
                if output.name() == AOName::Discard {
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        bit_prob
                            .player_can_have_card_alive_lazy(output.player_id(), output.cards()[0])
                    } else {
                        bit_prob
                            .player_can_have_cards_alive_lazy(output.player_id(), output.cards())
                    };
                    if !true_legality {
                        break;
                    }
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool =
                        bit_prob.player_can_have_card_alive_lazy(output.player_id(), output.card());
                    if !true_legality {
                        break;
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = bit_prob
                        .player_can_have_cards_alive_lazy(output.player_id(), output.cards());
                    if !true_legality {
                        break;
                    }
                }
                hh.push_ao(output);
                bit_prob.push_ao_public_lazy(&output);
                actions_processed += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            step += 1;
            if step > 1000 {
                break;
            }
            log::info!("");
        }
        game += 1;
    }
    let elapsed_time = start_time.elapsed();
    let process_per_action_us = elapsed_time.as_micros() as f64 / actions_processed as f64;
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!(
        "Estimated Time per nodes: {} micro seconds",
        process_per_action_us
    );
}
