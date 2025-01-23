
use log::LevelFilter;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rustapp::history_public::{AOName, ActionObservation, History};
use rustapp::prob_manager::naive_prob::NaiveProb;
use rustapp::prob_manager::bit_prob::BitCardCountManager;
use std::fs::File;
use std::io::Write;
use env_logger::{Builder, Env, Target};
pub const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
// CURRENT BUG: add_subset_group never adds => check all redundant checks => to reconsider what really is redundant
// ANOTHER BUG: ok even if nothing is added, why on earth does it keep panicking
fn main() {
    let game_no = 50;
    let log_bool = true;
    let bool_know_priv_info = false;
    let print_frequency: usize = 10;
    // game_rnd_constraint(game_no, bool_know_priv_info, print_frequency, log_bool);
    game_rnd(game_no, bool_know_priv_info, print_frequency, log_bool);
}
pub fn game_rnd_constraint(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % print_frequency == 0 {
            println!("Game: {}", game);
            println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
            println!("Inferred Constraints Correct: {}/{}", inferred_constraints_correct, total_tries);
            println!("Impossible Cases Correct: {}/{}", impossible_constraints_correct, total_tries);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            log::info!("=== Prob ===");
            prob.printlog();
            log::info!("=== BitProb ===");
            bit_prob.printlog();
            // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
            // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
            new_moves = hh.generate_legal_moves();
            if new_moves[0].name() != AOName::CollectiveChallenge {
                log::info!("{}", format!("Legal Moves: {:?}", new_moves));
            } else {
                // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
                log::info!("{}", format!("Legal Moves: CollectiveChallenge"));
            }
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                log::info!("{}", format!("Choice: {:?}", output));
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_constructor(output.player_id(), &output.cards()[0])
                    } else {
                        prob.player_can_have_cards_constructor(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_constructor(output.player_id(), &output.card());
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards_constructor(6, output.cards());
                    if !true_legality {
                        log::info!("Illegal Move, Ending Game");
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao(&output, bool_know_priv_info);
                bit_prob.push_ao(&output, bool_know_priv_info);
                prob.update_calculated_states();
                let validated_public_constraints = prob.validated_public_constraints();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.validated_impossible_constraints();
                let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
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
                    false => "FAILED",
                });
                log::info!("validated_impossible_constraints: {:?}", validated_impossible_constraints);
                log::info!("test_impossible_constraints: {:?}", test_impossible_constraints);
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                log::info!("impossible_constraints: {}", match pass_impossible_constraints {
                    true => "PASSED",
                    false => "FAILED",
                });
                public_constraints_correct += pass_public_constraints as usize;
                inferred_constraints_correct += pass_inferred_constraints as usize;
                impossible_constraints_correct += pass_impossible_constraints as usize;
                total_tries += 1;
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
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn game_rnd(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut total_tries: usize = 0;
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
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
            new_moves = hh.generate_legal_moves();
            if new_moves[0].name() != AOName::CollectiveChallenge {
                log::info!("{}", format!("Legal Moves: {:?}", new_moves));
            } else {
                // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
                log::info!("{}", format!("Legal Moves: CollectiveChallenge"));
            }
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                log::info!("{}", format!("Choice: {:?}", output));
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_constructor(output.player_id(), &output.cards()[0])
                    } else {
                        prob.player_can_have_cards_constructor(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_constructor(output.player_id(), &output.card());
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards_constructor(6, output.cards());
                    if !true_legality {
                        log::info!("Illegal Move, Ending Game");
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao(&output, bool_know_priv_info);
                bit_prob.push_ao(&output, bool_know_priv_info);
                total_tries += 1;
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


pub fn logger(level: LevelFilter){
    // let log_file = File::create("app.log").unwrap();

    let log_file = File::create("card_count_validation.log").expect("Failed to create log file");

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
        // .filter(None, LevelFilter::Trace) // Adjust the log level as needed
        .filter(None, level) // Adjust the log level as needed
        // Direct logs to the file
        .target(Target::Pipe(Box::new(log_file)))
        // Apply the configuration
        .init();
}