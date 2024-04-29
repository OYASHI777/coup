// mod unittests;
// mod string_utils;
// mod prob_manager;
use std::collections::{HashSet, HashMap};
pub mod history_public;
pub mod pcmccfr;
use crossbeam::thread;
use pcmccfr::ReachProbability;
use history_public::{ActionObservation, History, AOName, Card};
use std::fs::File;
use std::io::Write;
use std::io::{self};
use log::{info, LevelFilter};
use env_logger::{Builder, Env, Target};
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
pub mod prob_manager;
// use prob_manager::prob_state::ProbState;
mod string_utils;
use prob_manager::naive_prob::{NaiveProb};
use prob_manager::constraint::{GroupConstraint, CollectiveConstraint};
use std::time::Instant;
use rand::prelude::IteratorRandom;
use std::sync::Mutex;
// QUICK TEMP: Exchange Draw showing 2 cards should prune the other groups? because they found out the pile has 2 cards
//              Make Func to initialise past constraint history based on player perspective in naive_prob
//              Integrate this by having an initial constraint history that can be loaded in

// MACRO Objective
//     1. Settle Constraints and group representation history
//          1. Make Group Constraint
//          2. Adjust Collective Constraints to include new logic
//          3. Shift Constraint history to history
//     2. Make Constraints History seperate from History
//          [DONE] - Can identify impossible moves that might be suggested by history
//          [Constraints] - TODO C (TO WRITE A DOCUMENT FOR THIS TOO)
//          a) Personal, Group, Private (Exchange card seen which are chance sampled)
//          [Illegal Action Pruning in History] - TODO D
//          b) So illegal actions will be pruned here -> naive prob will only calculate the probabilities
//          c) Might a private/public exchange choice? [Public Only]
//                  If private exchange choice and card shown, then it is consistent to receive starting hand too
//                  No need to receive starting hand. The AI should work without knowing card dist see ReBeL Paper PBS Game type
//                  May avoid private exchange choice by having probabilities all conditional on infostate!
//                  Prob dont need private exchange choice as the choice is different for each infostate, so its not a commmon public choice
//                  So the choice rearranges the infostates and will not be stored.
//          [Private Constraints] - TODO B
//          d) But card seen on draw is random sampled by MC and so will be private information
//                  Each player on their exchangedraw will have some private information
//                  Store for each player? Will all then calculate probability based on all private information?
//                  a) Perspective 1: All calculate based on private information because its a simulation of what might happen
//                      Maybe thats why the history also stores private information | Store private information not private action
//                  b) So [Think] of if we need ambassador action! We do need it stored in strategy, and reflected in self play but idt we need it for simulation
//                      We do not need it stored in past history as we dont care about card state in history
//                      Self play should be with rust game.
//                  c) Only for exchangedraw not revealredraw
//          [Exchange Draw] - TODO A
//          d) Need exchangedraw? Yes
//                  Will however need to serialise the exchangechoice? Consider how to do this!
//                  Ok so we do need exchange draw because we will need to take an input for what card was drawn in an extensible way
//          e) What about revealredraw? and initial distribution?
//              I think we can ignore initial distribution because its uniform, and we want the model to not need to know starting card dist
//              Ignoring revealredraw chance ignores the proper prob of redraw and assumes uniform very wrongly
//              We could do MC for revealredraw and have a private constraint for it. We [Do not Serialise RevealRedraw private information]
//              [Serialise The Revealed Card] [Do not Serialise Drawn Card]
//              We [Serialise the Exchange Draw] because the exchange choices depends on it. You can only choose cards based on the pool given and infostate
//              Reveal Redraw is different as its information about your past hand, which we wish to represent in infostates not private info
//              We can however, represent the conditional prob NN input as a work around
//              For revealredraw, no choice depends on it, the game continues
//              Serialisation is only for future simulations, not past history so there should be no conflict with self play
//          f) How to choose an action for revealredraw or discard, as it affects infostate!
//              Each infostate will have a different Mixed Strategy and Best Response
//              We are sampling all possible actions so [no issue]
//          g) What happens in self play?
//              In past history, we know they Exchanged and saw something, but we do not know which, and we do not want to MC sample the past.
//              We can just not include them in private constraint, and continue only with public constraints
//              Might need private constraint of exchangedraw that was realised for the playing player!
//              So PMCCFR should take history that is public
//              So it should store a constraint history for simulation and a constraint history for future self play
//     2. Run a test PMCCFR with dummy value function

// TODO: Add comments for every function and file... its getting big

// TODO: Make constraint work properly
    // SUBTODO: Consider how to make only legal moves possible?
    // This should be handled on probability side. The game side only offers moves that are possible because it has game hand
    // Check according to constraints only possible?
        // Possible, check if the group constraints when merged includes the particular player
    // Might want to add constraints to history or consider if constraint should be tracked on both prob and history side
    // I think history should track constraints and naive_prob should take it as an input

//TOADD: Make the constraints a hash and store them locally to be preloaded
    // Since it can be stored, jsut test if belief prob are right then use this
    //TOCONSIDER: Next time if I should break up the hashmap to be stored based on groupings of depth 0-10, 11-20 etc..
    // For now just test how large it can be

//TODO: Make chance function for RevealRedraw and ExchangeChoice
    // For ExchangeChoice, dont have to actually calculate the probabilities
    // For each infostate Just filter out the possible states out of all states, randomly choose one state, and randomly draw two card for the new state
    // For RevealRedraw
    // A players infostate transitions to another infostate if reveal "A" all other than states with "A" are kept
    // "AA" "AB" "AC" "AD" "AE" are current and next infostates
    // Filter first all with "A"
    // For each current infostate filter again for the double constraint randomly choose a state, randomly choose among the 4 possible draws 
//TODO: Load Hashmap by depth instead figure out later!

//TODO: Make CardState into statehistory
//TODO: Make naive probability belief generator
//TODO: Make PureCFR run it for every initialised state => initialised state determines chance actions
//TODO: Since initialised policy is uniform, you can train basic value function based off of random games or dont

// 2024-03-23T23:18:59 [INFO] - Time taken for Optimal JC Filter: 300ns
// 2024-03-23T23:18:59 [INFO] - Time taken for Optimal PC Filter: 32.8897ms
// 2024-03-23T23:18:59 [INFO] - Time taken for Optimal GC Filter: 100ns
// 2024-03-23T23:18:59 [INFO] - Total Time taken for filter_state_optimal: 119.5825ms
fn main() {

    // game_rnd(1000, true);
    test_satis();
    // game_rnd_constraint(100000, true);
    // error_farmer(1000000, true);
    // find_overflow(500000, 200);
    // test_par_constructor(100000, false);
    // test_impossible_state(10000, true);
    // test_belief(20000000);
    // make_belief(20000000);
    // game_rnd(20000000, false);
    // test_filter(1000);
    // test_reach(); 
    // test_shuffle(100);
}
pub fn find_overflow(game_no: usize, rep_no: usize) {
    logger();
    for i in 0..rep_no {
        overflow_farmer(game_no, true);
        let output: Result<(), io::Error> = clear_logs();
    }
}
pub fn test_satis(){
    logger();
    let mut prob = NaiveProb::new();
//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(3, Card::Duke);
//     colcon.add_public_constraint(1, Card::Duke);

//     colcon.add_public_constraint(4, Card::Contessa);
//     colcon.add_public_constraint(0, Card::Ambassador);
//     colcon.add_public_constraint(0, Card::Contessa);
//     colcon.add_public_constraint(2, Card::Contessa);
//     colcon.add_public_constraint(2, Card::Assassin);

//     let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 1, 1], Card::Captain, 3 );
//     let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 1);
//     let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 1, 1], Card::Duke, 1);
//     let group4: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 1, 1], Card::Assassin, 2);
//     let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Captain, 2);

//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     colcon.add_raw_group(group4);
//     colcon.add_raw_group(group5);
//     colcon.add_inferred_groups();
//     colcon.group_redundant_prune();
//     log::info!(" === Test 1 === ");
//     colcon.printlog();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Ambassador);
//     if output {
//         println!("Test 1 Legal Wrong");
//     } else {
//         println!("Test 1 Illegal Correct");
//     }

//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(0, Card::Assassin);
//     colcon.add_public_constraint(2, Card::Ambassador);
//     colcon.add_public_constraint(3, Card::Duke);

//     colcon.add_public_constraint(4, Card::Contessa);
//     colcon.add_public_constraint(4, Card::Contessa);
//     colcon.add_public_constraint(1, Card::Assassin);
//     colcon.add_public_constraint(1, Card::Contessa);
//     colcon.add_public_constraint(5, Card::Ambassador);
//     colcon.add_public_constraint(5, Card::Duke);
//     let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Ambassador, 1 );
//     let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Ambassador, 2 );
//     let group3: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 1, 0, 0, 1], Card::Captain, 2 );
//     let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Captain, 1 );
//     let group5: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Assassin, 2 );
//     let group6: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Duke, 1 );
//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     colcon.add_raw_group(group4);
//     colcon.add_raw_group(group5);
//     colcon.add_raw_group(group6);
//     log::info!(" === Test 2 === ");
//     colcon.printlog();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 0, &Card::Captain);
//     if output {
//         println!("Test 2 Legal Correct");
//     } else {
//         println!("Test 2 Illegal Wrong");
//     }
//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(5, Card::Captain);
//     colcon.add_public_constraint(1, Card::Assassin);
//     colcon.add_public_constraint(3, Card::Duke);
//     colcon.add_public_constraint(2, Card::Contessa);

//     let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 1, 0, 0, 1], Card::Ambassador, 2 );
//     let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 1, 0, 0, 1], Card::Captain, 2 );
//     let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Contessa, 1 );
//     let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Captain, 1 );
//     let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Ambassador, 1 );
//     let group6: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Assassin, 1 );
//     let group7: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Duke, 2 );
//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     colcon.add_raw_group(group4);
//     colcon.add_raw_group(group5);
//     colcon.add_raw_group(group6);
//     colcon.add_raw_group(group7);
//     log::info!(" === Test 3 === ");
//     colcon.printlog();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Duke);
//     if output {
//         println!("Test 3 Legal Wrong");
//     } else {
//         println!("Test 3 Illegal Correct");
//     }
//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(5, Card::Captain);
//     colcon.add_public_constraint(1, Card::Assassin);

//     colcon.add_public_constraint(3, Card::Duke);
//     colcon.add_public_constraint(3, Card::Contessa);
//     colcon.add_public_constraint(2, Card::Ambassador);
//     colcon.add_public_constraint(2, Card::Assassin);
//     colcon.add_public_constraint(0, Card::Captain);
//     colcon.add_public_constraint(0, Card::Assassin);
//     colcon.add_public_constraint(4, Card::Duke);
//     colcon.add_public_constraint(4, Card::Duke);

//     let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Ambassador, 2 );
//     let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 1, 1], Card::Captain, 2 );
//     let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Captain, 1 );

//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     log::info!(" === Test 4 === ");
//     // This illegal wrong this is no reproducible 2 times
//     colcon.printlog();
//     // inferred is wrong
//     colcon.add_inferred_groups();
//     colcon.group_redundant_prune();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Captain);
//     if output {
//         println!("Test 4 Legal Correct");
//     } else {
//         println!("Test 4 Illegal Wrong");
//     }
//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(0, Card::Captain);
//     colcon.add_public_constraint(4, Card::Assassin);
//     colcon.add_public_constraint(2, Card::Contessa);

//     let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Duke, 2 );
//     let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 1 );
//     let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1 );
//     let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain , 2 );
    
//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     colcon.add_raw_group(group4);
//     log::info!(" === Test 5 === ");
//     // This illegal wrong this is no reproducible 2 times
//     colcon.printlog();
//     // inferred is wrong
//     colcon.add_inferred_groups();
//     colcon.group_redundant_prune();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Duke);
//     if output {
//         println!("Test 5 Legal Correct");
//     } else {
//         println!("Test 5 Illegal Wrong");
//     }

//     let mut colcon = CollectiveConstraint::new();
//     colcon.add_public_constraint(0, Card::Ambassador);
//     colcon.add_public_constraint(5, Card::Assassin);
//     colcon.add_public_constraint(3, Card::Contessa);

//     colcon.add_public_constraint(4, Card::Assassin);
//     colcon.add_public_constraint(4, Card::Captain);
//     colcon.add_public_constraint(1, Card::Assassin);
//     colcon.add_public_constraint(1, Card::Duke);
    
//     let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 1, 0, 0, 1], Card::Captain, 1 );
//     let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 1, 0, 0, 1], Card::Ambassador, 3 );
//     let group3: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 1, 0, 0, 1], Card::Duke, 1 );
//     let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Ambassador , 2 );
//     let group5: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Contessa , 2 );
    
//     colcon.add_raw_group(group1);
//     colcon.add_raw_group(group2);
//     colcon.add_raw_group(group3);
//     colcon.add_raw_group(group4);
//     colcon.add_raw_group(group5);
//     log::info!(" === Test 6 === ");
//     // This illegal wrong this is no reproducible 2 times
//     colcon.printlog();
//     // inferred is wrong
//     colcon.add_inferred_groups();
//     colcon.group_redundant_prune();
//     let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Captain);
//     if output {
//         println!("Test 6 Legal Wrong");
//     } else {
//         println!("Test 6 Illegal Correct");
//     }
    let mut colcon = CollectiveConstraint::new();
    colcon.add_public_constraint(5, Card::Assassin);

    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(3, Card::Contessa);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(2, Card::Duke);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Contessa, 0,  1);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 1, 1], Card::Captain, 0, 2 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 1 );
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke , 0, 1 );
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Captain , 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);
    log::info!(" === Test 7 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Assassin);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 1, &Card::Assassin).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();
    let start_time = Instant::now();
    colcon.add_raw_public_constraint(1, Card::Assassin);
    let construct_output: Option<String> = colcon.par_constructor(&colcon);
    let elapsed_time = start_time.elapsed();
    println!("Construct Time: {:?}", elapsed_time);
    if !construct_output.is_none() {
        println!("Test 7 Construct: Legal");
    } else {
        println!("Test 7 Construct: Illegal");
    }
    if output {
        println!("Test 7 Legal Correct");
    } else {
        println!("Test 7 Illegal Wrong");
    }
    let mut colcon = CollectiveConstraint::new();
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(0, Card::Assassin);

    colcon.add_public_constraint(4, Card::Captain);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(1, Card::Captain);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0,  2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Captain, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Ambassador, 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    log::info!(" === Test 8 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();
    let dead_hm = colcon.dead_card_count();
    log::trace!("dead_card_count: {:?}", dead_hm);
    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Ambassador);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 2, &Card::Ambassador).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();
    let start_time = Instant::now();
    colcon.add_raw_public_constraint(2, Card::Ambassador);
    let construct_output: Option<String> = colcon.par_constructor(&colcon);
    let elapsed_time = start_time.elapsed();
    println!("Construct Time: {:?}", elapsed_time);
    if !construct_output.is_none() {
        println!("Test 8 Construct: Legal");
    } else {
        println!("Test 8 Construct: Illegal");
    }
    if output {
        println!("Test 8 Legal Wrong");
    } else {
        println!("Test 8 Illegal Correct");
    }

    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Captain);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(3, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Captain, 0,  2);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 1, 0, 1], Card::Contessa, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Assassin, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Duke, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);

    log::info!(" === Test 9 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 5, &Card::Contessa);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 5, &Card::Contessa).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();
    colcon.add_raw_public_constraint(5, Card::Contessa);
    let start_time = Instant::now();
    let construct_output: Option<String> = colcon.par_constructor(&colcon);
    let elapsed_time = start_time.elapsed();
    println!("Construct Time: {:?}", elapsed_time);
    if !construct_output.is_none() {
        println!("Test 9 Construct: Legal");
    } else {
        println!("Test 9 Construct: Illegal");
    }
    if output {
        println!("Test 9 Legal Correct");
    } else {
        println!("Test 9 Illegal Wrong");
    }
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(3, Card::Duke);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(0, Card::Contessa);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Duke, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Duke, 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);


    log::info!(" === Test 10 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Assassin);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 1, &Card::Assassin).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();
    colcon.add_raw_public_constraint(1, Card::Assassin);
    let start_time = Instant::now();
    let construct_output: Option<String> = colcon.par_constructor(&colcon);
    let elapsed_time = start_time.elapsed();
    println!("Construct Time: {:?}", elapsed_time);
    if !construct_output.is_none() {
        println!("Test 10 Construct: Legal");
    } else {
        println!("Test 10 Construct: Illegal");
    }
    if output {
        println!("Test 10 Legal Correct");
    } else {
        println!("Test 10 Illegal Wrong");
    }
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(5, Card::Duke);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);

    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 1, 1, 0, 1], Card::Captain, 0, 3);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Duke, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 1, 0, 1], Card::Captain, 0, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Ambassador, 0, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Assassin, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);


    log::info!(" === Test 11 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Ambassador);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 3, &Card::Ambassador).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();
    if output {
        println!("Test 11 Legal Wrong");
    } else {
        println!("Test 11 Illegal Correct");
    }
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Captain);
    colcon.add_public_constraint(1, Card::Duke);
    colcon.add_public_constraint(5, Card::Captain);
    colcon.add_public_constraint(5, Card::Duke);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Contessa);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Ambassador, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 1, 0, 1], Card::Captain, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Ambassador, 0, 3);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Assassin, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);


    log::info!(" === Test 12 === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Assassin);
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 2, &Card::Assassin).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();

    if output {
        println!("Test 12 Legal Wrong");
    } else {
        println!("Test 12 Illegal Correct");
    }
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(1, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Duke);
    colcon.add_public_constraint(3, Card::Contessa);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Contessa);
    
    let group1: GroupConstraint = GroupConstraint::new_list([1, 1, 0, 0, 1, 0, 1], Card::Captain, 0, 3);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Ambassador, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 1, 0, 1], Card::Captain, 0, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Duke, 1, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Assassin, 0, 2);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);


    log::info!(" === Test Stack Overflow === ");
    // This illegal wrong this is no reproducible 2 times
    colcon.printlog();

    println!("Before canhavecard");
    let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 0, &Card::Duke);
    println!("Before canhavetest");
    let brute_output: bool = !prob.can_player_have_card_test(&colcon, 0, &Card::Duke).is_none();
    log::trace!("Brute: {}", brute_output);
    prob.filter_state_simple_test(&colcon);
    prob.log_calc_state();

    if output {
        println!("Test Stack Overflow Legal Wrong");
    } else {
        println!("Test Stack Overflow Illegal Correct");
    }
}
pub fn test_shuffle(iterations: usize){
    logger();
    let mut prob: NaiveProb = NaiveProb::new();

    for i in 0..iterations {
        let mut hh = History::new(0);

        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        let top: usize = rng.gen_range(50..200);
        let limit = rng.gen_range(0..10);

    // let limit: usize = 200;
        while !hh.game_won() {
            
            new_moves = hh.generate_legal_moves();
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                hh.push_ao(output);
                prob.push_ao(&output);
            } else {
                break;
            }
            step += 1;
            if step > limit {
                break;
            }
        }
    
    // So for small vectors after optimal_filter maybe around 150000 size, should shuffle then iterate through
    // For large vectors like 1500000 in size, maybe try to pogo jump it. The max search if no constraint is 1500000 - 90000 incredibly unlkely
    // 400ms
        let start_time = Instant::now();
        let output = prob.chance_reveal_redraw(0, vec!["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"].into_iter().map(|s| s.to_string()).collect());
        let elapsed_time = start_time.elapsed();
        log::info!("Time: {:?}", elapsed_time);
        log::info!("Output: {:?}", output);
        let start_time = Instant::now();
        let output = prob.chance_reveal_redraw_exit(0, vec!["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD","BE", "CC", "CD", "CE", "DD", "DE", "EE"].into_iter().map(|s| s.to_string()).collect());
        let elapsed_time = start_time.elapsed();
        log::info!("Time Exit: {:?}", elapsed_time);
        log::info!("Output Exit: {:?}", output);
        let start_time = Instant::now();
        let output = prob.chance_reveal_redraw_norm(0, vec!["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD","BE", "CC", "CD", "CE", "DD", "DE", "EE"].into_iter().map(|s| s.to_string()).collect());
        let elapsed_time = start_time.elapsed();
        log::info!("Time Norm: {:?}", elapsed_time);
        log::info!("Output Norm: {:?}", output);
        prob.reset();
    }
}
pub fn test_reach() {
    let mut rp: ReachProbability = ReachProbability::initialise();
    let vec0: Vec<String> = vec!["AA", "BE"].into_iter().map(|s| s.to_string()).collect();
    let vec1: Vec<String> = vec!["AA", "AB", "AC", "AD", "AE"].into_iter().map(|s| s.to_string()).collect();
    let vec2: Vec<String> = vec!["CC"].into_iter().map(|s| s.to_string()).collect();
    let vec3: Vec<String> = vec!["AD", "BD", "CD", "DD", "DE"].into_iter().map(|s| s.to_string()).collect();
    let vec4: Vec<String> = vec!["AB", "AC", "AD", "AE", "AA"].into_iter().map(|s| s.to_string()).collect();
    let vec5: Vec<String> = vec!["AE", "BE", "CE", "DE", "EE"].into_iter().map(|s| s.to_string()).collect();
    let hash_set0: HashSet<String> = vec!["AA", "BE"].into_iter().map(|s| s.to_string()).collect();
    let hash_set1: HashSet<String> = vec!["AA", "AB", "AC", "AD", "AE"].into_iter().map(|s| s.to_string()).collect();
    let hash_set2: HashSet<String> = vec!["CC"].into_iter().map(|s| s.to_string()).collect();
    let hash_set3: HashSet<String> = vec!["AD", "BD", "CD", "DD", "DE"].into_iter().map(|s| s.to_string()).collect();
    let hash_set4: HashSet<String> = vec!["AB", "AC", "AD", "AE", "AA"].into_iter().map(|s| s.to_string()).collect();
    let hash_set5: HashSet<String> = vec!["AE", "BE", "CE", "DE", "EE"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set0: HashSet<String> = vec!["AA", "AC", "AE", "BB", "BD", "CC", "CD", "CE", "DD"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set1: HashSet<String> = vec!["AB", "AC", "AD", "BC", "BD", "CC", "CD", "CE", "DD", "EE"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set2: HashSet<String> = vec!["AA", "AC", "AE", "BD", "BE", "CC", "CD", "CE", "DD", "DE"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set3: HashSet<String> = vec!["AB", "AB", "AE", "BB", "BD", "CC", "CD", "DD", "EE"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set4: HashSet<String> = vec!["AA", "AD", "AE", "BC", "BE", "CD", "CE", "DD", "DE"].into_iter().map(|s| s.to_string()).collect();
    // let hash_set5: HashSet<String> = vec!["AB", "AD", "AE", "BB", "BE", "CC", "CD", "CE", "DD", "EE"].into_iter().map(|s| s.to_string()).collect();
    rp.modify_player_set(0, hash_set0);
    rp.modify_player_set(1, hash_set1);
    rp.modify_player_set(2, hash_set2);
    rp.modify_player_set(3, hash_set3);
    rp.modify_player_set(4, hash_set4);
    rp.modify_player_set(5, hash_set5);
    rp.modify_player_vec(0, vec0);
    rp.modify_player_vec(1, vec1);
    rp.modify_player_vec(2, vec2);
    rp.modify_player_vec(3, vec3);
    rp.modify_player_vec(4, vec4);
    rp.modify_player_vec(5, vec5);
    let hand: String = "BE".to_string();
    let start_time = Instant::now();
    let output: bool = rp.info_state_prune(0, hand.clone());
    let elapsed_time = start_time.elapsed();
    println!("time: {:?}", elapsed_time);
    println!("output: {}", output);
    let start_time = Instant::now();
    let output: bool = rp.info_state_prune_vec(0, hand.clone());
    let elapsed_time = start_time.elapsed();
    println!("time: {:?}", elapsed_time);
    println!("output: {}", output);
}
// pub fn test_belief(iterations: usize){
//     logger();

//     let mut hh = History::new(0);
//     let mut step: usize = 0;
//     let mut new_moves: Vec<ActionObservation>;
    
//     // Just initial part
//     // Mid game can be like microseconds
    
//     log::trace!("Start");
//     new_moves = hh.generate_legal_moves();
//     let mut prob = NaiveProb::new();
    
//     if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
//         hh.push_ao(output);
//         prob.push_ao(&output);
//     } else {
//         log::trace!("Pushed bad move!");
//     }
//     let hist_vec: Vec<ActionObservation> = hh.get_history(0);
//     for i in 0..iterations {
//         let start_time = Instant::now();
//         let elapsed_time_concurrent = start_time.elapsed();
//         prob.filter_state_concurrent();
//         log::info!("Time taken for filter_state_concurrent: {:?}", elapsed_time_concurrent);
//         let start_time_2 = Instant::now();
//         let output: Vec<f64> = prob.get_latest_beliefs();
//         let output: Vec<f64> = prob.get_latest_beliefs_concurrent();
//         let elapsed_time = start_time_2.elapsed();
//         log::info!("Time taken for belief: {:?}", elapsed_time);
//         if i % 10 == 0 {
//             println!("Done with {}", i);
//         }
//     }
// }
// pub fn test_filter(iterations: usize){
//     logger();

//     let mut prob = NaiveProb::new();
//     // println!("Initialising Sets");
//     // let start_time = Instant::now();
//     // prob.set_generation();
//     // let elapsed_time = start_time.elapsed();
//     // println!("Finished Initialising Sets");
//     // println!("Total Time to Initialise Sets: {:?}", elapsed_time);
//     // log::info!("Total Time taken to Initialise Sets: {:?}", elapsed_time);
//     for i in 0..iterations {

//         let mut hh = History::new(0);
//         let mut step: usize = 0;
//         let mut new_moves: Vec<ActionObservation>;
//         let mut rng = thread_rng();
//         let top: usize = rng.gen_range(50..200);
//         let limit = rng.gen_range(0..10);
//         // let limit: usize = 200;
//         while !hh.game_won() {
            
//             new_moves = hh.generate_legal_moves();
            
//             if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
//                 hh.push_ao(output);
//                 prob.push_ao(&output);
//             } else {
//                 break;
//             }
//             step += 1;
//             if step > limit {
//                 break;
//             }
//         }
//         hh.log_state();
//         log::info!("{}", format!("Game Final Step : {:?}",step));
//         prob.printlog();
//         let start_time = Instant::now();
//         prob.filter_state();
//         let elapsed_time = start_time.elapsed();
//         prob.log_calc_state_len();
//         log::info!("Total Time taken for filter_state: {:?}", elapsed_time);
//         let start_time_concurrent = Instant::now();
//         prob.filter_state_concurrent();
//         let elapsed_time_concurrent = start_time_concurrent.elapsed();
//         prob.log_calc_state_len();
//         log::info!("Total Time taken for filter_state_concurrent: {:?}", elapsed_time_concurrent);
//         let start_time = Instant::now();
//         prob.filter_state_optimal();
//         let elapsed_time = start_time.elapsed();
//         log::info!("Total Time taken for filter_state_optimal: {:?}", elapsed_time);
//         let start_time = Instant::now();
//         prob.filter_state_optimal2();
//         let elapsed_time = start_time.elapsed();
//         log::info!("Total Time taken for filter_state_optimal2: {:?}", elapsed_time);
//         let start_time = Instant::now();
//         let output: Vec<f64> = prob.compute_beliefs_direct();
//         let elapsed_time = start_time.elapsed();
//         log::info!("Belief Prob: {:?}", output);
//         log::info!("Total Time taken for compute_belief_direct: {:?}", elapsed_time);
//         prob.log_calc_state_len();

//         let start_time_2 = Instant::now();
//         let output: Vec<f64> = prob.get_latest_beliefs();
//         let elapsed_time_belief = start_time_2.elapsed();
//         log::info!("Belief Prob: {:?}", output);
//         log::info!("Time taken for belief: {:?}", elapsed_time_belief);
//         let start_time_2 = Instant::now();
//         let output: Vec<f64> = prob.get_latest_beliefs_concurrent();
//         let elapsed_time_belief = start_time_2.elapsed();
//         log::info!("Belief Prob: {:?}", output);
//         log::info!("Time taken for conc belief: {:?}", elapsed_time_belief);
//         let start_time_2 = Instant::now();
//         let key: String = prob.make_key_belief();
//         let elapsed_time_belief = start_time_2.elapsed();
//         log::info!("Key: {:?}", key);
//         log::info!("Time taken to generate key: {:?}", elapsed_time_belief);
//         if i % 10 == 0 {
//             println!("Done with {}", i);
//         }
//         log::info!("");
//         prob.reset();
//     }
// }

// pub fn make_belief(iterations: usize){
//     // logger();

//     let mut prob = NaiveProb::new();
//     // println!("Initialising Sets");
//     // let start_time = Instant::now();
//     // prob.set_generation();
//     // let elapsed_time = start_time.elapsed();
//     // println!("Finished Initialising Sets");
//     // println!("Total Time to Initialise Sets: {:?}", elapsed_time);
//     // log::info!("Total Time taken to Initialise Sets: {:?}", elapsed_time);
//     prob.load_bson_hashmap();
//     println!("Load Success");
//     for i in 0..iterations {

//         let mut hh = History::new(0);
//         let mut step: usize = 0;
//         let mut new_moves: Vec<ActionObservation>;
//         let mut rng = thread_rng();
//         // let top: usize = rng.gen_range(50..200);
//         let limit = rng.gen_range(0..150);
//         while !hh.game_won() {
            
//             new_moves = hh.generate_legal_moves();
            
//             if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
//                 hh.push_ao(output);
//                 prob.push_ao(&output);
//             } else {
//                 break;
//             }
//             step += 1;
//             if step > limit {
//                 break;
//             }
//         }
//         hh.log_state();
//         log::info!("{}", format!("Game Final Step : {:?}",step));
//         prob.printlog();
//         if i % 10000 == 0 {
//             println!("Done with {}", i);
//             prob.save_bson_hashmap();
//             println!("Saved Bson!");
//             prob.print_belief_hm_len();
//         }
//         if prob.key_in_bson_hashmap(prob.make_key_belief()) {
//             continue;
//         }
//         prob.gen_and_save_belief();

//         log::info!("");
//         prob.reset();
//     }
// }

pub fn game_rnd(game_no: usize, log_bool: bool){
    if log_bool{
        logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut total_steps: usize = 0;
    let mut prob = NaiveProb::new();
    
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        if game % 1000000 == 0 {
            println!("Game: {}", game);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            log::trace!("Game Made:");
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();

            // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
            // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
            new_moves = hh.generate_legal_moves();
            if new_moves[0].name() != AOName::CollectiveChallenge {
    
                log::info!("{}", format!("Legal Moves: {:?}", new_moves));
            } else {
                log::info!("{}", format!("Legal Moves: {:?}", new_moves));
                // log::info!("{}", format!("Legal Moves: CollectiveChallenge"));
            }
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                log::info!("{}", format!("Choice: {:?}", output));
                hh.push_ao(output);
                prob.push_ao(&output);
                let start_time = Instant::now();
                let output: Option<String> = prob.chance_sample_exit();
                let elapsed_time = start_time.elapsed();
                println!("Test Time: {:?}", elapsed_time);
            } else {
                log::trace!("Pushed bad move!");
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
        total_steps += step;
        log::info!("{}", format!("Game Won : {:?}",step));
        hh.log_state();
        log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        log::info!("");
        prob.reset();
        game += 1;
    }

    log::info!("Most Steps: {}", max_steps);
    println!("Total Moves Calculated: {}", total_steps);
    println!("Most Steps: {}", max_steps);
}
pub fn game_rnd_constraint(game_no: usize, log_bool: bool){
    if log_bool{
        logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut total_wrong_legal: usize = 0;
    let mut total_wrong_illegal: usize = 0;
    let mut total_wrong_legal_discard_1: usize = 0;
    let mut total_wrong_illegal_discard_1: usize = 0;
    let mut total_wrong_legal_discard_2: usize = 0;
    let mut total_wrong_illegal_discard_2: usize = 0;
    let mut total_wrong_legal_reveal_redraw: usize = 0;
    let mut total_wrong_illegal_reveal_redraw: usize = 0;
    let mut total_wrong_legal_exchangedraw: usize = 0;
    let mut total_wrong_illegal_exchangedraw: usize = 0;
    let mut total_legal_discard_1: usize = 0;
    let mut total_illegal_discard_1: usize = 0;
    let mut total_legal_discard_2: usize = 0;
    let mut total_illegal_discard_2: usize = 0;
    let mut total_legal_reveal_redraw: usize = 0;
    let mut total_illegal_reveal_redraw: usize = 0;
    let mut total_legal_exchangedraw: usize = 0;
    let mut total_illegal_exchangedraw: usize = 0;
    let mut total_wrong_same_cards_exchangedraw: usize = 0;
    let mut total_already_illegal: usize = 0;
    let mut total_wrong_legal_proper: usize = 0;
    let mut total_wrong_illegal_proper: usize = 0;
    let mut total_same: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % (5000) == 0 {
            println!("Game: {}", game);
            println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
            println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
            println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
            println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
            println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
            println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
            println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
            println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
            println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
            println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
            println!("Total Tries: {}", total_tries);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            log::trace!("Game Made:");
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            prob.printlog();
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
                    if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.cards()[0]);
                        // let elapsed_time = start_time.elapsed();
                        // println!("Time: {:?}", elapsed_time);
                        // let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.cards()[0]);
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            log::trace!("Set: Legal Move");
                            total_legal_discard_1 += 1;
                        } else {
                            log::trace!("Set: Illegal Move");
                            total_illegal_discard_1 += 1;
                        }
                        if legality.is_none(){
                            log::trace!("Actual: Illegal Move");
                            if set_legality {
                                log::trace!("Verdict: Legal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            log::trace!("Actual: Legal Move");
                            if !set_legality {
                                log::trace!("Verdict: Illegal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    } else {
                        let set_legality: bool = prob.player_can_have_cards(output.player_id(), output.cards());
                        // let legality: Option<String> = prob.can_player_have_cards(output.player_id(), output.cards());
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[1]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            log::trace!("Set: Legal Move");
                            total_legal_discard_2 += 1;
                        } else {
                            log::trace!("Set: Illegal Move");
                            total_illegal_discard_2 += 1;
                        }
                        if legality.is_none(){
                            log::trace!("Actual: Illegal Move");
                            if set_legality {
                                log::trace!("Verdict: Legal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            log::trace!("Actual: Legal Move");
                            if !set_legality {
                                log::trace!("Verdict: Illegal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    }
                } else if output.name() == AOName::RevealRedraw {
                    let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.card());
                    if set_legality{
                        log::trace!("Set: Legal Move");
                        total_legal_reveal_redraw += 1;
                    } else {
                        log::trace!("Set: Illegal Move");
                        total_illegal_reveal_redraw += 1;
                    }
                    // let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.card());
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    latest_constraint.add_raw_public_constraint(output.player_id(), output.card());
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    // prob.filter_player_can_have_card(output.player_id(), &output.card());
                    // let proper: usize = prob.calc_state_len();
                    // if proper == 0 {
                    //     log::trace!("Actual Proper: Illegal Move");
                    //     if set_legality {
                    //         log::trace!("Verdict Proper: Legal Wrong");
                    //         total_wrong_legal_proper += 1;
                    //         prob.log_calc_state();
                    //         prob.log_calc_state_len();
                    //         hh.log_state();
                    //     }
                    // } else {
                    //     log::trace!("Actual Proper: Legal Move");
                    //     if !set_legality {
                    //         log::trace!("Verdict Proper: Illegal Wrong");
                    //         total_wrong_illegal_proper += 1;
                    //         prob.log_calc_state();
                    //         prob.log_calc_state_len();
                    //         hh.log_state();
                    //     }
                    // }
                    if legality.is_none(){
                        log::trace!("Actual: Illegal Move");
                        if set_legality {
                            log::trace!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        log::trace!("Actual: Legal Move");
                        if !set_legality {
                            log::trace!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                            log::trace!("Pringing another log for reproducibility");
                            prob.printlog();
                        }
                    }
                    // if proper == 0 && legality.is_none(){
                    //     total_same += 1;
                    // } else if proper > 0 && !legality.is_none() {
                    //     total_same += 1;
                    // }
                    total_tries += 1;
                    if !set_legality || legality.is_none(){
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    let set_legality: bool = prob.player_can_have_cards(6, output.cards());
                    // let legality: Option<String> = prob.can_player_have_cards(6, output.cards());
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    if output.cards()[0] == output.cards()[1] {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 2));
                    } else {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 1));
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[1], 0, 1));
                    }
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    if set_legality{
                        log::trace!("Set: Legal Move");
                        total_legal_exchangedraw += 1;
                    } else {
                        log::trace!("Set: Illegal Move");
                        total_illegal_exchangedraw += 1;
                    }
                    if legality.is_none(){
                        log::trace!("Actual: Illegal Move");
                        if set_legality {
                            log::trace!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        log::trace!("Actual: Legal Move");
                        if !set_legality {
                            log::trace!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    }
                    total_tries += 1;
                    if !set_legality || legality.is_none() {
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else {
                    hh.push_ao(output);
                    prob.push_ao(&output);
                }
            } else {
                log::trace!("Pushed bad move!");
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
        game += 1;
    }
    log::info!("Most Steps: {}", max_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
    println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
    println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
    println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
    println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
    println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
    println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
    println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
    println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
    println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
    println!("Total Tries: {}", total_tries);
}
pub fn test_par_constructor(game_no: usize, log_bool: bool){
    if log_bool{
        logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut total_wrong_legal: usize = 0;
    let mut total_wrong_illegal: usize = 0;
    let mut total_wrong_legal_discard_1: usize = 0;
    let mut total_wrong_illegal_discard_1: usize = 0;
    let mut total_wrong_legal_discard_2: usize = 0;
    let mut total_wrong_illegal_discard_2: usize = 0;
    let mut total_wrong_legal_reveal_redraw: usize = 0;
    let mut total_wrong_illegal_reveal_redraw: usize = 0;
    let mut total_wrong_legal_exchangedraw: usize = 0;
    let mut total_wrong_illegal_exchangedraw: usize = 0;
    let mut total_legal_discard_1: usize = 0;
    let mut total_illegal_discard_1: usize = 0;
    let mut total_legal_discard_2: usize = 0;
    let mut total_illegal_discard_2: usize = 0;
    let mut total_legal_reveal_redraw: usize = 0;
    let mut total_illegal_reveal_redraw: usize = 0;
    let mut total_legal_exchangedraw: usize = 0;
    let mut total_illegal_exchangedraw: usize = 0;
    let mut total_wrong_same_cards_exchangedraw: usize = 0;
    let mut total_already_illegal: usize = 0;
    let mut total_wrong_legal_proper: usize = 0;
    let mut total_wrong_illegal_proper: usize = 0;
    let mut total_same: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % (100) == 0 {
            println!("Game: {}", game);
            println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
            println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
            println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
            println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
            println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
            println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
            println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
            println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
            println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
            println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
            println!("Total Tries: {}", total_tries);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            log::trace!("Game Made:");
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            prob.printlog();
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
                    if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        let set_legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        // let elapsed_time = start_time.elapsed();
                        // println!("Time: {:?}", elapsed_time);
                        let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.cards()[0]);
                        if !set_legality.is_none(){
                            log::trace!("Set: Legal Move");
                            total_legal_discard_1 += 1;
                        } else {
                            log::trace!("Set: Illegal Move");
                            total_illegal_discard_1 += 1;
                        }
                        if legality.is_none(){
                            log::trace!("Actual: Illegal Move");
                            if !set_legality.is_none() {
                                log::trace!("Verdict: Legal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            log::trace!("Actual: Legal Move");
                            if set_legality.is_none() {
                                log::trace!("Verdict: Illegal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if set_legality.is_none() || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    } else {
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[1]);
                        let set_legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        let legality: Option<String> = prob.can_player_have_cards(output.player_id(), output.cards());
                        if !set_legality.is_none(){
                            log::trace!("Set: Legal Move");
                            total_legal_discard_2 += 1;
                        } else {
                            log::trace!("Set: Illegal Move");
                            total_illegal_discard_2 += 1;
                        }
                        if legality.is_none(){
                            log::trace!("Actual: Illegal Move");
                            if !set_legality.is_none() {
                                log::trace!("Verdict: Legal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            log::trace!("Actual: Legal Move");
                            if set_legality.is_none() {
                                log::trace!("Verdict: Illegal Wrong");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if set_legality.is_none() || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    }
                } else if output.name() == AOName::RevealRedraw {
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    latest_constraint.add_raw_public_constraint(output.player_id(), output.card());
                    let set_legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    if !set_legality.is_none(){
                        log::trace!("Set: Legal Move");
                        total_legal_reveal_redraw += 1;
                    } else {
                        log::trace!("Set: Illegal Move");
                        total_illegal_reveal_redraw += 1;
                    }
                    let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.card());
                    // prob.filter_player_can_have_card(output.player_id(), &output.card());
                    // let proper: usize = prob.calc_state_len();
                    // if proper == 0 {
                    //     log::trace!("Actual Proper: Illegal Move");
                    //     if set_legality {
                    //         log::trace!("Verdict Proper: Legal Wrong");
                    //         total_wrong_legal_proper += 1;
                    //         prob.log_calc_state();
                    //         prob.log_calc_state_len();
                    //         hh.log_state();
                    //     }
                    // } else {
                    //     log::trace!("Actual Proper: Legal Move");
                    //     if !set_legality {
                    //         log::trace!("Verdict Proper: Illegal Wrong");
                    //         total_wrong_illegal_proper += 1;
                    //         prob.log_calc_state();
                    //         prob.log_calc_state_len();
                    //         hh.log_state();
                    //     }
                    // }
                    if legality.is_none(){
                        log::trace!("Actual: Illegal Move");
                        if !set_legality.is_none() {
                            log::trace!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        log::trace!("Actual: Legal Move");
                        if set_legality.is_none() {
                            log::trace!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                            log::trace!("Pringing another log for reproducibility");
                            prob.printlog();
                        }
                    }
                    // if proper == 0 && legality.is_none(){
                    //     total_same += 1;
                    // } else if proper > 0 && !legality.is_none() {
                    //     total_same += 1;
                    // }
                    total_tries += 1;
                    if set_legality.is_none() || legality.is_none(){
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    let mut dummy_arr_1: [u8; 7] = [0, 0, 0, 0, 0, 0, 1];
                    let mut dummy_arr_2: [u8; 7] = [0, 0, 0, 0, 0, 0, 1];
                    if output.cards()[0] == output.cards()[1]{
                        latest_constraint.add_raw_group(GroupConstraint::new_list(dummy_arr_1, output.cards()[0], 0, 2));
                    } else {
                        latest_constraint.add_raw_group(GroupConstraint::new_list(dummy_arr_1, output.cards()[0], 0, 1));
                        latest_constraint.add_raw_group(GroupConstraint::new_list(dummy_arr_2, output.cards()[1], 0, 1));
                    }
                    let set_legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    let legality: Option<String> = prob.can_player_have_cards(6, output.cards());
                    if !set_legality.is_none(){
                        log::trace!("Set: Legal Move");
                        total_legal_exchangedraw += 1;
                    } else {
                        log::trace!("Set: Illegal Move");
                        total_illegal_exchangedraw += 1;
                    }
                    if legality.is_none(){
                        log::trace!("Actual: Illegal Move");
                        if !set_legality.is_none() {
                            log::trace!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        log::trace!("Actual: Legal Move");
                        if set_legality.is_none() {
                            log::trace!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    }
                    total_tries += 1;
                    if set_legality.is_none() || legality.is_none() {
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else {
                    hh.push_ao(output);
                    prob.push_ao(&output);
                }
            } else {
                log::trace!("Pushed bad move!");
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
        game += 1;
    }
    log::info!("Most Steps: {}", max_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
    println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
    println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
    println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
    println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
    println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
    println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
    println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
    println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
    println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
    println!("Total Tries: {}", total_tries);
}
pub fn error_farmer(game_no: usize, log_bool: bool){
    if log_bool{
        logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut total_wrong_legal: usize = 0;
    let mut total_wrong_illegal: usize = 0;
    let mut total_wrong_legal_discard_1: usize = 0;
    let mut total_wrong_illegal_discard_1: usize = 0;
    let mut total_wrong_legal_discard_2: usize = 0;
    let mut total_wrong_illegal_discard_2: usize = 0;
    let mut total_wrong_legal_reveal_redraw: usize = 0;
    let mut total_wrong_illegal_reveal_redraw: usize = 0;
    let mut total_wrong_legal_exchangedraw: usize = 0;
    let mut total_wrong_illegal_exchangedraw: usize = 0;
    let mut total_legal_discard_1: usize = 0;
    let mut total_illegal_discard_1: usize = 0;
    let mut total_legal_discard_2: usize = 0;
    let mut total_illegal_discard_2: usize = 0;
    let mut total_legal_reveal_redraw: usize = 0;
    let mut total_illegal_reveal_redraw: usize = 0;
    let mut total_legal_exchangedraw: usize = 0;
    let mut total_illegal_exchangedraw: usize = 0;
    let mut total_wrong_same_cards_exchangedraw: usize = 0;
    let mut total_already_illegal: usize = 0;
    let mut total_wrong_legal_proper: usize = 0;
    let mut total_wrong_illegal_proper: usize = 0;
    let mut total_same: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        // log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % (5000) == 0 {
            println!("Game: {}", game);
            println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
            println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
            println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
            println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
            println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
            println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
            println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
            println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
            println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
            println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
            println!("Total Tries: {}", total_tries);
            println!("Gone Fishing...");
        }

        while !hh.game_won() {

            new_moves = hh.generate_legal_moves();
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    if output.no_cards() == 1 {
                        let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.cards()[0]);
                        // let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.cards()[0]);
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            // log::trace!("Set: Legal Move");
                            total_legal_discard_1 += 1;
                        } else {
                            // log::trace!("Set: Illegal Move");
                            total_illegal_discard_1 += 1;
                        }
                        if legality.is_none(){
                            // log::trace!("Actual: Illegal Move");
                            if set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Legal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            // log::trace!("Actual: Legal Move");
                            if !set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Illegal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    } else {
                        let set_legality: bool = prob.player_can_have_cards(output.player_id(), output.cards());
                        // let legality: Option<String> = prob.can_player_have_cards(output.player_id(), output.cards());
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[1]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            // log::trace!("Set: Legal Move");
                            total_legal_discard_2 += 1;
                        } else {
                            // log::trace!("Set: Illegal Move");
                            total_illegal_discard_2 += 1;
                        }
                        if legality.is_none(){
                            // log::trace!("Actual: Illegal Move");
                            if set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Legal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            // log::trace!("Actual: Legal Move");
                            if !set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Illegal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    }
                } else if output.name() == AOName::RevealRedraw {
                    let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.card());
                    if set_legality{
                        // log::trace!("Set: Legal Move");
                        total_legal_reveal_redraw += 1;
                    } else {
                        // log::trace!("Set: Illegal Move");
                        total_illegal_reveal_redraw += 1;
                    }
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    latest_constraint.add_raw_public_constraint(output.player_id(), output.card());
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);

                    if legality.is_none(){
                        // log::trace!("Actual: Illegal Move");
                        if set_legality {
                            prob.printlog();
                            log::info!("{}", format!("Choice: {:?}", output));
                            log::info!("Verdict: Legal Wrong");
                            log::info!("");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        // log::trace!("Actual: Legal Move");
                        if !set_legality {
                            prob.printlog();
                            log::info!("{}", format!("Choice: {:?}", output));
                            log::info!("Verdict: Illegal Wrong");
                            log::info!("");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                            // log::trace!("Pringing another log for reproducibility");
                            // prob.printlog();
                        }
                    }
                    total_tries += 1;
                    if !set_legality || legality.is_none(){
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    let set_legality: bool = prob.player_can_have_cards(6, output.cards());
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    if output.cards()[0] == output.cards()[1] {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 2));
                    } else {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 1));
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[1], 0, 1));
                    }
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    if set_legality{
                        // log::trace!("Set: Legal Move");
                        total_legal_exchangedraw += 1;
                    } else {
                        // log::trace!("Set: Illegal Move");
                        total_illegal_exchangedraw += 1;
                    }
                    if legality.is_none(){
                        // log::trace!("Actual: Illegal Move");
                        if set_legality {
                            // prob.printlog();
                            // log::info!("{}", format!("Choice: {:?}", output));
                            // log::info!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                // prob.log_calc_state();
                            }
                        }
                    } else {
                        // log::trace!("Actual: Legal Move");
                        if !set_legality {
                            // prob.printlog();
                            // log::info!("{}", format!("Choice: {:?}", output));
                            // log::info!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                // prob.log_calc_state();
                            }
                        }
                    }
                    total_tries += 1;
                    if !set_legality || legality.is_none() {
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else {
                    hh.push_ao(output);
                    prob.push_ao(&output);
                }
            } else {
                // log::trace!("Pushed bad move!");
                break;
            }
            step += 1;
            if step > 1000 {
                break;
            }
            // log::info!("");
        }
        if step > max_steps {
            max_steps = step;
        }
        // log::info!("{}", format!("Game Won : {:?}",step));
        // hh.log_state();
        // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        // log::info!("");
        prob.reset();
        game += 1;

    }
    log::info!("Most Steps: {}", max_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
    println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
    println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
    println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
    println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
    println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
    println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
    println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
    println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
    println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
    println!("Total Tries: {}", total_tries);
}
pub fn overflow_farmer(game_no: usize, log_bool: bool){
    if log_bool{
        // logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let mut total_wrong_legal: usize = 0;
    let mut total_wrong_illegal: usize = 0;
    let mut total_wrong_legal_discard_1: usize = 0;
    let mut total_wrong_illegal_discard_1: usize = 0;
    let mut total_wrong_legal_discard_2: usize = 0;
    let mut total_wrong_illegal_discard_2: usize = 0;
    let mut total_wrong_legal_reveal_redraw: usize = 0;
    let mut total_wrong_illegal_reveal_redraw: usize = 0;
    let mut total_wrong_legal_exchangedraw: usize = 0;
    let mut total_wrong_illegal_exchangedraw: usize = 0;
    let mut total_legal_discard_1: usize = 0;
    let mut total_illegal_discard_1: usize = 0;
    let mut total_legal_discard_2: usize = 0;
    let mut total_illegal_discard_2: usize = 0;
    let mut total_legal_reveal_redraw: usize = 0;
    let mut total_illegal_reveal_redraw: usize = 0;
    let mut total_legal_exchangedraw: usize = 0;
    let mut total_illegal_exchangedraw: usize = 0;
    let mut total_wrong_same_cards_exchangedraw: usize = 0;
    let mut total_already_illegal: usize = 0;
    let mut total_wrong_legal_proper: usize = 0;
    let mut total_wrong_illegal_proper: usize = 0;
    let mut total_same: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        // log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % (5000) == 0 {
            println!("Game: {}", game);
            println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
            println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
            println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
            println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
            println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
            println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
            println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
            println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
            println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
            println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
            println!("Total Tries: {}", total_tries);
            println!("Gone Fishing...");
        }

        while !hh.game_won() {

            new_moves = hh.generate_legal_moves();
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                prob.printlog();
                if output.name() == AOName::Discard{
                    log::info!("{}", format!("Choice: {:?}", output));
                    if output.no_cards() == 1 {
                        let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.cards()[0]);
                        // let legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.cards()[0]);
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            // log::trace!("Set: Legal Move");
                            total_legal_discard_1 += 1;
                        } else {
                            // log::trace!("Set: Illegal Move");
                            total_illegal_discard_1 += 1;
                        }
                        if legality.is_none(){
                            // log::trace!("Actual: Illegal Move");
                            if set_legality {
                                prob.printlog();
                                log::info!("Verdict: Legal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            // log::trace!("Actual: Legal Move");
                            if !set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Illegal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_1 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    } else {
                        let set_legality: bool = prob.player_can_have_cards(output.player_id(), output.cards());
                        // let legality: Option<String> = prob.can_player_have_cards(output.player_id(), output.cards());
                        let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[0]);
                        latest_constraint.add_raw_public_constraint(output.player_id(), output.cards()[1]);
                        let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                        if set_legality{
                            // log::trace!("Set: Legal Move");
                            total_legal_discard_2 += 1;
                        } else {
                            // log::trace!("Set: Illegal Move");
                            total_illegal_discard_2 += 1;
                        }
                        if legality.is_none(){
                            // log::trace!("Actual: Illegal Move");
                            if set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Legal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_legal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        } else {
                            // log::trace!("Actual: Legal Move");
                            if !set_legality {
                                prob.printlog();
                                log::info!("{}", format!("Choice: {:?}", output));
                                log::info!("Verdict: Illegal Wrong");
                                log::info!("");
                                prob.filter_state_simple();
                                if prob.calc_state_len() == 0 {
                                    total_already_illegal += 1;
                                } else {
                                    total_wrong_illegal_discard_2 += 1;
                                }
                                if log_bool {
                                    prob.log_calc_state();
                                }
                            }
                        }
                        total_tries += 1;
                        if !set_legality || legality.is_none(){
                            break    
                        } else {
                            hh.push_ao(output);
                            prob.push_ao(&output);
                        }
                    }
                } else if output.name() == AOName::RevealRedraw {
                    log::info!("{}", format!("Choice: {:?}", output));
                    let set_legality: bool = prob.player_can_have_card(output.player_id(), &output.card());
                    if set_legality{
                        // log::trace!("Set: Legal Move");
                        total_legal_reveal_redraw += 1;
                    } else {
                        // log::trace!("Set: Illegal Move");
                        total_illegal_reveal_redraw += 1;
                    }
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    latest_constraint.add_raw_public_constraint(output.player_id(), output.card());
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);

                    if legality.is_none(){
                        // log::trace!("Actual: Illegal Move");
                        if set_legality {
                            prob.printlog();
                            log::info!("{}", format!("Choice: {:?}", output));
                            log::info!("Verdict: Legal Wrong");
                            log::info!("");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                        }
                    } else {
                        // log::trace!("Actual: Legal Move");
                        if !set_legality {
                            prob.printlog();
                            log::info!("{}", format!("Choice: {:?}", output));
                            log::info!("Verdict: Illegal Wrong");
                            log::info!("");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_reveal_redraw += 1;
                            }
                            if log_bool {
                                prob.log_calc_state();
                            }
                            // log::trace!("Pringing another log for reproducibility");
                            // prob.printlog();
                        }
                    }
                    total_tries += 1;
                    if !set_legality || legality.is_none(){
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else if output.name() == AOName::ExchangeDraw {
                    log::info!("{}", format!("Choice: {:?}", output));
                    let set_legality: bool = prob.player_can_have_cards(6, output.cards());
                    let mut latest_constraint: CollectiveConstraint = prob.latest_constraint();
                    if output.cards()[0] == output.cards()[1] {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 2));
                    } else {
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[0], 0, 1));
                        latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], output.cards()[1], 0, 1));
                    }
                    let legality: Option<String> = latest_constraint.par_constructor(&latest_constraint);
                    if set_legality{
                        // log::trace!("Set: Legal Move");
                        total_legal_exchangedraw += 1;
                    } else {
                        // log::trace!("Set: Illegal Move");
                        total_illegal_exchangedraw += 1;
                    }
                    if legality.is_none(){
                        // log::trace!("Actual: Illegal Move");
                        if set_legality {
                            // prob.printlog();
                            // log::info!("{}", format!("Choice: {:?}", output));
                            // log::info!("Verdict: Legal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_legal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                // prob.log_calc_state();
                            }
                        }
                    } else {
                        // log::trace!("Actual: Legal Move");
                        if !set_legality {
                            // prob.printlog();
                            // log::info!("{}", format!("Choice: {:?}", output));
                            // log::info!("Verdict: Illegal Wrong");
                            prob.filter_state_simple();
                            if prob.calc_state_len() == 0 {
                                total_already_illegal += 1;
                            } else {
                                total_wrong_illegal_exchangedraw += 1;
                                if output.cards()[0] == output.cards()[1] {
                                    total_wrong_same_cards_exchangedraw += 1;
                                }
                            }
                            if log_bool {
                                // prob.log_calc_state();
                            }
                        }
                    }
                    total_tries += 1;
                    if !set_legality || legality.is_none() {
                        break    
                    } else {
                        hh.push_ao(output);
                        prob.push_ao(&output);
                    }
                } else {
                    hh.push_ao(output);
                    prob.push_ao(&output);
                }
            } else {
                // log::trace!("Pushed bad move!");
                break;
            }
            step += 1;
            if step > 1000 {
                break;
            }
            // log::info!("");
        }
        if step > max_steps {
            max_steps = step;
        }
        // log::info!("{}", format!("Game Won : {:?}",step));
        // hh.log_state();
        // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        // log::info!("");
        prob.reset();
        game += 1;
        if game % 1000 == 1 {
            let smth = clear_logs();
        }
    }
    log::info!("Most Steps: {}", max_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total already illegal Wrong: {}/{}", total_already_illegal, total_tries);
    println!("Total (Discard 1) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_1, total_legal_discard_1);
    println!("Total (Discard 1) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_1, total_illegal_discard_1);
    println!("Total (Discard 2) Legal Predictions Wrong: {}/{}", total_wrong_legal_discard_2, total_legal_discard_2);
    println!("Total (Discard 2) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_discard_2, total_illegal_discard_2);
    println!("Total (RevealRedraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_reveal_redraw, total_legal_reveal_redraw);
    println!("Total (RevealRedraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_reveal_redraw, total_illegal_reveal_redraw);
    println!("Total (ExchangeDraw) Legal Predictions Wrong: {}/{}", total_wrong_legal_exchangedraw, total_legal_exchangedraw);
    println!("Total (ExchangeDraw) Illegal Predictions Wrong: {}/{}", total_wrong_illegal_exchangedraw, total_illegal_exchangedraw);
    println!("Total Same Card Exchange Draw Wrong: {}", total_wrong_same_cards_exchangedraw);
    println!("Total Tries: {}", total_tries);
}

pub fn test_impossible_state(game_no: usize, log_bool: bool){
    if log_bool{
        logger();
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut total_steps: usize = 0;
    let mut prob = NaiveProb::new();
    let start_time = Instant::now();
    while game < game_no {
        log::info!("Game : {}", game);
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        if game % 200 == 0 {
            println!("Game: {}", game);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            log::trace!("Game Made:");
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            prob.printlog();
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
                if !prob.latest_constraint_is_empty() {
                    if output.name() == AOName::Discard {
                        if output.no_cards() == 1 {
                            let chosen_move_legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.cards()[0]);
                            if chosen_move_legality.is_none() {
                                log::trace!("Cant choose this move!");
                                break;
                            }
                        } else {
                            let chosen_move_legality: Option<String> = prob.can_player_have_cards(output.player_id(), &output.cards());
                            if chosen_move_legality.is_none() {
                                log::trace!("Cant choose this move!");
                                break;
                            }
                        }
                    } else if output.name() == AOName::RevealRedraw {
                        let chosen_move_legality: Option<String> = prob.can_player_have_card(output.player_id(), &output.card());
                        if chosen_move_legality.is_none() {
                            log::trace!("Cant choose this move!");
                            break;
                        }
                    } else if output.name() == AOName::ExchangeDraw {
                        let chosen_move_legality: Option<String> = prob.can_player_have_cards(output.player_id(), &output.cards());
                        if chosen_move_legality.is_none() {
                            log::trace!("Cant choose this move!");
                            break;
                        }
                    }
                }
                hh.push_ao(output);
                prob.push_ao(&output);
                if !prob.latest_constraint_is_empty(){
                    let legality: Option<String> = prob.chance_sample_exit();
                    if legality.is_none(){
                        log::trace!("New State Now Illegal!: ");
                        prob.printlog();
                        break;
                    }
                }
            } else {
                log::trace!("Pushed bad move!");
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
        total_steps += step;
        log::info!("{}", format!("Game Won : {:?}",step));
        hh.log_state();
        log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        log::info!("");
        prob.reset();
        game += 1;
    }
    let elapsed_time = start_time.elapsed();
    log::info!("Most Steps: {}", max_steps);
    println!("Total Moves Calculated: {}", total_steps);
    println!("Most Steps: {}", max_steps);
    println!("Total Time: {:?}", elapsed_time);
}

pub fn logger(){
    // let log_file = File::create("app.log").unwrap();

    let log_file = File::create("rustapp.log").expect("Failed to create log file");

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
        .filter(None, LevelFilter::Trace) // Adjust the log level as needed
        // Direct logs to the file
        .target(Target::Pipe(Box::new(log_file)))
        // Apply the configuration
        .init();
}

pub fn clear_logs() -> io::Result<()> {
    // Open the file in write mode to truncate and clear it
    let mut file = File::create("rustapp.log")?;
    // Optionally, you can write something to indicate the logs were cleared
    writeln!(file, "Logs cleared at {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"))?;
    Ok(())
}