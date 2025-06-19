use log::LevelFilter;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rustapp::history_public::{AOName, ActionObservation, Card, History};
use rustapp::prob_manager::backtracking_collective_constraints::{ActionInfo};
use rustapp::prob_manager::brute_prob_generic::{BruteCardCountManagerGeneric};
use rustapp::prob_manager::models::card_state_u64::CardStateu64;
use rustapp::traits::prob_manager::coup_analysis::{CoupPossibilityAnalysis, CoupTraversal};
use std::fs::{File, OpenOptions};
use std::io::{Write};
use std::time::Instant;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use env_logger::{Builder, Env, Target};
use ActionObservation::*;
pub const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
pub const LOG_FILE_NAME: &str = "just_test_replay_000000000.log";
// TODO: [REFACTOR] in this bin
// TODO: [REFACTOR] Lite to take history instead of store again and again
fn main() {
    let game_no = 100000000;
    let log_bool = true;
    let bool_know_priv_info = true;
    let bool_skip_exchange = false;
    let bool_lazy =  true;
    let print_frequency: usize = 100;
    let print_frequency_fast: usize = 5000;
    let min_dead_check: usize = 0;
    let num_threads = 12;
    game_rnd_constraint_bt_mt(num_threads, game_no, bool_know_priv_info, bool_skip_exchange, print_frequency, min_dead_check, bool_lazy);
    // game_rnd_constraint_bt_st_debug(game_no, bool_know_priv_info, print_frequency, min_dead_check, log_bool);
    
    game_rnd_constraint_bt_bench(100000, bool_know_priv_info);
    game_rnd_constraint_bt_bench_lazy(100000, bool_know_priv_info);
    game_rnd(game_no, bool_know_priv_info, bool_skip_exchange, print_frequency, min_dead_check, log_bool);
    test_variant_recurse();
}
// TODO: Move to collective_constraint when finalized
pub fn test_variant_recurse() {
    use rustapp::prob_manager::tools::recursion_fn::RecursionTest;
    RecursionTest::test_variant_recurse(LOG_FILE_NAME);
}
#[derive(Default)]
pub struct Stats {
    pub games: usize,
    pub max_steps: usize,
    pub public_constraints_correct: usize,
    pub inferred_constraints_correct: usize,
    pub impossible_constraints_correct: usize,
    pub impossible_constraints_2_correct: usize,
    pub impossible_constraints_3_correct: usize,
    pub over_inferred_count: usize,
    pub total_tries: usize,
    pub pushed_bad_move: usize,
    pub replay_string: String,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add(&mut self, other: &Stats) {
        self.games += other.games;
        self.max_steps = self.max_steps.max(other.max_steps);
        self.public_constraints_correct += other.public_constraints_correct;
        self.inferred_constraints_correct += other.inferred_constraints_correct;
        self.impossible_constraints_correct += other.impossible_constraints_correct;
        self.impossible_constraints_2_correct += other.impossible_constraints_2_correct;
        self.impossible_constraints_3_correct += other.impossible_constraints_3_correct;
        self.over_inferred_count += other.over_inferred_count;
        self.total_tries += other.total_tries;
        self.pushed_bad_move += other.pushed_bad_move;
    }
    
    pub fn public_correct(&self) -> bool {
        self.total_tries == self.public_constraints_correct
    }

    pub fn inferred_correct(&self) -> bool {
        self.total_tries == self.inferred_constraints_correct
    }

    pub fn overinferred(&self) -> bool {
        self.over_inferred_count > 0
    }
    pub fn impossible_correct(&self) -> bool {
        self.total_tries == self.impossible_constraints_correct
    }
    pub fn impossible_2_correct(&self) -> bool {
        self.total_tries == self.impossible_constraints_2_correct
    }
    pub fn impossible_3_correct(&self) -> bool {
        self.total_tries == self.impossible_constraints_3_correct
    }
    pub fn pushed_bad_move(&self) -> bool {
        self.pushed_bad_move > 0
    }

    pub fn games(&self) -> usize {
        self.games
    }

    pub fn print(&self) {
        println!("Game: {}", self.games);
        println!("Public Constraints Incorrect: {}/{}", self.total_tries - self.public_constraints_correct, self.total_tries);
        println!("Inferred Constraints Incorrect: {}/{}", self.total_tries - self.inferred_constraints_correct, self.total_tries);
        println!("Inferred Constraints Overinferred: {}/{}", self.over_inferred_count, self.total_tries);
        println!("Impossible Cases Incorrect: {}/{}", self.total_tries - self.impossible_constraints_correct, self.total_tries);
        println!("Impossible Cases Incorrect: {}/{}", self.total_tries - self.impossible_constraints_2_correct, self.total_tries);
        println!("Impossible Cases Incorrect: {}/{}", self.total_tries - self.impossible_constraints_3_correct, self.total_tries);
        println!("Bad Moves Pushed: {}/{}", self.pushed_bad_move, self.total_tries);
    }
}
pub fn game_rnd_constraint_bt_mt(num_threads: usize, game_no: usize, bool_know_priv_info: bool, bool_skip_exchange: bool, print_frequency: usize, min_dead_check: usize, bool_lazy: bool)
{
    let replay_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("replays_only.log")
            .expect("Unable to open replays_only.log"),
    ));
    
    let (tx, rx) = mpsc::channel();
    let games_per_thread = game_no / num_threads;
    let extra_games = game_no % num_threads;
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let thread_tx = tx.clone();
        let thread_games = games_per_thread + if i < extra_games {1} else {0};
        let thread_bool_know_priv_info = bool_know_priv_info;
        let thread_min_dead_check = min_dead_check;
        let handle = thread::spawn(move || {
            if bool_lazy {
                game_rnd_constraint_bt2_st_lazy(thread_games, thread_bool_know_priv_info, bool_skip_exchange, thread_min_dead_check, thread_tx);
            } else {
                game_rnd_constraint_bt2_st_new(thread_games, thread_bool_know_priv_info, bool_skip_exchange, thread_min_dead_check, thread_tx);
            }
        });
        handles.push(handle);
    }


    let mut final_stats = Stats::new();
    for received in rx {
        let mut log_replay = false;
        let mut status = "".to_string();
        if !received.public_correct() {
            status = format!("{status}|PUBLIC CONSTRAINTS WRONG");
            log_replay = true;
        }
        if !received.inferred_correct() {
            if received.overinferred() {
                status = format!("{status}|>+ OVERINFERRED INFERRED CONSTRAINTS");
            } else {
                status = format!("{status}|<- UNDERINFERRED INFERRED CONSTRAINTS");
            }
            log_replay = true;
        }
        if !received.impossible_correct() {
            status = format!("{status}|<- WRONG IMPOSSIBLE CONSTRAINTS");
            log_replay = true;
        }
        if received.pushed_bad_move() {
            status = format!("{status}|PUSHED BAD MOVE");
            log_replay = true;
        }
        // if !received.impossible_constraints_correct {
        //     status = format!("{status}|IMPOSSIBLE CONSTRAINTS WRONG");
        //     log_replay = true;
        // }
        if log_replay {
            let replay_data = received.replay_string.clone();
            if let Ok(mut file) = replay_file.lock() {
                // Append a separator and the replay content to the file.
                writeln!(file, "{status}").expect("Failed to write to file");
                writeln!(file, "{}", replay_data).expect("Failed to write replay");
            }
        }
        final_stats.add(&received);
        if final_stats.games() % print_frequency == 0 {
            final_stats.print();
        }
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
pub fn game_rnd_constraint_bt2_st_new(game_no: usize, bool_know_priv_info: bool, bool_skip_exchange: bool, min_dead_check: usize, tx: Sender<Stats>)
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new(false, false);
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraintLight> = BackTrackCardCountManager::new();
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        let private_player: Option<usize> = if bool_know_priv_info {
            Some(rng.gen_range(0..6))
        } else {
            None
        };
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            prob.start_private(private_player.unwrap(), &cards);
            bit_prob.start_private(private_player.unwrap(), &cards);
        } else {
            prob.start_public(7);
            bit_prob.start_public();
        }
        while !hh.game_won() {
            
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            // These are legal from a public sense only, but may be illegal depending on card 
            // TODO: [FIX] ExchangeChoice should generate regardless of hand as the card counter can determine it themselves
            new_moves = hh.generate_legal_moves(private_player);
            // TODO: create generate_legal_moves(None) for public and private
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            if bool_skip_exchange {
                new_moves.retain(|m| m.name() != AOName::Exchange);
            }
            let mut check_moves_prob = new_moves.clone();
            retain_legal_moves_with_card_constraints(&hh, &mut check_moves_prob, &mut prob, private_player);
            let mut check_moves_bit_prob = new_moves.clone();
            check_moves_bit_prob.retain(|ao| {
                if Some(ao.player_id()) == private_player {
                    bit_prob.is_legal_move_private(ao)
                } else {
                    bit_prob.is_legal_move_public(ao)
                }
            });
            if check_moves_prob.len() != check_moves_bit_prob.len() {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.sorted_public_constraints().clone());
                println!("inferred constraints: {:?}", prob.sorted_inferred_constraints().clone());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.player_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.player_impossible_constraints_paired());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.player_impossible_constraints_triple());
                println!("private_player: {:?}", private_player);
                println!("check_moves_prob: {:?}", check_moves_prob);
                println!("check_moves_bit_prob: {:?}", check_moves_bit_prob);
                panic!();
            }
            // TODO: [FIX] generate_legal_moves_with_card_constraints to determine legal Choices given hand and ExchangeDraw
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &mut prob, private_player);
            let (player, action_obs, action_info) = result.unwrap_or_else(|_| {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.validated_public_constraints());
                println!("inferred constraints: {:?}", prob.validated_inferred_constraints());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.validated_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.validated_impossible_constraints_2());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.validated_impossible_constraints_3());
                panic!("no legit moves found");
            });
            hh.push_ao(action_obs);
            if bool_know_priv_info && Some(action_obs.player_id()) == private_player {
                prob.push_ao_private(&action_obs);
                bit_prob.push_ao_private(&action_obs);
            } else {
                prob.push_ao_public(&action_obs);
                bit_prob.push_ao_public(&action_obs);
            }
            // TODO: Add test for if player is alive they cannot be all impossible
            let total_dead: usize = bit_prob.sorted_public_constraints().iter().map(|v| v.len()).sum();
            if total_dead >= min_dead_check {
                let validated_public_constraints = prob.sorted_public_constraints().clone();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.player_impossible_constraints().clone();
                let validated_impossible_constraints_2 = prob.player_impossible_constraints_paired().clone();
                let validated_impossible_constraints_3 = prob.player_impossible_constraints_triple().clone();
                // let validated_impossible_constraints_2 = prob.validated_impossible_constraints_2();
                // let validated_impossible_constraints_3 = prob.validated_impossible_constraints_3();
                let test_public_constraints = bit_prob.sorted_public_constraints().clone();
                let test_inferred_constraints = bit_prob.sorted_inferred_constraints().clone();
                let test_impossible_constraints = bit_prob.player_impossible_constraints().clone();
                let test_impossible_constraints_2 = bit_prob.player_impossible_constraints_paired().clone();
                let test_impossible_constraints_3 = bit_prob.player_impossible_constraints_triple().clone();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_impossible_constraints_2: bool = validated_impossible_constraints_2 == test_impossible_constraints_2;
                let pass_impossible_constraints_3: bool = validated_impossible_constraints_3 == test_impossible_constraints_3;
                let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                    test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                });
                // let pass_brute_prob_validity = prob.validate();
                stats.public_constraints_correct += pass_public_constraints as usize;
                stats.inferred_constraints_correct += pass_inferred_constraints as usize;
                stats.impossible_constraints_correct += pass_impossible_constraints as usize;
                stats.impossible_constraints_2_correct += pass_impossible_constraints_2 as usize;
                stats.impossible_constraints_3_correct += pass_impossible_constraints_3 as usize;
                stats.total_tries += 1;
                if !pass_public_constraints {
                    break;
                }
                if bool_test_over_inferred {
                    // what we are testing inferred too many things
                    stats.over_inferred_count += 1;
                    println!("public: {:?}", validated_public_constraints);
                    println!("vali: {:?}", validated_inferred_constraints);
                    println!("test: {:?}", test_inferred_constraints);
                    println!("test im 2: {:?}", test_impossible_constraints_2);
                    println!("{}", hh.get_replay_history_braindead());
                    panic!();
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred to many items!")
                }
                if !pass_inferred_constraints {
                    // println!("vali: {:?}", validated_inferred_constraints);
                    // println!("test: {:?}", test_inferred_constraints);
                    // println!("{}", hh.get_replay_history_braindead());
                    // panic!();
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred constraints do not match!")
                }
                if !pass_impossible_constraints {
                    // println!("vali: {:?}", validated_impossible_constraints);
                    // println!("test: {:?}", test_impossible_constraints);
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!()
                }
            }
            step += 1;
            if step > 1000 {
                break;
            }
        }
        if step > max_steps {
            max_steps = step;
        }
        stats.replay_string = hh.get_replay_history_braindead();
        stats.games += 1;
        tx.send(stats).unwrap();
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
}
pub fn game_rnd_constraint_bt2_st_lazy(game_no: usize, bool_know_priv_info: bool, bool_skip_exchange: bool, min_dead_check: usize, tx: Sender<Stats>)
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new(false, false);
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraintLight> = BackTrackCardCountManager::new();
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        let private_player: Option<usize> = if bool_know_priv_info {
            Some(rng.gen_range(0..6))
        } else {
            None
        };
        // let private_player: usize = 0;
        let skip_prob: f32 = 0.8;
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            prob.start_private(private_player.unwrap(), &cards);
            bit_prob.start_private(private_player.unwrap(), &cards);
        } else {
            prob.start_public(7);
            bit_prob.start_public();
        }
        while !hh.game_won() {
            
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            // These are legal from a public sense only, but may be illegal depending on card 
            // TODO: [FIX] ExchangeChoice should generate regardless of hand as the card counter can determine it themselves
            new_moves = hh.generate_legal_moves(private_player);
            // TODO: create generate_legal_moves(None) for public and private
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            if bool_skip_exchange {
                new_moves.retain(|m| m.name() != AOName::Exchange);
            }
            let mut check_moves_prob = new_moves.clone();
            retain_legal_moves_with_card_constraints(&hh, &mut check_moves_prob, &mut prob, private_player);
            let mut check_moves_bit_prob = new_moves.clone();
            check_moves_bit_prob.retain(|ao| {
                if Some(ao.player_id()) == private_player {
                    bit_prob.is_legal_move_private(ao)
                } else {
                    bit_prob.is_legal_move_public(ao)
                }
            });
            if check_moves_prob.len() != check_moves_bit_prob.len() {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.sorted_public_constraints().clone());
                println!("inferred constraints: {:?}", prob.sorted_inferred_constraints().clone());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.player_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.player_impossible_constraints_paired());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.player_impossible_constraints_triple());
                println!("check_moves_prob: {:?}", check_moves_prob);
                println!("check_moves_bit_prob: {:?}", check_moves_bit_prob);
                panic!();
            }
            // TODO: [FIX] generate_legal_moves_with_card_constraints to determine legal Choices given hand and ExchangeDraw
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &mut prob, private_player);
            let (player, action_obs, action_info) = result.unwrap_or_else(|_| {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.sorted_public_constraints().clone());
                println!("inferred constraints: {:?}", prob.sorted_inferred_constraints().clone());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.player_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.player_impossible_constraints_paired());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.player_impossible_constraints_triple());
                panic!("no legit moves found");
            });
            hh.push_ao(action_obs);
            let mut rng = thread_rng();
            let num: f32 = rng.gen_range(0.0..=1.0);
            if bool_know_priv_info && Some(action_obs.player_id()) == private_player {
                prob.push_ao_private(&action_obs);
                bit_prob.push_ao_private_lazy(&action_obs);
                if num < skip_prob {
                    continue;
                }
            } else {
                prob.push_ao_public(&action_obs);
                bit_prob.push_ao_public_lazy(&action_obs);
                if num < skip_prob {
                    continue;
                }
            }
            bit_prob.generate_all_constraints();
            let total_dead: usize = bit_prob.sorted_public_constraints().iter().map(|v| v.len()).sum();
            if total_dead >= min_dead_check {
                let validated_public_constraints = prob.sorted_public_constraints().clone();
                let validated_inferred_constraints = prob.sorted_inferred_constraints().clone();
                let validated_impossible_constraints = prob.player_impossible_constraints().clone();
                let validated_impossible_constraints_2 = prob.player_impossible_constraints_paired().clone();
                let validated_impossible_constraints_3 = prob.player_impossible_constraints_triple().clone();
                // let validated_impossible_constraints_2 = prob.validated_impossible_constraints_2();
                // let validated_impossible_constraints_3 = prob.validated_impossible_constraints_3();
                let test_public_constraints = bit_prob.sorted_public_constraints().clone();
                let test_inferred_constraints = bit_prob.sorted_inferred_constraints().clone();
                let test_impossible_constraints = bit_prob.player_impossible_constraints().clone();
                let test_impossible_constraints_2 = bit_prob.player_impossible_constraints_paired().clone();
                let test_impossible_constraints_3 = bit_prob.player_impossible_constraints_triple().clone();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_impossible_constraints_2: bool = validated_impossible_constraints_2 == test_impossible_constraints_2;
                let pass_impossible_constraints_3: bool = validated_impossible_constraints_3 == test_impossible_constraints_3;
                let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                    test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                });
                // let pass_brute_prob_validity = prob.validate();
                stats.public_constraints_correct += pass_public_constraints as usize;
                stats.inferred_constraints_correct += pass_inferred_constraints as usize;
                stats.impossible_constraints_correct += pass_impossible_constraints as usize;
                stats.impossible_constraints_2_correct += pass_impossible_constraints_2 as usize;
                stats.impossible_constraints_3_correct += pass_impossible_constraints_3 as usize;
                stats.total_tries += 1;
                if !pass_public_constraints {
                    break;
                }
                if bool_test_over_inferred {
                    // what we are testing inferred too many things
                    stats.over_inferred_count += 1;
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred to many items!")
                }
                if !pass_inferred_constraints {
                    // println!("vali: {:?}", validated_inferred_constraints);
                    // println!("test: {:?}", test_inferred_constraints);
                    // println!("{}", hh.get_replay_history_braindead());
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred constraints do not match!")
                }
                if !pass_impossible_constraints {
                    // println!("vali: {:?}", validated_impossible_constraints);
                    // println!("test: {:?}", test_impossible_constraints);
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!()
                }
            }
            step += 1;
            if step > 1000 {
                break;
            }
        }
        if step > max_steps {
            max_steps = step;
        }
        stats.replay_string = hh.get_replay_history_braindead();
        stats.games += 1;
        tx.send(stats).unwrap();
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
}
pub fn game_rnd_constraint_bt_st_debug(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, min_dead_check: usize, log_bool: bool)
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new(false, false);
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    while game < game_no {
        if game % print_frequency == 0 {
            println!("Debug Checked game: {game}");
        }
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // let private_player: usize = rng.gen_range(0..6);
        let private_player: usize = 0;
        let mut rng = thread_rng();
        let card_0: u8 = rng.gen_range(0..5);
        let card_1: u8 = rng.gen_range(0..5);
        let starting_hand = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            // TODO: Fill those up
            prob.start_private(private_player, &starting_hand);
            bit_prob.start_private(private_player, &starting_hand);
        } else {
            prob.start_public(7);
            bit_prob.start_public();
        }
        while !hh.game_won() {
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            // These are legal from a public sense only, but may be illegal depending on card 
            // TODO: [FIX] ExchangeChoice should generate regardless of hand as the card counter can determine it themselves
            new_moves = hh.generate_legal_moves(Some(private_player));
            // TODO: create generate_legal_moves(None) for public and private
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            // new_moves.retain(|m| m.name() != AOName::Exchange);
            // TODO: [FIX] generate_legal_moves_with_card_constraints to determine legal Choices given hand and ExchangeDraw
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &mut prob, Some(private_player));
            let (player, action_obs, action_info) = result.unwrap_or_else(|_| {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.validated_public_constraints());
                println!("inferred constraints: {:?}", prob.validated_inferred_constraints());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.validated_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.validated_impossible_constraints_2());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.validated_impossible_constraints_3());
                panic!("no legit moves found");
            });
            log::info!("{}", format!("Player: {} Choice: {:?}", player, action_obs));
            hh.push_ao(action_obs);
            if bool_know_priv_info && action_obs.player_id() == private_player {
                prob.push_ao_private(&action_obs);
                bit_prob.push_ao_private(&action_obs);
            } else {
                prob.push_ao_public(&action_obs);
                bit_prob.push_ao_public(&action_obs);
            }
            let total_dead: usize = bit_prob.sorted_public_constraints().iter().map(|v| v.len()).sum();
            if total_dead >= min_dead_check {
                let validated_public_constraints = prob.validated_public_constraints();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.validated_impossible_constraints();
                let test_public_constraints = bit_prob.sorted_public_constraints().clone();
                let test_inferred_constraints = bit_prob.sorted_inferred_constraints().clone();
                let test_impossible_constraints = bit_prob.player_impossible_constraints().clone();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                    test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                });
                // let pass_brute_prob_validity = prob.validate();
                stats.public_constraints_correct += pass_public_constraints as usize;
                stats.inferred_constraints_correct += pass_inferred_constraints as usize;
                stats.impossible_constraints_correct += pass_impossible_constraints as usize;
                stats.total_tries += 1;
                if !pass_public_constraints {
                    break;
                }
                if prob.len() == 0 {
                    println!("Found problematic game, please hold...");
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
                    panic!("Illegal move played somewhere above!");
                    break;
                }
                if bool_test_over_inferred {
                    // what we are testing inferred too many things
                    println!("Found problematic game, please hold...");
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
                    panic!("Inferred constraints do not match!");
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred to many items!")
                }
                if !pass_inferred_constraints {
                    // println!("vali: {:?}", validated_inferred_constraints);
                    // println!("test: {:?}", test_inferred_constraints);
                    // println!("{}", hh.get_replay_history_braindead());
                    println!("Found problematic game, please hold...");
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
                    panic!("Inferred constraints do not match!");
                    break;
                }
                if !pass_impossible_constraints {
                    // println!("vali: {:?}", validated_impossible_constraints);
                    // println!("test: {:?}", test_impossible_constraints);
                    println!("Found problematic game, please hold...");
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
                    panic!("Impossible Constraints Failed!");
                    
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                }
            }
            step += 1;
            if step > 1000 {
                break;
            }
            log::info!("");
        }
        if step > max_steps {
            max_steps = step;
        }
        stats.replay_string = hh.get_replay_history_braindead();
        stats.games += 1;
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
}
// TODO: Shift this to be a method in prob! or at least just to check a new_move!
pub fn generate_legal_moves_with_card_constraints(history: &History, new_moves: &mut Vec<ActionObservation>, prob: &mut BruteCardCountManagerGeneric<CardStateu64>, private_player: Option<usize>) -> Result<(usize, ActionObservation, Option<ActionInfo>), ()> {
    // Clone the moves and shuffle them in place
    new_moves.shuffle(&mut thread_rng());
    // This assumes all moves are by the same player
    // In the case of Challenge, it does not matter
    for candidate in new_moves.iter() {
        match candidate {
            Discard { player_id, card, no_cards } => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if *no_cards == 1 {
                        if prob.player_can_have_card_alive(*player_id, card[0]) {
                            return Ok((*player_id, *candidate, Some(ActionInfo::Discard { discard: card[0] })));
                        }
                    } else {
                        if prob.player_can_have_cards(*player_id, candidate.cards()) {
                            // todo!("Make multiple Discard");
                            // return (*player_id, *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
                            return Ok((*player_id, *candidate, None));
                        }
                    }
                } else {
                    if *no_cards == 1 {
                        if prob.player_can_have_card_alive(*player_id, card[0]) {
                            return Ok((*player_id, *candidate, Some(ActionInfo::Discard { discard: card[0] })));
                        }
                    } else {
                        if prob.player_can_have_cards(*player_id, candidate.cards()) {
                            // todo!("Make multiple Discard");
                            // return (candidate.player_id(), *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
                            return Ok((*player_id, *candidate, None));
                        }
                        log::info!("prob.player_can_have_cards({}, {:?}) = false", *player_id, candidate.cards());
                    }
                }
            },
            RevealRedraw { player_id, reveal: reveal_card , redraw: redraw_card} => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if prob.player_can_have_card_alive(*player_id, *reveal_card) {
                        if *reveal_card == *redraw_card {
                            return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *reveal_card, redraw: Some(*redraw_card), relinquish: None } )));
                        }
                        if prob.player_can_have_card_alive(6, *redraw_card) {
                            return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *reveal_card, redraw: Some(*redraw_card), relinquish: None } )));
                        }
                    }
                } else {
                    if prob.player_can_have_card_alive(*player_id, *reveal_card) {
                        return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *reveal_card, redraw: None, relinquish: None })));
                    }
                }
            },
            ExchangeDraw { player_id, card } => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if prob.player_can_have_cards(6, card) {
                        return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeDraw { draw: card.to_vec() })));
                    }
                } else {
                    return Ok((*player_id, *candidate, None));
                }
            },
            ExchangeChoice { player_id, relinquish } => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if let ExchangeDraw { card: draw, .. } = history.latest_move() {
                        let player_dead_cards = &prob.validated_public_constraints()[*player_id];
                        if prob.player_can_have_cards_after_draw(*player_id, player_dead_cards, relinquish, draw) {
                            return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeChoice { relinquish: relinquish.to_vec() })));
                        }
                    } 
                } else {
                    return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeDrawChoice { draw: vec![], relinquish: vec![] })));
                }
            },
            _ => {
                return Ok((candidate.player_id(), *candidate, None));
            }
        }
    }
    Err(())
}
pub fn retain_legal_moves_with_card_constraints(history: &History, new_moves: &mut Vec<ActionObservation>, prob: &mut BruteCardCountManagerGeneric<CardStateu64>, private_player: Option<usize>) {
    // Clone the moves and shuffle them in place
    // This assumes all moves are by the same player
    // In the case of Challenge, it does not matter
    pub fn is_legal(history: &History, ao: &ActionObservation, prob: &mut BruteCardCountManagerGeneric<CardStateu64>, private_player: Option<usize>) -> bool {
        match ao {
            Discard { player_id, card, no_cards } => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if *no_cards == 1 {
                        if prob.player_can_have_card_alive(*player_id, card[0]) {
                            return true
                        }
                    } else {
                        if prob.player_can_have_cards(*player_id, ao.cards()) {
                            // todo!("Make multiple Discard");
                            // return (*player_id, *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
                            return true
                        }
                    }
                } else {
                    if *no_cards == 1 {
                        if prob.player_can_have_card_alive(*player_id, card[0]) {
                            return true
                        }
                    } else {
                        if prob.player_can_have_cards(*player_id, ao.cards()) {
                            // todo!("Make multiple Discard");
                            // return (candidate.player_id(), *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
                            return true
                        }
                        log::info!("prob.player_can_have_cards({}, {:?}) = false", *player_id, ao.cards());
                    }
                }
            },
            RevealRedraw { player_id, reveal: reveal_card , redraw: redraw_card} => {
                if Some(*player_id) == private_player {
                    return prob.player_can_have_card_alive(*player_id, *reveal_card) 
                    && (*reveal_card == *redraw_card || prob.player_can_have_card_alive(6, *redraw_card))
                } else {
                    if prob.player_can_have_card_alive(*player_id, *reveal_card) {
                        return true
                    }
                }
            },
            ExchangeDraw { player_id, card } => {
                if Some(*player_id) == private_player {
                    if prob.player_can_have_cards(6, card) {
                        return true;
                    }
                } else {
                    return true;
                }
            },
            ExchangeChoice { player_id, relinquish } => {
                if private_player.is_some() && *player_id == private_player.unwrap() {
                    if let ExchangeDraw { card: draw, .. } = history.latest_move() {
                        let player_dead_cards = &prob.validated_public_constraints()[*player_id];
                        if prob.player_can_have_cards_after_draw(*player_id, player_dead_cards, relinquish, draw) {
                            return true
                        }
                    } 
                } else {
                    return true
                }
            },
            _ => {
                return true
            }
        }
        false
    }
    new_moves.retain(|ao| is_legal(history, ao, prob, private_player));
}
pub fn game_rnd_constraint_bt_bench(game_no : usize, bool_know_priv_info: bool) {
    let mut game: usize = 0;
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let mut start_time = Instant::now();
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
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            bit_prob.start_private(private_player, &cards);
        } else {
            bit_prob.start_public();
        }
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        bit_prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        bit_prob.player_can_have_cards_alive(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = bit_prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = bit_prob.player_can_have_cards_alive(output.player_id(), output.cards());
                    if !true_legality {
                        break    
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
        bit_prob.reset();
        game += 1;
    }
    let elapsed_time = start_time.elapsed();
    let process_per_action_us = elapsed_time.as_micros() as f64 / actions_processed as f64;
    println!("Benchmark for: {}", "backtracking_prob_2");
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!("Estimated Time per nodes: {} micro seconds", process_per_action_us);
}
pub fn game_rnd_constraint_bt_bench_lazy(game_no : usize, bool_know_priv_info: bool) {
    let mut game: usize = 0;
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let mut start_time = Instant::now();
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
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            bit_prob.start_private(private_player, &cards);
        } else {
            bit_prob.start_public();
        }
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        bit_prob.player_can_have_card_alive_lazy(output.player_id(), output.cards()[0])
                    } else {
                        bit_prob.player_can_have_cards_alive_lazy(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = bit_prob.player_can_have_card_alive_lazy(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = bit_prob.player_can_have_cards_alive_lazy(output.player_id(), output.cards());
                    if !true_legality {
                        break    
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
        bit_prob.reset();
        game += 1;
    }
    let elapsed_time = start_time.elapsed();
    let process_per_action_us = elapsed_time.as_micros() as f64 / actions_processed as f64;
    println!("Benchmark for: {}", "backtracking_prob_2");
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!("Estimated Time per nodes: {} micro seconds", process_per_action_us);
}
pub fn replay_game_constraint_bt(replay: Vec<ActionObservation>, bool_know_priv_info: bool, private_player: usize, starting_hand: &[Card; 2], log_bool: bool)
{
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new(true, true);
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    if log_bool {
        clear_log().expect("failed to clear log");
    }
    log::info!("Game : {}", game);
    let mut hh = History::new(0);
    let mut step: usize = 0;
    let mut new_moves: Vec<ActionObservation>;
    log::trace!("Game Made:");
    if bool_know_priv_info {
        prob.start_private(private_player, starting_hand);
        bit_prob.start_private(private_player, starting_hand);
    } else {
        prob.start_public(7);
        bit_prob.start_public();
    }
    while !hh.game_won() {
        
        // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        new_moves = hh.generate_legal_moves(None);
        let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &mut prob, None);
        let (_, _, _) = result.unwrap_or_else(|_| {
            println!("{}", hh.get_replay_history_braindead());
            println!("new_moves: {:?}", new_moves);
            println!("public constraints: {:?}", prob.validated_public_constraints());
            println!("inferred constraints: {:?}", prob.validated_inferred_constraints());
            println!("impossible constraints: {:?}", prob.validated_impossible_constraints());
            prob.set_impossible_constraints_2();
            println!("impossible constraints_2: {:?}", prob.validated_impossible_constraints_2());
            prob.set_impossible_constraints_3();
            println!("impossible constraints_3: {:?}", prob.validated_impossible_constraints_3());
            panic!("no legit moves found");
        });
        if let Some(output) = replay.get(step) {
            log::info!("{}", format!("Choice: {:?}", output));
            hh.push_ao(*output);
            if bool_know_priv_info && output.player_id() == private_player {
                prob.push_ao_private(&output);
                bit_prob.push_ao_private(&output);
            } else {
                prob.push_ao_public(&output);
                bit_prob.push_ao_public(&output);
            }
            log::info!("Just before validation");
            bit_prob.printlog();
            let validated_public_constraints = prob.validated_public_constraints();
            let validated_inferred_constraints = prob.validated_inferred_constraints();
            let validated_impossible_constraints = prob.validated_impossible_constraints();
            // prob.set_impossible_constraints_2();
            // prob.set_impossible_constraints_3();
            let validated_impossible_constraints_2 = prob.validated_impossible_constraints_2();
            let validated_impossible_constraints_3 = prob.validated_impossible_constraints_3();
            let test_public_constraints = bit_prob.sorted_public_constraints().clone();
            let test_inferred_constraints = bit_prob.sorted_inferred_constraints().clone();
            let test_impossible_constraints = bit_prob.player_impossible_constraints().clone();
            let test_impossible_constraints_2 = bit_prob.player_impossible_constraints_paired().clone();
            let test_impossible_constraints_3 = bit_prob.player_impossible_constraints_triple().clone();
            log::info!("validated_public_constraints: {:?}", validated_public_constraints);
            log::info!("test_public_constraints: {:?}", test_public_constraints);
            let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
            log::info!("public_constraints: {}", match pass_public_constraints {
                true => "PASSED",
                false => "FAILED",
            });
            log::info!("validated_inferred_constraints: {:?}", validated_inferred_constraints);
            log::info!("test_inferred_constraints: {:?}", test_inferred_constraints);
            let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
            log::info!("inferred_constraints: {}", match pass_inferred_constraints {
                true => "PASSED",
                false => {
                    "FAILED"
                },
            });
            log::info!("prob_states_count: {:?}", prob.len());
            log::info!("validated_impossible_constraints: {:?}", validated_impossible_constraints);
            log::info!("test_impossible_constraints: {:?}", test_impossible_constraints);
            log::info!("validated_impossible_constraints_2: {:?}", validated_impossible_constraints_2);
            log::info!("test_impossible_constraints_2: {:?}", test_impossible_constraints_2);
            log::info!("validated_impossible_constraints_3: {:?}", validated_impossible_constraints_3);
            log::info!("test_impossible_constraints_3: {:?}", test_impossible_constraints_3);
            let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
            log::info!("impossible_constraints: {}", match pass_impossible_constraints {
                true => "PASSED",
                false => "FAILED",
            });
            let pass_impossible_constraints_2: bool = validated_impossible_constraints_2 == test_impossible_constraints_2;
            log::info!("impossible_constraints_2: {}", match pass_impossible_constraints_2 {
                true => "PASSED",
                false => "FAILED",
            });
            let pass_impossible_constraints_3: bool = validated_impossible_constraints_3 == test_impossible_constraints_3;
            log::info!("impossible_constraints_3: {}", match pass_impossible_constraints_3 {
                true => "PASSED",
                false => "FAILED",
            });
            if !pass_inferred_constraints {
                // prob.print_legal_states();
                println!("vali: {:?}", validated_inferred_constraints);
                println!("test: {:?}", test_inferred_constraints);
                hh.print_replay_history_braindead();
                panic!()
            }
            if !pass_impossible_constraints || !pass_impossible_constraints_2 || !pass_impossible_constraints_3{
                hh.print_replay_history_braindead();
                panic!()
            }
            
        } else {
            log::trace!("End of Replay!");
            println!("=> no issues");
            return;
        }
        step += 1;
        if step > 1000 {
            break;
        }
        log::info!("");
    }
    
    if step > max_steps {
        max_steps = step;
    }
    hh.print_replay_history_braindead();
    hh.log_state();
    // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
    // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
    log::info!("");
    log::info!("Most Steps: {}", max_steps);
    println!("=> no issues");
    // println!("Most Steps: {}", max_steps);
    // println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    // println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    // println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    // println!("Total Tries: {}", total_tries);
}


pub fn game_rnd(game_no: usize, bool_know_priv_info: bool, bool_skip_exchange: bool, print_frequency: usize, min_dead_check: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new(false, false);
    let mut bit_prob: rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager = rustapp::prob_manager::backtracking_prob_hybrid::BackTrackCardCountManager::new();
    let mut total_tries: usize = 0;
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        let private_player: Option<usize> = if bool_know_priv_info {
            Some(rng.gen_range(0..6))
        } else {
            None
        };
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            prob.start_private(private_player.unwrap(), &cards);
            bit_prob.start_private(private_player.unwrap(), &cards);
        } else {
            prob.start_public(7);
            bit_prob.start_public();
        }
        // if game % (game_no / 10) == 0 {
        if game % print_frequency == 0 {
            println!("Game: {}", game);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            log::trace!("Game Made:");
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            log::info!("=== Prob ===");
            prob.printlog();
            log::info!("=== BitProb ===");
            bit_prob.printlog();
            // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
            // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
            new_moves = hh.generate_legal_moves(None);
            
            if bool_skip_exchange {
                new_moves.retain(|m| m.name() != AOName::Exchange);
            }
            let mut check_moves_prob = new_moves.clone();
            retain_legal_moves_with_card_constraints(&hh, &mut check_moves_prob, &mut prob, private_player);
            let mut check_moves_bit_prob = new_moves.clone();
            check_moves_bit_prob.retain(|ao| {
                if Some(ao.player_id()) == private_player {
                    bit_prob.is_legal_move_private(ao)
                } else {
                    bit_prob.is_legal_move_public(ao)
                }
            });
            if check_moves_prob.len() != check_moves_bit_prob.len() {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.sorted_public_constraints().clone());
                println!("inferred constraints: {:?}", prob.sorted_inferred_constraints().clone());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.player_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.player_impossible_constraints_paired());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.player_impossible_constraints_triple());
                println!("private_player: {:?}", private_player);
                println!("check_moves_prob: {:?}", check_moves_prob);
                println!("check_moves_bit_prob: {:?}", check_moves_bit_prob);
                panic!();
            }
            // TODO: [FIX] generate_legal_moves_with_card_constraints to determine legal Choices given hand and ExchangeDraw
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &mut prob, private_player);
            let (player, action_obs, action_info) = result.unwrap_or_else(|_| {
                println!("{}", hh.get_replay_history_braindead());
                println!("new_moves: {:?}", new_moves);
                println!("public constraints: {:?}", prob.validated_public_constraints());
                println!("inferred constraints: {:?}", prob.validated_inferred_constraints());
                prob.set_impossible_constraints();
                println!("impossible constraints: {:?}", prob.validated_impossible_constraints());
                prob.set_impossible_constraints_2();
                println!("impossible constraints_2: {:?}", prob.validated_impossible_constraints_2());
                prob.set_impossible_constraints_3();
                println!("impossible constraints_3: {:?}", prob.validated_impossible_constraints_3());
                panic!("no legit moves found");
            });
            hh.push_ao(action_obs);
            if bool_know_priv_info && Some(action_obs.player_id()) == private_player {
                prob.push_ao_private(&action_obs);
                bit_prob.push_ao_private(&action_obs);
            } else {
                prob.push_ao_public(&action_obs);
                bit_prob.push_ao_public(&action_obs);
            }
            // TODO: Add test for if player is alive they cannot be all impossible
            let total_dead: usize = bit_prob.sorted_public_constraints().iter().map(|v| v.len()).sum();
            if total_dead >= min_dead_check {
                let validated_public_constraints = prob.sorted_public_constraints().clone();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.player_impossible_constraints().clone();
                let validated_impossible_constraints_2 = prob.player_impossible_constraints_paired().clone();
                let validated_impossible_constraints_3 = prob.player_impossible_constraints_triple().clone();
                // let validated_impossible_constraints_2 = prob.validated_impossible_constraints_2();
                // let validated_impossible_constraints_3 = prob.validated_impossible_constraints_3();
                let test_public_constraints = bit_prob.sorted_public_constraints().clone();
                let test_inferred_constraints = bit_prob.sorted_inferred_constraints().clone();
                let test_impossible_constraints = bit_prob.player_impossible_constraints().clone();
                let test_impossible_constraints_2 = bit_prob.player_impossible_constraints_paired().clone();
                let test_impossible_constraints_3 = bit_prob.player_impossible_constraints_triple().clone();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_impossible_constraints_2: bool = validated_impossible_constraints_2 == test_impossible_constraints_2;
                let pass_impossible_constraints_3: bool = validated_impossible_constraints_3 == test_impossible_constraints_3;
                let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                    test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                });
                // let pass_brute_prob_validity = prob.validate();
                if !pass_public_constraints {
                    break;
                }
                if bool_test_over_inferred {
                    // what we are testing inferred too many things
                    println!("public: {:?}", validated_public_constraints);
                    println!("vali: {:?}", validated_inferred_constraints);
                    println!("test: {:?}", test_inferred_constraints);
                    println!("test im 2: {:?}", test_impossible_constraints_2);
                    println!("{}", hh.get_replay_history_braindead());
                    panic!();
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred to many items!")
                }
                if !pass_inferred_constraints {
                    // println!("vali: {:?}", validated_inferred_constraints);
                    // println!("test: {:?}", test_inferred_constraints);
                    // println!("{}", hh.get_replay_history_braindead());
                    // panic!();
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!("Inferred constraints do not match!")
                }
                if !pass_impossible_constraints {
                    // println!("vali: {:?}", validated_impossible_constraints);
                    // println!("test: {:?}", test_impossible_constraints);
                    break;
                    // let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!()
                }
            }
            step += 1;
            if step > 1000 {
                break;
            }
            log::info!("");
        }
        if step > max_steps {
            max_steps = step;
        }
        log::info!("{}", format!("Game Won : {:?}",step));
        hh.log_state();
        // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        log::info!("");
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
    log::info!("Most Steps: {}", max_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total Tries: {}", total_tries);
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
        .write(true)
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