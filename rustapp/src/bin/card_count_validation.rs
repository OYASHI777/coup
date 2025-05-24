use log::LevelFilter;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rustapp::history_public::{AOName, ActionObservation, Card, History};
use rustapp::prob_manager::backtracking_collective_constraints::{ActionInfo, BackTrackCollectiveConstraint};
use rustapp::prob_manager::backtracking_collective_constraints_lazy::BackTrackCollectiveConstraintLazy;
use rustapp::prob_manager::backtracking_collective_constraints_lite::BackTrackCollectiveConstraintLite;
use rustapp::prob_manager::backtracking_prob::{BackTrackCardCountManager, CoupConstraint, CoupConstraintAnalysis};
use rustapp::prob_manager::brute_prob_generic::{BruteCardCountManagerGeneric};
use rustapp::prob_manager::card_state::card_state_u64::{self, CardStateu64};
use rustapp::prob_manager::compressed_group_constraint::{CompressedGroupConstraint};
use rustapp::prob_manager::collective_constraint::{CompressedCollectiveConstraint};
use rustapp::prob_manager::brute_prob::BruteCardCountManager;
use rustapp::prob_manager::bit_prob::BitCardCountManager;
use rustapp::prob_manager::path_dependent_prob::PathDependentCardCountManager;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Write};
use std::path::Path;
use std::time::Instant;
use itertools::Itertools;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use env_logger::{Builder, Env, Target};
use ActionObservation::*;
use Card::*;
pub const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
pub const LOG_FILE_NAME: &str = "just_test_replay_000000000.log";
// TODO: Add a test function to compare path_dependent_group_constraint with new approach and if different run inference for the brute force approach
fn main() {
    let game_no = 100000000;
    let log_bool = true;
    let bool_know_priv_info = true;
    let print_frequency: usize = 500;
    let print_frequency_fast: usize = 5000;
    let min_dead_check: usize = 0;
    let num_threads = 12;
    // TODO: YOU NEED TO FIND THE SUBTRACT WITH OVERFLOW!!!
    // let replay = vec![Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Duke, redraw: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 0 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Captain, redraw: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Duke, redraw: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Assassinate { player_id: 0, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Assassin, redraw: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, BlockAssassinate { player_id: 5, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }];
    // let replay = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, true], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Captain, redraw: Ambassador }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 0, card: Captain }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Captain, redraw: Duke }];
    // replay_game_constraint_bt::<BackTrackCollectiveConstraintLite>(replay.clone(), false, 0,true);
    // TODO: Add player and cards
    game_rnd_constraint_bt_mt::<BackTrackCollectiveConstraintLite>(num_threads, game_no, bool_know_priv_info, print_frequency, min_dead_check);
    // game_rnd_constraint_bt_st_debug::<BackTrackCollectiveConstraintLite>(game_no, bool_know_priv_info, min_dead_check, log_bool);
    // This has the no legit move issue
    // vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Captain, redraw: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain, redraw: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Duke, redraw: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain, redraw: Contessa }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, RevealRedraw { player_id: 0, reveal: Assassin, redraw: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockAssassinate { player_id: 4, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }];
    // new_moves: [Discard { player_id: 4, card: [Ambassador, Duke], no_cards: 2 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 2 }, Discard { player_id: 4, card: [Ambassador, Assassin], no_cards: 2 }, Discard { player_id: 4, card: [Ambassador, Captain], no_cards: 2 }, Discard { player_id: 4, card: [Captain, Duke], no_cards: 2 }, Discard { player_id: 4, card: [Assassin, Duke], no_cards: 2 }, Discard { player_id: 4, card: [Assassin, Captain], no_cards: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 2 }]
    
    // game_rnd_constraint_bt_mt_g::<BackTrackCollectiveConstraintLite, BackTrackCollectiveConstraintLazy>(num_threads, game_no, bool_know_priv_info, print_frequency_fast, min_dead_check);
    // game_rnd_constraint_bt_bench(100);
    // TODO: Make a Lite with Lazy Inferred Constraints
    // game_rnd_constraint_bt_generic_bench::<BackTrackCollectiveConstraint>(1000);
    // game_rnd_constraint_bt_generic_bench::<BackTrackCollectiveConstraintLite>(1000);
    // game_rnd_constraint_bt_generic_bench::<BackTrackCollectiveConstraintLazy>(1000);
    // game_rnd_constraint_bt_bench(100);
    // game_rnd_constraint_brute_bench(10);
    // game_rnd_constraint_pd_mt(num_threads, game_no, bool_know_priv_info, print_frequency, min_dead_check);
    // game_rnd_constraint_pd(game_no, bool_know_priv_info, print_frequency, log_bool, min_dead_check);
    // test_brute(game_no, bool_know_priv_info, print_frequency, log_bool);
    // speed(game_no, bool_know_priv_info, 10, log_bool);
    // game_rnd_constraint_debug(game_no, bool_know_priv_info, print_frequency, log_bool);
    // game_rnd_constraint_debug_pd(1, print_frequency, log_bool);
    // game_rnd_constraint_debug_pd_alone(game_no, print_frequency, log_bool);
    // {
    //     use ActionObservation::*;
    //     use Card::*;
    //     // Please test out different redundancies
    //     // Failed impossible constraint
    //     let replay = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, card: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 2, opposing_player_id: 5, amount: 0 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, card: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, card: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, true, true, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 3 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, card: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, card: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }];
    //     replay_game_constraint_pd(replay, bool_know_priv_info, log_bool);
    // }
    // game_rnd(game_no, bool_know_priv_info, print_frequency, log_bool);
    // temp_test_brute();
    // instant_delete();
    // bt_test::<BackTrackCollectiveConstraintLite>();
    // bt_test::<BackTrackCollectiveConstraintLazy>();
    // bt_test::<BackTrackCollectiveConstraint>();
    // test();
    test_variant_recurse();
}
use rustapp::prob_manager::path_dependent_collective_constraint::{self, PathDependentCollectiveConstraint};
// TODO: Move to collective_constraint when finalized
pub fn test_variant_recurse() {
    fn gen_variants(
        card_types: &[Card],
        max_cards: usize,
    ) -> Vec<Vec<Card>> {
        let mut variants = Vec::new();
        // for each possible hand size 0..=max_cards
        for size in 0..=max_cards {
            // Generate all multisets of exactly `size` cards
            for combo in card_types.iter().combinations_with_replacement(size) {
                // sort and collect into Vec<Card>
                let mut hand = combo.into_iter().cloned().collect::<Vec<_>>();
                // ensure deterministic ordering
                hand.sort_by_key(|c| *c as u8);
                variants.push(hand);
            }
        }
        variants
    }
    logger(LevelFilter::Info);
    let mut test_inferred_constraints: HashSet<Vec<Vec<Card>>> = HashSet::new();
    for player_hand in gen_variants(&vec![Card::Ambassador, Card::Assassin], 2) {
        'outer: for pile_hand in gen_variants(&vec![Card::Ambassador, Card::Assassin, Card::Captain], 3) {
            for card in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa] {
                if player_hand.iter().filter(|c| **c == card).count() + pile_hand.iter().filter(|c| **c == card).count() > 3 {
                    continue 'outer;
                }
                test_inferred_constraints.insert(vec![player_hand.clone(), vec![], vec![], vec![], vec![], vec![], pile_hand.clone()]);
            }
        }
    }
    let mut symmetric_keys: HashSet<[u8; 10]> = HashSet::with_capacity(200);
    'outer: for item in test_inferred_constraints.iter() {
        for card_num in 0..5 {
            if item.iter().map(|v| v.iter().filter(|c| **c as usize == card_num).count() as u8).sum::<u8>() > 3 {
                continue 'outer;
            }
        }
        // let reveal = PathDependentCollectiveConstraint::return_variants_reveal_redraw_none(Card::Ambassador, 0, item);
        // log::info!("src rr unop: {:?}", reveal);
        // let reveal = PathDependentCollectiveConstraint::return_variants_reveal_redraw_none_opt(Card::Ambassador, 0, item);
        // log::info!("src rr none: {:?}", reveal);
        // let redraw = PathDependentCollectiveConstraint::return_variants_reveal_redraw(Card::Ambassador, Card::Assassin, 0, item);
        // log::info!("src rr draw: {:?}", redraw);
        // let relin = PathDependentCollectiveConstraint::return_variants_reveal_relinquish_opt(Card::Ambassador, 0, item);
        // log::info!("src rr rel: {:?}", relin);
        // let exchange_1 = PathDependentCollectiveConstraint::return_variants_exchange_opt(1, 0, item);
        // log::info!("src ex one: {:?}", exchange_1);
        // let exchange_2 = PathDependentCollectiveConstraint::return_variants_exchange_opt(2, 0, item);
        // log::info!("src ex two: {:?}", exchange_2);
        // WRite a test for all cases, generate some start state, go fwd, then go bwd, then check if bwd generated item includes start state
        let mut hand_count: [u8; 5] = [0; 5];
        item[0].iter().for_each(|c| hand_count[*c as usize] += 1);
        let mut pile_count: [u8; 5] = [0; 5];
        item[6].iter().for_each(|c| pile_count[*c as usize] += 1);
        let mut symmetric_key: [u8; 10] = [0; 10];
        let mut temp = hand_count.clone();
        temp.sort_unstable();
        temp.iter().enumerate().for_each(|(i, c)| symmetric_key[i] = *c);
        let mut temp = pile_count.clone();
        temp.sort_unstable();
        temp.iter().enumerate().for_each(|(i, c)| symmetric_key[i+5] = *c);
        if symmetric_keys.contains(&symmetric_key) {
            // Avoiding symmetric cases
            continue 'outer;
        }
        symmetric_keys.insert(symmetric_key);
        let mut hand_and_pile_count = hand_count.clone();
        hand_and_pile_count.iter_mut().zip(pile_count.iter()).for_each(|(hp, p)| *hp += *p);
        for i in 0..5 {
            for j in i..5 {
                for k in 0..5 {
                    'inner: for l in k..5 {
                        let mut temp_count: [u8; 5] = hand_count.clone();
                        temp_count[i as usize] += 1;
                        temp_count[j as usize] += 1;
                        let mut relin_count: [u8; 5] = [0; 5];
                        relin_count[k as usize] += 1;
                        relin_count[l as usize] += 1;
                        let mut draw_count: [u8; 5] = [0; 5];
                        draw_count[i as usize] += 1;
                        draw_count[j as usize] += 1;
                        // Get union of all 3
                        let mut union_count_0: u8 = 0;
                        let mut union_count_1: u8 = 0;
                        for m in 0..5 {
                            // relinquish must be in outcome hand + draw
                            // not exact
                            // draw + hand + relin at most 4
                            union_count_0 += hand_count[m].max(draw_count[m]).max(relin_count[m]);
                            // draw + hand + relin + pile at most 5
                            let max_union_count = hand_and_pile_count[m].max(relin_count[m]).max(pile_count[m]);
                            // max cards at most 3
                            if max_union_count > 3 {
                                continue 'inner;
                            }
                            union_count_1 += max_union_count;
                        }
                        if union_count_0 > 4 || union_count_1 > 5{
                            continue 'inner;
                        }
                        // Enough space to have received the relinquish
                        let mut pile_space: u8 = 0;
                        pile_count.iter().zip(relin_count.iter()).for_each(|(p, r)| pile_space += *p.max(r));
                        if pile_space > 3 {
                            continue 'inner;
                        }
                        // Total amount in (draw union relinquish) and not in (hand + pile) cannot exceed (5 - hand.len() - pile.len())
                        let mut draw_union_relin_count: [u8; 5] = [0; 5];
                        let mut hand_plus_pile_count: [u8; 5] = [0; 5];
                        let mut degrees_of_freedom_used: u8 = 0;
                        draw_count.iter().zip(relin_count.iter()).enumerate().for_each(|(c, (d, r))| draw_union_relin_count[c] += *d.max(r));
                        hand_plus_pile_count.iter_mut().enumerate().for_each(|(i, count)| *count = hand_count[i] + pile_count[i]);
                        hand_plus_pile_count.iter().zip(draw_union_relin_count.iter()).for_each(|(h, d)| {
                            if *d > *h {
                                degrees_of_freedom_used += *d - *h;
                            }
                        });
                        if degrees_of_freedom_used + hand_plus_pile_count.iter().sum::<u8>() > 5{
                            continue 'inner;
                        }
                        // destination hand must be  
                        let draw = vec![Card::try_from(i).unwrap(), Card::try_from(j).unwrap()];
                        let relinquish = vec![Card::try_from(k).unwrap(), Card::try_from(l).unwrap()];
                        let exchange_private = PathDependentCollectiveConstraint::return_variants_exchange_private_3(0, &draw, &relinquish,item);
                        log::info!("src ex_p draw: {:?}, relin: {:?} : {:?}", draw, relinquish, exchange_private);
                        // Length Check
                        for v in exchange_private.iter() {
                            if v[0].len() > 2 || v[6].len() > 3 {
                                log::warn!("FAILED ORIG LEN CHECK");
                                continue 'inner;
                            }
                        }
                        // Test if when all draw != relin, old hand == relinquish
                        // Test can be done via reversing it to verify if given solution is legit
                    }
                }
            }
        }
        // if reveal.len() < relin.len() {
        //     log::warn!("reveal failed constraint check");
        // }
        // if reveal.len() < redraw.len() {
        //     log::warn!("reveal redraw failed constraint check");
        // }
        // if relin.len() < redraw.len() {
        //     log::warn!("redraw relin failed constraint check");
        // }
        log::info!("dest: {:?}", item);

    }
    // dest: [[Ambassador], [], [], [], [], [], [Ambassador, Ambassador, Assassin]]
    // dest: [[Assassin], [], [], [], [], [], [Assassin, Captain]]
    // dest: [[Ambassador], [], [], [], [], [], [Ambassador, Ambassador, Captain]]
    // dest: [[Ambassador], [], [], [], [], [], [Assassin]]
    // src rr none: [[[Assassin, Ambassador], [], [], [], [], [], [Assassin]], [[Assassin, Ambassador], [], [], [], [], [], [Assassin, Assassin]]]
    // src rr draw: [[[Ambassador, Ambassador], [], [], [], [], [], [Assassin, Assassin]]]
    // src rr rel: [[[Assassin, Ambassador], [], [], [], [], [], [Assassin]], [[Assassin, Ambassador], [], [], [], [], [], [Assassin, Assassin]]]
    // dest: [[Ambassador, Assassin], [], [], [], [], [], [Assassin]]
    // for item in test_inferred_constraints.iter() {
    //     let cc = PathDependentCollectiveConstraint::return_variants_reveal_redraw_none_opt(Card::Ambassador, 0, item);
    //     log::info!("src: {:?}", cc);
    //     log::info!("dest: {:?}", item);
    // }
    // for item in test_inferred_constaints.iter() {
    //     let cc = PathDependentCollectiveConstraint::return_variants_reveal_redraw(Card::Ambassador, Card::Captain, 0, item);
    //     log::info!("src: {:?}", cc);
    //     log::info!("dest: {:?}", item);
    // }
}
pub fn bt_test<C>() 
    where
        C: CoupConstraint + CoupConstraintAnalysis,
{
    // let rr_0 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 3, opposing_player_id: 4, amount: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Captain }];
    // let rr_imp_0 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, true, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 3 }, Assassinate { player_id: 4, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
    // let rr_imp_1 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 0 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 1 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 1 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 2 }, Assassinate { player_id: 4, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
    // let amb_0 = vec![Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 1, final_actioner: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Assassin }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockAssassinate { player_id: 1, opposing_player_id: 1 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Exchange { player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, ExchangeDraw { player_id: 4, card: [Ambassador, Ambassador] }];
    // let amb_1 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Exchange { player_id: 3 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Ambassador }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, ExchangeDraw { player_id: 3, card: [Ambassador, Contessa] }, ExchangeChoice { player_id: 3, no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Ambassador }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, ExchangeDraw { player_id: 3, card: [Ambassador, Duke] }, ExchangeChoice { player_id: 3, no_cards: 1 }, Income { player_id: 5 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }];
    // let amb_2 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Ambassador }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ExchangeDraw { player_id: 1, card: [Captain, Contessa] }, ExchangeChoice { player_id: 1, no_cards: 2 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Exchange { player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 3, final_actioner: 3 }, ExchangeDraw { player_id: 3, card: [Captain, Captain] }, ExchangeChoice { player_id: 3, no_cards: 2 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }];
    // // Unsure if this is just Lazy or affects all?
    // let amb_3 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 0 }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, ExchangeDraw { player_id: 1, card: [Captain, Contessa] }, ExchangeChoice { player_id: 1, no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Ambassador }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Exchange { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }];
    // let amb_4 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 5 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, ExchangeDraw { player_id: 5, card: [Assassin, Assassin] }, ExchangeChoice { player_id: 5, no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Assassin }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 1, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }];
    // let amb_5 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Exchange { player_id: 4 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Exchange { player_id: 5 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ExchangeDraw { player_id: 5, card: [Assassin, Captain] }, ExchangeChoice { player_id: 5, no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Exchange { player_id: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }];
    
    // println!("Testing: {}", stringify!(rr_0)); 
    // replay_game_constraint_bt::<C>(rr_0, false, true);
    // println!("Testing: {}", stringify!(rr_imp_0)); 
    // replay_game_constraint_bt::<C>(rr_imp_0, false, false);
    // println!("Testing: {}", stringify!(rr_imp_1)); 
    // replay_game_constraint_bt::<C>(rr_imp_1, false, false);
    // println!("Testing: {}", stringify!(amb_0)); 
    // replay_game_constraint_bt::<C>(amb_0, false, false);
    // println!("Testing: {}", stringify!(amb_1)); 
    // replay_game_constraint_bt::<C>(amb_1, false, false);
    // println!("Testing: {}", stringify!(amb_2)); 
    // replay_game_constraint_bt::<C>(amb_2, false, true);
    // println!("Testing: {}", stringify!(amb_3)); 
    // replay_game_constraint_bt::<C>(amb_3, false, true);
    // println!("Testing: {}", stringify!(amb_4)); 
    // replay_game_constraint_bt::<C>(amb_4, false, false);
    // println!("Testing: {}", stringify!(amb_5)); 
    // replay_game_constraint_bt::<C>(amb_5.clone(), false, false);
}
// pub fn test<C>() 
//     where
//         C: CoupConstraint + CoupConstraintAnalysis,
// {
//     {
//         // TODO: !!! You added both the start inference and the pile inference, test them seperately

//         let relinquish_0 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let subtract_overflow_0 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }];
//         let subtract_overflow_1 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 3, final_actioner: 3 }, BlockSteal { player_id: 5, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Income { player_id: 3 }, Assassinate { player_id: 4, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 5, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }];
//         let bad_push_0 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let bad_push_1 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 2, card: Captain }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }];
//         let bad_push_2 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 1, final_actioner: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }];
//         let bad_push_3 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 5 }, Income { player_id: 0 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let reveal_redraw_issue_0 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }];
//         let reveal_redraw_replay_0 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 5 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }];
//         let reveal_redraw_replay_1 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 1 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Assassin }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }];
//         let reveal_redraw_replay_2 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 1, final_actioner: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, true, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 2, opposing_player_id: 5, amount: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }];
//         let reveal_redraw_replay_3 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 0, card: Captain }, CollectiveChallenge { participants: [true, true, true, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 0 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 3 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, true, false, true, false, true], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 5 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Assassin }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockAssassinate { player_id: 4, opposing_player_id: 4 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 5 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }];
//         let reveal_redraw_replay_4 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, true, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }];
//         let reveal_redraw_replay_5 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }];
//         let reveal_redraw_replay_6 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
//         let reveal_redraw_replay_7 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, false, true], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }];
//         let reveal_redraw_replay_8 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, true, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 3 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }];
//         let reveal_redraw_replay_9 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 5, card: Captain }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }];
//         let reveal_redraw_replay_10 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let reveal_redraw_replay_11 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 5, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 0, final_actioner: 0 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, true, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }];
//         let reveal_redraw_replay_12 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, BlockSteal { player_id: 2, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 5, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 0 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 1 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }];
//         let reveal_redraw_replay_13 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, false, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Duke }];
//         let reveal_redraw_replay_14 = vec![Income { player_id: 0 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 5, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 4, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 5 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }];
//         let reveal_redraw_replay_15 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 0 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 2 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 1 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }];
//         let reveal_redraw_replay_16 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, Steal { player_id: 1, opposing_player_id: 2, amount: 0 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 1, final_actioner: 1 }, Income { player_id: 1 }, Income { player_id: 4 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }];
//         let reveal_redraw_replay_17 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
//         let reveal_redraw_replay_18 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 2 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 5, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Assassin }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let reveal_redraw_replay_19 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let full_test_replay_0 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }];
//         let full_test_replay_1 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 0, final_actioner: 0 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 4, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_1_modified = vec![Income { player_id: 0 }, Income { player_id: 1 }, Income { player_id: 2 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 0, final_actioner: 0 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 4, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_2 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, true, true, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let full_test_replay_3 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 2, opposing_player_id: 4, amount: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_4 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 3, card: Captain }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Income { player_id: 0 }, Assassinate { player_id: 1, opposing_player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 1, amount: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
//         let full_test_replay_5 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }];
//         let full_test_replay_6 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 3, card: Captain }, Steal { player_id: 3, opposing_player_id: 0, amount: 0 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 0 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Assassin }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 5 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }];
//         let full_test_replay_7 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 4 }, Assassinate { player_id: 0, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 2 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_8 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, false, false, false, false], opposing_player_id: 3, final_actioner: 3 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let full_test_replay_9 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_10 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 0 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }];
//         let full_test_replay_11 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, true, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 2 }, Assassinate { player_id: 3, opposing_player_id: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_12 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 0, card: Captain }, CollectiveChallenge { participants: [true, false, true, true, true, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_13 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, true, true, true, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }];
//         let full_test_replay_14 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, true, true, false, false], opposing_player_id: 1, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, false, false, false], opposing_player_id: 5, final_actioner: 5 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }];
//         let full_test_replay_15 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Ambassador }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 4, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let full_test_replay_16 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 0, final_actioner: 0 }, BlockSteal { player_id: 4, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 3 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let redundancy_replay_0 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }];
//         let full_test_overflow_0 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let full_test_overflow_1 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, false, false, false, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 4 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, false, true, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }];
//         let full_test_overflow_2 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, true, true, true, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, true, true, true, true, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }];
//         let whole_replay_0 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }];
//         // Uses Exchange
//         let whole_replay_1 = vec![Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 5 }, RevealRedraw { player_id: 0, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, ExchangeDraw { player_id: 0, card: [Ambassador, Duke] }, ExchangeChoice { player_id: 0, no_cards: 2 }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Ambassador }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, ExchangeDraw { player_id: 1, card: [Ambassador, Assassin] }, ExchangeChoice { player_id: 1, no_cards: 2 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }];
//         // Uses Exchange
//         let whole_replay_2 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Exchange { player_id: 1 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Exchange { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Ambassador }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ExchangeDraw { player_id: 2, card: [Duke, Duke] }, ExchangeChoice { player_id: 2, no_cards: 2 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 2 }, Income { player_id: 4 }, Exchange { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Ambassador }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ExchangeDraw { player_id: 0, card: [Ambassador, Captain] }, ExchangeChoice { player_id: 0, no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let whole_replay_3 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 1 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }];
//         // Uses Exchange
//         let whole_replay_4 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Exchange { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Ambassador }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, ExchangeDraw { player_id: 2, card: [Captain, Duke] }, ExchangeChoice { player_id: 2, no_cards: 2 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Exchange { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }];
//         let backward_compat_0 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 0 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let impossible_0 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, true, true, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, true, false, true, true], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }];
//         let impossible_1 = vec![Income { player_id: 0 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, true, true, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 3, amount: 1 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Duke }];
//         let impossible_2 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, true, true, false, false], opposing_player_id: 5, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 5, final_actioner: 5 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 1 }, Income { player_id: 3 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }];
//         let impossible_3 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
//         let impossible_4 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
//         let impossible_5 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 0 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
//         let impossible_6 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, true, true, true, true], opposing_player_id: 1, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 0 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Income { player_id: 4 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let impossible_7 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 5, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Ambassador }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Assassin }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, BlockAssassinate { player_id: 1, opposing_player_id: 1 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_0 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_1 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Income { player_id: 4 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Assassin }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, BlockAssassinate { player_id: 4, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 0 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_2 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, false], opposing_player_id: 3, final_actioner: 0 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, true, true, true, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 1 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 4, amount: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 4, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Assassin }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }];
//         let overinferred_3 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 3 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, false, true, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 3 }, Assassinate { player_id: 0, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_4 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_5 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, true, false, true], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, reveal: Assassin }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockAssassinate { player_id: 2, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, true, false, true, true, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 1 }, Income { player_id: 3 }, Income { player_id: 4 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }];
//         let overinferred_6 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 3 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 4 }, Income { player_id: 5 }, Assassinate { player_id: 0, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_7 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Assassin }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 5 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Captain }];
//         let overinferred_8 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, true, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 2 }, Income { player_id: 3 }, Income { player_id: 1 }, Assassinate { player_id: 2, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Assassin }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, BlockAssassinate { player_id: 3, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_9 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 1, card: Captain }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_10 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, true, false, true, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [true, true, false, true, false, true], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Ambassador }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Ambassador }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }, Income { player_id: 0 }, Income { player_id: 2 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_11 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Duke }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 1 }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 3, card: Ambassador }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, true], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }];
//         let overinferred_12 = vec![Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Duke }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, true, true], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 0, card: Captain }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, true, false, true], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 2, card: Captain }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 3, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 5, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
//         let overinferred_13 = vec![Income { player_id: 0 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 4 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, false], opposing_player_id: 5, final_actioner: 4 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 1 }, CollectiveBlock { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 5 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 0, final_actioner: 5 }, RevealRedraw { player_id: 0, reveal: Assassin }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }];
//         let overinferred_14 = vec![Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, RevealRedraw { player_id: 5, reveal: Captain }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 5, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Ambassador }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, reveal: Ambassador }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }];
//         let overinferred_15 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, RevealRedraw { player_id: 3, reveal: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, reveal: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 0, card: Ambassador }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, false, false, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 2 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }];
//         let overinferred_16 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 2, final_actioner: 2 }, BlockSteal { player_id: 5, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, true, true, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, reveal: Duke }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 5, amount: 0 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, reveal: Captain }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, false], opposing_player_id: 0, final_actioner: 4 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 2 }, Assassinate { player_id: 3, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 2 }, Income { player_id: 3 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }];
//         let overinferred_17 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, true], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 0, card: Captain }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 0 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 0, final_actioner: 2 }, RevealRedraw { player_id: 0, reveal: Captain }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 0, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Income { player_id: 3 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, RevealRedraw { player_id: 5, reveal: Duke }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 5, final_actioner: 0 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, RevealRedraw { player_id: 0, reveal: Assassin }];
//         let overinferred_18 = vec![Steal { player_id: 0, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, false, false], opposing_player_id: 1, final_actioner: 0 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, RevealRedraw { player_id: 4, reveal: Duke }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, reveal: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 4, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, reveal: Duke }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }];
//         // second relinquish
//         // println!("Testing: {}", stringify!(relinquish_0)); 
//         // replay_game_constraint_bt::<C>(relinquish_0, false, false);
//         // Stack Custom illegal
//         println!("Testing: {}", stringify!(reveal_redraw_replay_19)); 
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_19, false, false);
//         // Some max negation
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_18)); 
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_18, false, false);
//         // ====================
//         println!("Testing: {}", stringify!(reveal_redraw_replay_16)); 
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_16, false, false);
//         // Possible to solve with backtracking and not fix with group_constraints
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_15)); 
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_15, false, false);
//         // ===================== 
//         // Second Inference 
//         println!("Testing: {}", stringify!(overinferred_18)); 
//         replay_game_constraint_bt::<C>(overinferred_18, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_14)); 
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_14, false, false);
//         // How to deal with second inference
//         println!("Testing: {}", stringify!(bad_push_1));
//         replay_game_constraint_bt::<C>(bad_push_1, false, false);
//         // ===================== 
//         // Crazy impossible
//         // println!("Testing: {}", stringify!(impossible_6));
//         // replay_game_constraint_bt::<C>(impossible_6, false, false);
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_12)); 
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_12, false, true);
//         // ===================== 
//         // Relinquish case!!
//         println!("Testing: {}", stringify!(reveal_redraw_replay_13)); 
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_13, false, false);
//         // overinferred_11 This tells us the merging did not work out very well
//         // maybe merge also discard into reveal?
//         // actually the front count doesnt really make sense
//         // esp if someone amb
//         // failed after relinquish => the relinquish does not produce the correct groups...
//         println!("Testing: {}", stringify!(overinferred_11)); 
//         replay_game_constraint_bt::<C>(overinferred_11, false, false);
//         // ===================== 
//         println!("Testing: {}", stringify!(reveal_redraw_replay_10));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_10, false, false);
//         println!("Testing: {}", stringify!(overinferred_16)); 
//         replay_game_constraint_bt::<C>(overinferred_16, false, false);
//         println!("Testing: {}", stringify!(overinferred_17)); 
//         replay_game_constraint_bt::<C>(overinferred_17, false, false);
//         println!("Testing: {}", stringify!(overinferred_14)); 
//         replay_game_constraint_bt::<C>(overinferred_14, false, false);
//         println!("Testing: {}", stringify!(impossible_7)); // Helpful for relinquish
//         replay_game_constraint_bt::<C>(impossible_7, false, false);
//         println!("Testing: {}", stringify!(bad_push_3));
//         replay_game_constraint_bt::<C>(bad_push_3, false, false);
//         println!("Testing: {}", stringify!(bad_push_2));
//         replay_game_constraint_bt::<C>(bad_push_2, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_11));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_11, false, false);


//         // IMPT!! Check 4 things & align the RR and Discard, Kinda like RR more
//         // filter() in Discard?
//         // retain is_none() needed in discard?
//         // ordering of discard_player pushes? 
//         // bool_all_dead in discard?
//         // This might fail when AMB is reintroduced
//         // Hacky cases for Redraw inference
//         // ========================================
//         // === Non PD inference
//         // I have no idea whats wrong here lol, maybe same as subtract overflow
//         // println!("Testing: {}", stringify!(overinferred_8));
//         // replay_game_constraint_bt::<C>(overinferred_8, false, true);
//         // Set inference
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_9));
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_9, false, false);
//         // === Non PD inference
//         // === PD RevealRedraw cases
//         // Hmm what about revealredraws that happen before?
//         // P1 RR D
//         // P2 RR D
//         // P3 RR D
//         // P3 D Duke
//         // => P3 redrew the Duke
//         // maybe i do a forward pass to find all the first time Reveal Dukes?
//         // How does this interact with Ambassador?
//         // Or like
//         // P2 RR D
//         // P1 RR D
//         // P2 D D
//         // P1 D D
//         // P3 D D
//         println!("Testing: {}", stringify!(overinferred_13));
//         replay_game_constraint_bt::<C>(overinferred_13, false, false);
//         println!("Testing: {}", stringify!(overinferred_9));
//         replay_game_constraint_bt::<C>(overinferred_9, false, false);
//         println!("Testing: {}", stringify!(overinferred_10));
//         replay_game_constraint_bt::<C>(overinferred_10, false, false);
//         println!("Testing: {}", stringify!(impossible_5));
//         replay_game_constraint_bt::<C>(impossible_5, false, false);
//         // TODO: TEST THIS => Single Group constraint of all known at start forward pass?
//         println!("Testing: {}", stringify!(overinferred_12)); // stored but untested
//         replay_game_constraint_bt::<C>(overinferred_12, false, false);

//         println!("Testing: {}", stringify!(overinferred_7));
//         replay_game_constraint_bt::<C>(overinferred_7, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_8));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_8, false, false);
//         println!("Testing: {}", stringify!(overinferred_6));
//         replay_game_constraint_bt::<C>(overinferred_6, false, false);
//         println!("Testing: {}", stringify!(overinferred_5));
//         replay_game_constraint_bt::<C>(overinferred_5, false, false);
//         println!("Testing: {}", stringify!(impossible_2));
//         replay_game_constraint_bt::<C>(impossible_2, false, false);
//         // === PD RevealRedraw cases
//         // ===================================
//         // Future features / weird bugs
//         // Can't find root of this bug
//         // println!("Testing: {}", stringify!(subtract_overflow_1));
//         // replay_game_constraint_bt::<C>(subtract_overflow_1, false, true);
//         // Recursion
//         // println!("Testing: {}", stringify!(impossible_3));
//         // replay_game_constraint_bt::<C>(impossible_3, false, false);
//         // Set inference
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_6));
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_6, false, false);
//         // println!("Testing: {}", stringify!(reveal_redraw_replay_17)); 
//         // replay_game_constraint_bt::<C>(reveal_redraw_replay_17, false, false);
//         // ===================================
        
//         // Broke this
//         println!("Testing: {}", stringify!(full_test_overflow_0));
//         replay_game_constraint_bt::<C>(full_test_overflow_0, false, false);
//         println!("Testing: {}", stringify!(overinferred_0));
//         replay_game_constraint_bt::<C>(overinferred_0, false, false);
//         println!("Testing: {}", stringify!(overinferred_1));
//         replay_game_constraint_bt::<C>(overinferred_1, false, false);
//         println!("Testing: {}", stringify!(overinferred_2));
//         replay_game_constraint_bt::<C>(overinferred_2, false, false);
//         println!("Testing: {}", stringify!(overinferred_3));
//         replay_game_constraint_bt::<C>(overinferred_3, false, false);
//         println!("Testing: {}", stringify!(overinferred_4));
//         replay_game_constraint_bt::<C>(overinferred_4, false, false);
//         println!("Testing: {}", stringify!(subtract_overflow_0));
//         replay_game_constraint_bt::<C>(subtract_overflow_0, false, false);
//         println!("Testing: {}", stringify!(bad_push));
//         replay_game_constraint_bt::<C>(bad_push_0, false, false);
//         println!("Testing: {}", stringify!(impossible_0));
//         replay_game_constraint_bt::<C>(impossible_0, false, false);
//         println!("Testing: {}", stringify!(impossible_1));
//         replay_game_constraint_bt::<C>(impossible_1, false, false);
//         println!("Testing: {}", stringify!(impossible_4));
//         replay_game_constraint_bt::<C>(impossible_4, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_1));
//         replay_game_constraint_bt::<C>(full_test_replay_1, false, false);
        
//         println!("Testing: {}", stringify!(full_test_overflow_1));
//         replay_game_constraint_bt::<C>(full_test_overflow_1, false, false);
//         println!("Testing: {}", stringify!(full_test_overflow_2));
//         replay_game_constraint_bt::<C>(full_test_overflow_2, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_issue_0));
//         replay_game_constraint_bt::<C>(reveal_redraw_issue_0, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_0));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_0, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_1));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_1, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_2));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_2, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_3));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_3, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_4));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_4, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_5));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_5, false, false);
//         println!("Testing: {}", stringify!(reveal_redraw_replay_7));
//         replay_game_constraint_bt::<C>(reveal_redraw_replay_7, false, false);
        
//         println!("Testing: {}", stringify!(full_test_replay_0));
//         replay_game_constraint_bt::<C>(full_test_replay_0, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_1_modified));
//         replay_game_constraint_bt::<C>(full_test_replay_1_modified, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_2));
//         replay_game_constraint_bt::<C>(full_test_replay_2, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_3));
//         replay_game_constraint_bt::<C>(full_test_replay_3, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_4));
//         replay_game_constraint_bt::<C>(full_test_replay_4, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_5));
//         replay_game_constraint_bt::<C>(full_test_replay_5, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_6));
//         replay_game_constraint_bt::<C>(full_test_replay_6, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_7));
//         replay_game_constraint_bt::<C>(full_test_replay_7, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_8));
//         replay_game_constraint_bt::<C>(full_test_replay_8, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_9));
//         replay_game_constraint_bt::<C>(full_test_replay_9, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_10));
//         replay_game_constraint_bt::<C>(full_test_replay_10, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_11));
//         replay_game_constraint_bt::<C>(full_test_replay_11, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_12));
//         replay_game_constraint_bt::<C>(full_test_replay_12, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_13));
//         replay_game_constraint_bt::<C>(full_test_replay_13, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_14));
//         replay_game_constraint_bt::<C>(full_test_replay_14, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_15));
//         replay_game_constraint_bt::<C>(full_test_replay_15, false, false);
//         println!("Testing: {}", stringify!(full_test_replay_16));
//         replay_game_constraint_bt::<C>(full_test_replay_16, false, false);
//         // === fails for impossible to have a particular card
//         // Because of ambassador we store this first for later
//         // println!("Testing: {}", stringify!(whole_replay_0));
//         // replay_game_constraint_bt::<C>(whole_replay_0, false, false);
//         // println!("Testing: {}", stringify!(whole_replay_1));
//         // replay_game_constraint_bt::<C>(whole_replay_1, false, true);
//         // println!("Testing: {}", stringify!(whole_replay_2));
//         // replay_game_constraint_bt::<C>(whole_replay_2, false, false);
//         // println!("Testing: {}", stringify!(whole_replay_3));
//         // replay_game_constraint_bt::<C>(whole_replay_3, false, false);
//         // println!("Testing: {}", stringify!(whole_replay_4));
//         // replay_game_constraint_bt::<C>(whole_replay_4, false, false);
//         // =================================================
        
//         println!("Testing: {}", stringify!(redundancy_replay_0));
//         replay_game_constraint_bt::<C>(redundancy_replay_0, false, false);
//         println!("Testing: {}", stringify!(backward_compat_0));
//         replay_game_constraint_bt::<C>(backward_compat_0, false, false);
//         println!("ALL PASSED"); 
//     }
// }

#[derive(Default)]
pub struct Stats {
    pub games: usize,
    pub max_steps: usize,
    pub public_constraints_correct: usize,
    pub inferred_constraints_correct: usize,
    pub impossible_constraints_correct: usize,
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
    pub fn pushed_bad_move(&self) -> bool {
        self.pushed_bad_move > 0
    }

    pub fn games(&self) -> usize {
        self.games
    }

    pub fn print(&self) {
        println!("Game: {}", self.games);
        println!("Public Constraints Incorrect: {}/{}", self.total_tries - self.public_constraints_correct, self.games);
        println!("Inferred Constraints Incorrect: {}/{}", self.total_tries - self.inferred_constraints_correct, self.games);
        println!("Inferred Constraints Overinferred: {}/{}", self.over_inferred_count, self.games);
        println!("Impossible Cases Incorrect: {}/{}", self.total_tries - self.impossible_constraints_correct, self.games);
        println!("Bad Moves Pushed: {}/{}", self.pushed_bad_move, self.games);
    }
}
pub fn game_rnd_constraint_mt(num_threads: usize, game_no: usize, bool_know_priv_info: bool, print_frequency: usize, min_dead_check: usize){
    let replay_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("replays_only.log")
            .expect("Unable to open replays_only.log"),
    ));
    
    let (tx, rx) = mpsc::channel();
    let games_per_thread = game_no / 4;
    let extra_games = game_no % 4;
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let thread_tx = tx.clone();
        let thread_games = games_per_thread + if i < extra_games {1} else {0};
        let thread_bool_know_priv_info = bool_know_priv_info;
        let thread_min_dead_check = min_dead_check;
        let handle = thread::spawn(move || {
            game_rnd_constraint_st(thread_games, thread_bool_know_priv_info, thread_min_dead_check, thread_tx);
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
        if received.public_correct() && received.inferred_correct() && !received.impossible_correct() {
            status = format!("{status}|<- WRONG ONLY IMPOSSIBLE CONSTRAINTS");
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
pub fn game_rnd_constraint_pd_mt(num_threads: usize, game_no: usize, bool_know_priv_info: bool, print_frequency: usize, min_dead_check: usize){
    let replay_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("replays_only.log")
            .expect("Unable to open replays_only.log"),
    ));
    
    let (tx, rx) = mpsc::channel();
    let games_per_thread = game_no / 4;
    let extra_games = game_no % 4;
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let thread_tx = tx.clone();
        let thread_games = games_per_thread + if i < extra_games {1} else {0};
        let thread_bool_know_priv_info = bool_know_priv_info;
        let thread_min_dead_check = min_dead_check;
        let handle = thread::spawn(move || {
            game_rnd_constraint_pd_st(thread_games, thread_bool_know_priv_info, thread_min_dead_check, thread_tx);
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
        if received.public_correct() && received.inferred_correct() && !received.impossible_correct() {
            status = format!("{status}|<- WRONG ONLY IMPOSSIBLE CONSTRAINTS");
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
pub fn game_rnd_constraint_bt_mt<C>(num_threads: usize, game_no: usize, bool_know_priv_info: bool, print_frequency: usize, min_dead_check: usize)
    where
        C: CoupConstraint + CoupConstraintAnalysis,
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
            game_rnd_constraint_bt_st_new::<C>(thread_games, thread_bool_know_priv_info, thread_min_dead_check, thread_tx);
            // game_rnd_constraint_bt_st::<V, T>(thread_games, thread_bool_know_priv_info, thread_min_dead_check, thread_tx);
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
pub fn game_rnd_constraint_bt_mt_g<V, T>(num_threads: usize, game_no: usize, bool_know_priv_info: bool, print_frequency: usize, min_dead_check: usize)
    where
        V: CoupConstraint + CoupConstraintAnalysis,
        T: CoupConstraint + CoupConstraintAnalysis,
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
            game_rnd_constraint_bt_st_g::<V, T>(thread_games, thread_bool_know_priv_info, thread_min_dead_check, thread_tx);
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
pub fn game_rnd_constraint_st(game_no: usize, bool_know_priv_info: bool, min_dead_check: usize, tx: Sender<Stats>){
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        while !hh.game_won() {
            
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            // new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao_public(&output);
                bit_prob.push_ao(&output, bool_know_priv_info);
                let total_dead: usize = bit_prob.latest_constraint().sorted_public_constraints().iter().map(|v| v.len()).sum();
                if total_dead >= min_dead_check {
                    let validated_public_constraints = prob.validated_public_constraints();
                    let validated_inferred_constraints = prob.validated_inferred_constraints();
                    let validated_impossible_constraints = prob.validated_impossible_constraints();
                    let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                    let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                    let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
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
                    if bool_test_over_inferred {
                        // what we are testing inferred too many things
                        stats.over_inferred_count += 1;
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred to many items!")
                    }
                    if !pass_inferred_constraints {
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred constraints do not match!")
                    }
                    if !pass_impossible_constraints {
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!()
                    }
                }


            } else {
                // log::trace!("Pushed bad move somewhere earlier!");
                stats.pushed_bad_move += 1;
                break;
            }
            // bit_prob.debug_panicker();
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
pub fn game_rnd_constraint_pd_st(game_no: usize, bool_know_priv_info: bool, min_dead_check: usize, tx: Sender<Stats>){
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob = PathDependentCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        while !hh.game_won() {
            
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao_public(&output);
                bit_prob.push_ao_public(&output);
                let total_dead: usize = bit_prob.latest_constraint().sorted_public_constraints().iter().map(|v| v.len()).sum();
                if total_dead >= min_dead_check {
                    let validated_public_constraints = prob.validated_public_constraints();
                    let validated_inferred_constraints = prob.validated_inferred_constraints();
                    let validated_impossible_constraints = prob.validated_impossible_constraints();
                    let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                    let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                    let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
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
                    if bool_test_over_inferred {
                        // what we are testing inferred too many things
                        stats.over_inferred_count += 1;
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred to many items!")
                    }
                    if !pass_inferred_constraints {
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred constraints do not match!")
                    }
                    if !pass_impossible_constraints {
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!()
                    }
                }


            } else {
                // log::trace!("Pushed bad move somewhere earlier!");
                stats.pushed_bad_move += 1;
                break;
            }
            // bit_prob.debug_panicker();
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
pub fn game_rnd_constraint_bt_st_new<C>(game_no: usize, bool_know_priv_info: bool, min_dead_check: usize, tx: Sender<Stats>)
    where
        C: CoupConstraint + CoupConstraintAnalysis,
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraintLight> = BackTrackCardCountManager::new();
    let mut bit_prob: BackTrackCardCountManager<C> = BackTrackCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        // let private_player: usize = rng.gen_range(0..6);
        let private_player: usize = 0;
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            let mut rng = thread_rng();
            let card_0: u8 = rng.gen_range(0..5);
            let card_1: u8 = rng.gen_range(0..5);
            let cards = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
            // TODO: Fill those up
            prob.start_private(private_player, &cards);
            bit_prob.start_private(private_player, &cards);
        } else {
            prob.start_public();
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
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &prob, Some(private_player));
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
            if bool_know_priv_info && action_obs.player_id() == private_player {
                prob.push_ao_private(&action_obs);
                bit_prob.push_ao_private(&action_obs);
            } else {
                prob.push_ao_public(&action_obs);
                bit_prob.push_ao_public(&action_obs);
            }
            // if let Some(ai) = action_info {
            //     // TEMP fix for no DiscardMultiple
            //     // TODO: Change back to nice API
            //     if let ActionObservation::Discard { player_id, card, no_cards } = action_obs {
            //         if no_cards == 2 { 
            //             prob.push_ao_public(&action_obs);
            //             bit_prob.push_ao_public(&action_obs);
            //         } else {
            //             prob.push_ao(player, &ai);
            //             bit_prob.push_ao(player, &ai);
            //         }
            //     } else {
            //         prob.push_ao(player, &ai);
            //         bit_prob.push_ao(player, &ai);
            //     }
            // }
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
pub fn game_rnd_constraint_bt_st_debug<C>(game_no: usize, bool_know_priv_info: bool, min_dead_check: usize, log_bool: bool)
    where
        C: CoupConstraint + CoupConstraintAnalysis,
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob: BackTrackCardCountManager<C> = BackTrackCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        let mut rng = thread_rng();
        // let private_player: usize = rng.gen_range(0..6);
        let private_player: usize = 0;
        let mut rng = thread_rng();
        let card_0: u8 = rng.gen_range(0..5);
        let card_1: u8 = rng.gen_range(0..5);
        let starting_hand = [Card::try_from(card_0).unwrap(), Card::try_from(card_1).unwrap()];
           
        if bool_know_priv_info {
            // Choose random player
            // Initialize for that player
            prob.start_private(private_player, &starting_hand);
            bit_prob.start_private(private_player, &starting_hand);
        } else {
            prob.start_public();
            bit_prob.start_public();
        }
        while !hh.game_won() {
            hh.log_state();
            prob.printlog();
            bit_prob.printlog();
            // These are legal from a public sense only, but may be illegal depending on card 
            // TODO: [FIX] ExchangeChoice should generate regardless of hand as the card counter can determine it themselves
            new_moves = hh.generate_legal_moves(Some(private_player));
            // TODO: create generate_legal_moves(None) for public and private
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            // TODO: [FIX] generate_legal_moves_with_card_constraints to determine legal Choices given hand and ExchangeDraw
            let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &prob, Some(private_player));
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
            // if let Some(ai) = action_info {
            //     // TEMP fix for no DiscardMultiple
            //     // TODO: Change back to nice API
            //     if let ActionObservation::Discard { player_id, card, no_cards } = action_obs {
            //         if no_cards == 2 { 
            //             prob.push_ao_public(&action_obs);
            //             bit_prob.push_ao_public(&action_obs);
            //         } else {
            //             prob.push_ao(player, &ai);
            //             bit_prob.push_ao(player, &ai);
            //         }
            //     } else {
            //         prob.push_ao(player, &ai);
            //         bit_prob.push_ao(player, &ai);
            //     }
            // }
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
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt::<C>(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
                    panic!("Inferred constraints do not match!");
                    break;
                }
                if !pass_impossible_constraints {
                    // println!("vali: {:?}", validated_impossible_constraints);
                    // println!("test: {:?}", test_impossible_constraints);
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint_bt::<C>(replay, bool_know_priv_info, private_player, &starting_hand, log_bool);
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
pub fn generate_legal_moves_with_card_constraints(history: &History, new_moves: &mut Vec<ActionObservation>, prob: &BruteCardCountManagerGeneric<CardStateu64>, private_player: Option<usize>) -> Result<(usize, ActionObservation, Option<ActionInfo>), ()> {
    // Clone the moves and shuffle them in place
    new_moves.shuffle(&mut thread_rng());
    // This assumes all moves are by the same player
    // In the case of Challenge, it does not matter
    // if private_player.is_some() && new_moves[0].player_id() == private_player.unwrap() {
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
                ExchangeChoice { player_id, no_cards, hand, relinquish } => {
                    // TODO: [REFACTOR] need what was drawn and hand!
                    if private_player.is_some() && *player_id == private_player.unwrap() {
                        if let ExchangeDraw { card: draw, .. } = history.latest_move() {
                            if prob.player_can_have_cards_after_draw(*player_id, relinquish, draw) {
                                return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeChoice { hand: vec![], relinquish: relinquish.to_vec() })));
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
    // } else {
        // for candidate in new_moves.iter() {
        //     match candidate {
        //         Discard { player_id, card, no_cards } => {
        //             if *no_cards == 1 {
        //                 if prob.player_can_have_card_alive(*player_id, card[0]) {
        //                     return Ok((*player_id, *candidate, Some(ActionInfo::Discard { discard: card[0] })));
        //                 }
        //             } else {
        //                 if prob.player_can_have_cards(*player_id, candidate.cards()) {
        //                     // todo!("Make multiple Discard");
        //                     // return (candidate.player_id(), *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
        //                     return Ok((*player_id, *candidate, None));
        //                 }
        //                 log::info!("prob.player_can_have_cards({}, {:?}) = false", *player_id, candidate.cards());
        //             }
        //         },
        //         RevealRedraw { player_id, reveal: card , ..} => {
        //             if prob.player_can_have_card_alive(*player_id, *card) {
        //                 return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *card, redraw: None, relinquish: None })));
        //             }
        //         },
        //         ExchangeDraw { player_id, .. } => {
        //             return Ok((*player_id, *candidate, None));
        //         },
        //         ExchangeChoice { player_id, .. } => {
        //             // TODO: Change placeholder
        //             return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeDrawChoice { draw: vec![], relinquish: vec![] })));
        //         },
        //         _ => {
        //             return Ok((candidate.player_id(), *candidate, None));
        //         }
        //     }
        // }
    // }
    Err(())
}
// pub fn generate_legal_moves_with_card_constraints_g<C: CoupConstraint + CoupConstraintAnalysis>(new_moves: &mut Vec<ActionObservation>, prob: &BackTrackCardCountManager<C>, bool_know_priv_info: bool) -> Result<(usize, ActionObservation, Option<ActionInfo>), ()> {
//     // Clone the moves and shuffle them in place
//     new_moves.shuffle(&mut thread_rng());
//     if bool_know_priv_info {
//         for candidate in new_moves.iter() {
//             match candidate {
//                 Discard { player_id, card, no_cards } => {
//                     if *no_cards == 1 {
//                         if prob.player_can_have_card_alive(*player_id as u8, card[0]) {
//                             return Ok((*player_id, *candidate, Some(ActionInfo::Discard { discard: card[0] })));
//                         }
//                     } else {
//                         if prob.player_can_have_cards_alive(*player_id as u8, candidate.cards()) {
//                             // todo!("Make multiple Discard");
//                             // return (*player_id, *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
//                             return Ok((*player_id, *candidate, None));
//                         }
//                     }
//                 },
//                 RevealRedraw { player_id, reveal: reveal_card , redraw: redraw_card} => {
//                     if prob.player_can_have_card_alive(*player_id, *reveal_card) {
//                         if prob.player_can_have_card_alive(6, *redraw_card) {
//                             return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *reveal_card, redraw: Some(*redraw_card), relinquish: None } )));
//                         }
//                     }
//                 },
//                 ExchangeDraw { player_id, card } => {
//                     if prob.player_can_have_cards(6, candidate.cards()) {
//                         todo!("Add ExchangeDraw ActionInfo")
//                     }
//                 },
//                 ExchangeChoice { player_id, no_cards } => {
//                     todo!("Add ExchangeChoice ActionInfo")
//                 },
//                 _ => {
//                     return Ok((candidate.player_id(), *candidate, None));
//                 }
//             }
//         }
//     } else {
//         for candidate in new_moves.iter() {
//             match candidate {
//                 Discard { player_id, card, no_cards } => {
//                     if *no_cards == 1 {
//                         if prob.player_can_have_card_alive(*player_id, card[0]) {
//                             return Ok((*player_id, *candidate, Some(ActionInfo::Discard { discard: card[0] })));
//                         }
//                     } else {
//                         if prob.player_can_have_cards(*player_id, candidate.cards()) {
//                             // todo!("Make multiple Discard");
//                             // return (candidate.player_id(), *candidate, Some(ActionInfo::Discard { discard: candidate.cards()[0] }));
//                             return Ok((*player_id, *candidate, None));
//                         }
//                     }
//                 },
//                 RevealRedraw { player_id, reveal: card , ..} => {
//                     if prob.player_can_have_card_alive(*player_id, *card) {
//                         return Ok((*player_id, *candidate, Some(ActionInfo::RevealRedraw { reveal: *card, redraw: None, relinquish: None })));
//                     }
//                 },
//                 ExchangeDraw { player_id, .. } => {
//                     return Ok((*player_id, *candidate, None));
//                 },
//                 ExchangeChoice { player_id, no_cards } => {
//                     // TODO: Change placeholder
//                     return Ok((*player_id, *candidate, Some(ActionInfo::ExchangeDrawChoice { draw: vec![], relinquish: vec![] })));
//                 },
//                 _ => {
//                     return Ok((candidate.player_id(), *candidate, None));
//                 }
//             }
//         }
//     }
//     Err(())
// }
pub fn game_rnd_constraint_bt_st_g<V, T>(game_no: usize, bool_know_priv_info: bool, min_dead_check: usize, tx: Sender<Stats>)
    where
        V: CoupConstraint + CoupConstraintAnalysis,
        T: CoupConstraint + CoupConstraintAnalysis,
{
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    // let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    // let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraintLight> = BackTrackCardCountManager::new();
    let mut prob: BackTrackCardCountManager<V> = BackTrackCardCountManager::new();
    let mut bit_prob: BackTrackCardCountManager<T> = BackTrackCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut stats = Stats::new();
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        while !hh.game_won() {
            
            // hh.log_state();
            // prob.printlog();
            // bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            // new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        prob.player_can_have_card_alive(output.player_id() as u8, output.cards()[0])
                    } else {
                        prob.player_can_have_cards_alive(output.player_id() as u8, &output.cards().to_vec())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id() as u8, output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards_alive(6, &output.cards().to_vec());
                    if !true_legality {
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao_public(&output);
                bit_prob.push_ao_public(&output);
                let total_dead: usize = bit_prob.sorted_public_constraints().iter().map(|v| v.len()).sum();
                if total_dead >= min_dead_check {
                    let validated_public_constraints = prob.sorted_public_constraints().clone();
                    let validated_inferred_constraints = prob.sorted_inferred_constraints().clone();
                    let validated_impossible_constraints = prob.player_impossible_constraints().clone();
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
                    if bool_test_over_inferred {
                        // what we are testing inferred too many things
                        stats.over_inferred_count += 1;
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred to many items!")
                    }
                    if !pass_inferred_constraints {
                        println!("vali: {:?}", validated_inferred_constraints);
                        println!("vali: {:?}", test_inferred_constraints);
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!("Inferred constraints do not match!")
                    }
                    if !pass_impossible_constraints {
                        println!("vali: {:?}", validated_impossible_constraints);
                        println!("test: {:?}", test_impossible_constraints);
                        println!("{}", hh.get_replay_history_braindead());
                        break;
                        // let replay = hh.get_history(hh.store_len());
                        // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        // panic!()
                    }
                }


            } else {
                // log::trace!("Pushed bad move somewhere earlier!");
                stats.pushed_bad_move += 1;
                break;
            }
            // bit_prob.debug_panicker();
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
pub fn game_rnd_constraint(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool, min_dead_check: usize){
    // if log_bool{
    //     logger(LOG_LEVEL);
    // }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % print_frequency == 0 {
            println!("Game: {}", game);
            println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
            println!("Inferred Constraints Correct: {}/{}", inferred_constraints_correct, total_tries);
            println!("Inferred Constraints Overinferred: {}/{}", over_inferred_count, total_tries);
            println!("Impossible Cases Correct: {}/{}", impossible_constraints_correct, total_tries);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            prob.printlog();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao_public(&output);
                bit_prob.push_ao(&output, bool_know_priv_info);
                let total_dead: usize = bit_prob.latest_constraint().sorted_public_constraints().iter().map(|v| v.len()).sum();
                if total_dead >= min_dead_check {
                    let validated_public_constraints = prob.validated_public_constraints();
                    let validated_inferred_constraints = prob.validated_inferred_constraints();
                    let validated_impossible_constraints = prob.validated_impossible_constraints();
                    let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                    let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                    let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
                    let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                    let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                    let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                    let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                        test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                    });
                    // let pass_brute_prob_validity = prob.validate();
                    public_constraints_correct += pass_public_constraints as usize;
                    inferred_constraints_correct += pass_inferred_constraints as usize;
                    impossible_constraints_correct += pass_impossible_constraints as usize;
                    total_tries += 1;
                    if bool_test_over_inferred {
                        // what we are testing inferred too many things
                        over_inferred_count += 1;
                        break;
                        let replay = hh.get_history(hh.store_len());
                        replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        panic!("Inferred to many items!")
                    }
                    if !pass_inferred_constraints {
                        break;
                        let replay = hh.get_history(hh.store_len());
                        replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        panic!("Inferred constraints do not match!")
                    }
                    // if !pass_brute_prob_validity{
                    //     break;
                    //     let replay = hh.get_history(hh.store_len());
                    //     replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    //     panic!()
                    // }
                    if !pass_impossible_constraints {
                        break;
                        let replay = hh.get_history(hh.store_len());
                        replay_game_constraint(replay, bool_know_priv_info, log_bool);
                        panic!()
                    }
                }


            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            bit_prob.debug_panicker();
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
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn game_rnd_constraint_bt_generic_bench<C>(game_no : usize) 
    where
        C: CoupConstraint + CoupConstraintAnalysis,
{
    let mut game: usize = 0;
    let mut bit_prob: BackTrackCardCountManager<C> = BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let start_time = Instant::now();
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            new_moves = hh.generate_legal_moves(None);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        bit_prob.player_can_have_card_alive(output.player_id() as u8, output.cards()[0])
                    } else {
                        bit_prob.player_can_have_cards_alive(output.player_id() as u8, &output.cards().to_vec())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = bit_prob.player_can_have_card_alive(output.player_id() as u8, output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = bit_prob.player_can_have_cards_alive(6, &output.cards().to_vec());
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
    println!("Benchmark for: {}", std::any::type_name::<C>());
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!("Estimated Time per nodes: {} micro seconds", process_per_action_us);
}
pub fn game_rnd_constraint_bt_bench(game_no : usize) {
    let mut game: usize = 0;
    let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let mut start_time = Instant::now();
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        !bit_prob.player_impossible_constraints()[output.player_id()][output.cards()[0] as usize]
                    } else {
                        !bit_prob.player_impossible_constraints_paired()[output.player_id()][output.cards()[0] as usize][output.cards()[1] as usize]
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = !bit_prob.player_impossible_constraints()[output.player_id()][output.card() as usize];
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = !bit_prob.player_impossible_constraints_paired()[output.player_id()][output.cards()[0] as usize][output.cards()[1] as usize];
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
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!("Estimated Time per nodes: {} micro seconds", process_per_action_us);
}
pub fn game_rnd_constraint_brute_bench(game_no : usize) {
    let mut game: usize = 0;
    let mut bit_prob = BruteCardCountManager::new();
    let mut actions_processed: u128 = 0;
    let mut start_time = Instant::now();
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        bit_prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        bit_prob.player_can_have_cards(output.player_id(), output.cards())
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
                    let true_legality: bool = bit_prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                let validated_public_constraints = bit_prob.validated_public_constraints();
                let validated_inferred_constraints = bit_prob.validated_inferred_constraints();
                let validated_impossible_constraints = bit_prob.validated_impossible_constraints();
                hh.push_ao(output);
                bit_prob.push_ao(&output, false);
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
    println!("Games Ran: {}", game_no);
    println!("Nodes Processed: {}", actions_processed);
    println!("Estimated Time per nodes: {} micro seconds", process_per_action_us);
}
pub fn game_rnd_constraint_pd(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool, min_dead_check: usize){
    if log_bool{
        logger(LevelFilter::Warn);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut over_inferred_count: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if game % print_frequency == 0 {
            println!("Game: {}", game);
            println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
            println!("Inferred Constraints Correct: {}/{}", inferred_constraints_correct, total_tries);
            println!("Inferred Constraints Overinferred: {}/{}", over_inferred_count, total_tries);
            println!("Impossible Cases Correct: {}/{}", impossible_constraints_correct, total_tries);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            if log_bool {
                log::logger().flush();
                let _ = clear_log();
            }
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            prob.printlog();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                log::info!("{}", format!("Choice: {:?}", output));
                hh.push_ao(output);
                hh.print_replay_history_braindead();
                prob.push_ao_public(&output);
                bit_prob.push_ao_public(&output);
                let total_dead: usize = bit_prob.latest_constraint().sorted_public_constraints().iter().map(|v| v.len()).sum();
                if total_dead >= min_dead_check {
                    let validated_public_constraints = prob.validated_public_constraints();
                    let validated_inferred_constraints = prob.validated_inferred_constraints();
                    let validated_impossible_constraints = prob.validated_impossible_constraints();
                    let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                    let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                    let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
                    let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                    let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                    let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                    let bool_test_over_inferred: bool = validated_inferred_constraints.iter().zip(test_inferred_constraints.iter()).any(|(val, test)| {
                        test.iter().any(|item| !val.contains(item)) || test.len() > val.len()
                    });
                    // let pass_brute_prob_validity = prob.validate();
                    public_constraints_correct += pass_public_constraints as usize;
                    inferred_constraints_correct += pass_inferred_constraints as usize;
                    impossible_constraints_correct += pass_impossible_constraints as usize;
                    total_tries += 1;
                    if bool_test_over_inferred {
                        // what we are testing inferred too many things
                        over_inferred_count += 1;
                        break;
                        let replay = hh.get_history(hh.store_len());
                        replay_game_constraint_pd(replay, bool_know_priv_info, log_bool);
                        panic!("Inferred to many items!")
                    }
                    if !pass_inferred_constraints {
                        break;
                        let replay = hh.get_history(hh.store_len());
                        replay_game_constraint_pd(replay, bool_know_priv_info, log_bool);
                        panic!("Inferred constraints do not match!")
                    }
                    // if !pass_brute_prob_validity{
                    //     break;
                    //     let replay = hh.get_history(hh.store_len());
                    //     replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    //     panic!()
                    // }
                    // if !pass_impossible_constraints {
                    //     break;
                    //     let replay = hh.get_history(hh.store_len());
                    //     replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    //     panic!()
                    // }
                }


            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            // bit_prob.debug_panicker();
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
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn game_rnd_constraint_debug(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if log_bool {
            let _ = clear_log();
        }
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
            prob.printlog();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                log::info!("{}", format!("Choice: {:?}", output));
                hh.push_ao(output);
                hh.print_replay_history_braindead();
                prob.push_ao(&output, bool_know_priv_info);
                bit_prob.push_ao(&output, bool_know_priv_info);
                let validated_public_constraints = prob.validated_public_constraints();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.validated_impossible_constraints();
                let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_brute_prob_validity = prob.validate();
                if !pass_inferred_constraints {
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    panic!()
                }
                if !pass_brute_prob_validity{
                    let replay = hh.get_history(hh.store_len());
                    replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    panic!()
                }
                // bit_prob.check_three();
                public_constraints_correct += pass_public_constraints as usize;
                inferred_constraints_correct += pass_inferred_constraints as usize;
                impossible_constraints_correct += pass_impossible_constraints as usize;
                total_tries += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            bit_prob.debug_panicker();
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
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn game_rnd_constraint_debug_pd(game_no: usize, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut bool_know_priv_info = false;
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
    let mut bit_prob: BackTrackCardCountManager<BackTrackCollectiveConstraint> = BackTrackCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if log_bool {
            let _ = clear_log();
        }
        if game % print_frequency == 0 {
            println!("Game: {}", game);
            println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
            println!("Inferred Constraints Correct: {}/{}", inferred_constraints_correct, total_tries);
            println!("Impossible Cases Correct: {}/{}", impossible_constraints_correct, total_tries);
        }
        println!("Game: {}", game);
        log::trace!("Game Made:");
        while !hh.game_won() {
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            // prob.printlog();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                log::info!("{}", format!("Choice: {:?}", output));
                hh.push_ao(output);
                hh.print_replay_history_braindead();
                prob.push_ao(&output, bool_know_priv_info);
                bit_prob.push_ao_public(&output);
                let validated_public_constraints = prob.validated_public_constraints();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                // let validated_impossible_constraints = prob.validated_impossible_constraints();
                let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
                let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
                // let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                // let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_brute_prob_validity = prob.validate();
                if !pass_inferred_constraints {
                    let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!()
                }
                if !pass_brute_prob_validity{
                    let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    // panic!()
                }
                // bit_prob.check_three();
                public_constraints_correct += pass_public_constraints as usize;
                inferred_constraints_correct += pass_inferred_constraints as usize;
                // impossible_constraints_correct += pass_impossible_constraints as usize;
                total_tries += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            // bit_prob.debug_panicker();
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
        prob.reset();
        bit_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn game_rnd_constraint_debug_pd_alone(game_no: usize, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut bool_know_priv_info = false;
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
    let mut bit_prob = PathDependentCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        // if game % (game_no / 10) == 0 {
        if log_bool {
            let _ = clear_log();
        }
        if game % print_frequency == 0 {
            println!("Game: {}", game);
            println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
            println!("Inferred Constraints Correct: {}/{}", inferred_constraints_correct, total_tries);
            println!("Impossible Cases Correct: {}/{}", impossible_constraints_correct, total_tries);
        }
        println!("Game: {}", game);
        log::trace!("Game Made:");
        while !hh.game_won() {
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            // prob.printlog();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                log::info!("{}", format!("Choice: {:?}", output));
                hh.push_ao(output);
                prob.push_ao(&output, false);
                hh.print_replay_history_braindead();
                bit_prob.push_ao_public(&output);
                // impossible_constraints_correct += pass_impossible_constraints as usize;
                total_tries += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            bit_prob.debug_panicker();
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
        bit_prob.reset();
        prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}
pub fn replay_game_constraint(replay: Vec<ActionObservation>, bool_know_priv_info: bool, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    log::info!("REPLAY ID");
    log::info!("vec!{:?};", replay);
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    if log_bool {
        clear_log().expect("failed to clear log");
    }
    log::info!("Game : {}", game);
    let mut hh = History::new(0);
    let mut step: usize = 0;
    let mut new_moves: Vec<ActionObservation>;
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
        new_moves = hh.generate_legal_moves(None);
        log::info!("{}", format!("Legal Moves: {:?}", new_moves));
        log::info!("{}", format!("Legal Moves Retained: {:?}", new_moves));
        if new_moves[0].name() != AOName::CollectiveChallenge {
            // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
        } else {
            // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
            log::info!("{}", format!("Legal Moves: CollectiveChallenge"));
        }
        
        if let Some(output_ref) = replay.get(step) {
            let output = output_ref.clone();
            log::info!("{}", format!("Choice: {:?}", output));
            hh.print_history();
            if output.name() == AOName::Discard{
                let true_legality = if output.no_cards() == 1 {
                    // let start_time = Instant::now();
                    prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                } else {
                    prob.player_can_have_cards(output.player_id(), output.cards())
                };
                if !true_legality{
                    log::info!("Illegal Move, Ending Game");
                    break    
                } 
            } else if output.name() == AOName::RevealRedraw {
                let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                if !true_legality{
                    log::info!("Illegal Move, Ending Game");
                    break    
                } 
            } else if output.name() == AOName::ExchangeDraw {
                let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                if !true_legality {
                    log::info!("Illegal Move, Ending Game");
                    break    
                }
            } 
            hh.push_ao(output);
            prob.push_ao_public(&output);
            bit_prob.push_ao(&output, bool_know_priv_info);
            match output.name() {
                AOName::RevealRedraw | 
                AOName::Discard | AOName::ExchangeDraw | AOName::Exchange => {
                    // prob.print_legal_states();
                },
                _ => {},
            }
            log::info!("Just before validation");
            bit_prob.printlog();
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
                false => {
                    "FAILED"
                },
            });
            log::info!("validated_impossible_constraints: {:?}", validated_impossible_constraints);
            log::info!("test_impossible_constraints: {:?}", test_impossible_constraints);
            let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
            log::info!("impossible_constraints: {}", match pass_impossible_constraints {
                true => "PASSED",
                false => "FAILED",
            });
            // let pass_brute_prob_validity = prob.validate();
            // if !pass_brute_prob_validity {
            //     log::info!("Brute Prob Public Constraint Validity: FAILED");
            // } else {
            //     log::info!("Brute Prob Public Constraint Validity: PASSED");
            // }
            if !pass_inferred_constraints {
                prob.print_legal_states();
            }
            if !pass_inferred_constraints {
                hh.print_replay_history_braindead();
                panic!()
            }
            // if !pass_brute_prob_validity{
            //     hh.print_replay_history_braindead();
            //     panic!()
            // }
            // bit_prob.check_three();
            public_constraints_correct += pass_public_constraints as usize;
            inferred_constraints_correct += pass_inferred_constraints as usize;
            impossible_constraints_correct += pass_impossible_constraints as usize;
            total_tries += 1;
        } else {
            log::trace!("End of Replay!");
            return;
        }
        bit_prob.debug_panicker();
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
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}

pub fn replay_game_constraint_pd(replay: Vec<ActionObservation>, bool_know_priv_info: bool, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    log::info!("REPLAY ID");
    log::info!("vec!{:?};", replay);
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob = PathDependentCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    if log_bool {
        clear_log().expect("failed to clear log");
    }
    log::info!("Game : {}", game);
    let mut hh = History::new(0);
    let mut step: usize = 0;
    let mut new_moves: Vec<ActionObservation>;
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
        new_moves = hh.generate_legal_moves(None);
        log::info!("{}", format!("Legal Moves: {:?}", new_moves));
        log::info!("{}", format!("Legal Moves Retained: {:?}", new_moves));
        if new_moves[0].name() != AOName::CollectiveChallenge {
            // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
        } else {
            // log::info!("{}", format!("Legal Moves: {:?}", new_moves));
            log::info!("{}", format!("Legal Moves: CollectiveChallenge"));
        }
        
        if let Some(output_ref) = replay.get(step) {
            let output = output_ref.clone();
            log::info!("{}", format!("Choice: {:?}", output));
            if output.name() == AOName::Discard{
                let true_legality = if output.no_cards() == 1 {
                    // let start_time = Instant::now();
                    prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                } else {
                    prob.player_can_have_cards(output.player_id(), output.cards())
                };
                if !true_legality{
                    log::info!("Illegal Move, Ending Game");
                    break    
                } 
            } else if output.name() == AOName::RevealRedraw {
                let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                if !true_legality{
                    log::info!("Illegal Move, Ending Game");
                    break    
                } 
            } else if output.name() == AOName::ExchangeDraw {
                let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                if !true_legality {
                    log::info!("Illegal Move, Ending Game");
                    break    
                }
            } 
            hh.push_ao(output);
            hh.print_replay_history_braindead();
            prob.push_ao_public(&output);
            bit_prob.push_ao_public(&output);
            match output.name() {
                AOName::RevealRedraw | 
                AOName::Discard | AOName::ExchangeDraw | AOName::Exchange => {
                    // prob.print_legal_states();
                },
                _ => {},
            }
            log::info!("Just before validation");
            bit_prob.printlog();
            let validated_public_constraints = prob.validated_public_constraints();
            let validated_inferred_constraints = prob.validated_inferred_constraints();
            let validated_impossible_constraints = prob.validated_impossible_constraints();
            let test_public_constraints = bit_prob.latest_constraint().sorted_public_constraints();
            let test_inferred_constraints = bit_prob.latest_constraint().sorted_inferred_constraints();
            let test_impossible_constraints = bit_prob.latest_constraint_mut().generate_one_card_impossibilities_player_card_indexing();
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
            log::info!("validated_impossible_constraints: {:?}", validated_impossible_constraints);
            log::info!("test_impossible_constraints: {:?}", test_impossible_constraints);
            let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
            log::info!("impossible_constraints: {}", match pass_impossible_constraints {
                true => "PASSED",
                false => "FAILED",
            });
            // let pass_brute_prob_validity = prob.validate();
            // if !pass_brute_prob_validity {
            //     log::info!("Brute Prob Public Constraint Validity: FAILED");
            // } else {
            //     log::info!("Brute Prob Public Constraint Validity: PASSED");
            // }
            if !pass_inferred_constraints {
                prob.print_legal_states();
                hh.print_replay_history_braindead();
                panic!()
            }
            // if !pass_brute_prob_validity{
            //     hh.print_replay_history_braindead();
            //     panic!()
            // }
            // bit_prob.check_three();
            public_constraints_correct += pass_public_constraints as usize;
            inferred_constraints_correct += pass_inferred_constraints as usize;
            impossible_constraints_correct += pass_impossible_constraints as usize;
            total_tries += 1;
        } else {
            log::trace!("End of Replay!");
            println!("=> no issues");
            return;
        }
        bit_prob.debug_panicker();
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
pub fn replay_game_constraint_bt<C>(replay: Vec<ActionObservation>, bool_know_priv_info: bool, private_player: usize, starting_hand: &[Card; 2], log_bool: bool)
    where
        C: CoupConstraint + CoupConstraintAnalysis,
{
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut bit_prob: BackTrackCardCountManager<C> = BackTrackCardCountManager::new();
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
        prob.start_public();
        bit_prob.start_public();
    }
    while !hh.game_won() {
        
        // log::info!("{}", format!("Dist_from_turn: {:?}",hh.get_dist_from_turn(step)));
        // log::info!("{}", format!("History: {:?}",hh.get_history(step)));
        new_moves = hh.generate_legal_moves(None);
        let result = generate_legal_moves_with_card_constraints(&hh, &mut new_moves, &prob, None);
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
            prob.set_impossible_constraints_2();
            prob.set_impossible_constraints_3();
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

pub fn game_rnd(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
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
            new_moves = hh.generate_legal_moves(None);
            let test_impossible_constraints = bit_prob.latest_constraint().generate_one_card_impossibilities_player_card_indexing();
            log::info!("Impossible cards before choosing move: {:?}", test_impossible_constraints);
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
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        log::info!("Illegal Move, Ending Game");
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
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
pub fn temp_test() {
    // TODO: Test minor functions
    // TODO: test and check how brute is used for all push_ao and shit
    logger(LOG_LEVEL);
    let mut brute_prob = BruteCardCountManager::new();
    brute_prob.printlog();

    brute_prob.add_public_constraint(0, Card::Ambassador);
    brute_prob.add_public_constraint(0, Card::Assassin);
    brute_prob.restrict(0, vec!['A', 'B']);
    brute_prob.update_constraints();
    brute_prob.printlog();

    brute_prob.add_public_constraint(1, Card::Captain);
    brute_prob.add_public_constraint(1, Card::Duke);
    brute_prob.restrict(1, vec!['C', 'D']);
    brute_prob.update_constraints();
    brute_prob.printlog();
    
    brute_prob.add_public_constraint(3, Card::Assassin);
    brute_prob.restrict(3, vec!['B']);
    brute_prob.update_constraints();
    brute_prob.printlog();

    brute_prob.reveal_redraw(2, 'B');
    brute_prob.update_constraints();
    brute_prob.printlog();

    let can_amb: bool = brute_prob.player_can_have_card(2, Card::Ambassador);
    let can_ass: bool = brute_prob.player_can_have_card(2, Card::Assassin);
    let can_cap: bool = brute_prob.player_can_have_card(2, Card::Captain);
    let can_duk: bool = brute_prob.player_can_have_card(2, Card::Duke);
    let can_con: bool = brute_prob.player_can_have_card(2, Card::Contessa);
    log::info!("A: {can_amb}, B: {can_ass}, C: {can_cap}, D: {can_duk}, E: {can_con}");
    let can_0: bool = brute_prob.player_can_have_cards(2, &vec![Card::Ambassador, Card::Ambassador]);
    let can_1: bool = brute_prob.player_can_have_cards(2, &vec![Card::Ambassador, Card::Assassin]);
    let can_2: bool = brute_prob.player_can_have_cards(2, &vec![Card::Ambassador, Card::Captain]);
    let can_3: bool = brute_prob.player_can_have_cards(2, &vec![Card::Ambassador, Card::Duke]);
    let can_4: bool = brute_prob.player_can_have_cards(2, &vec![Card::Ambassador, Card::Contessa]);
    let can_5: bool = brute_prob.player_can_have_cards(2, &vec![Card::Assassin, Card::Assassin]);
    let can_6: bool = brute_prob.player_can_have_cards(2, &vec![Card::Assassin, Card::Captain]);
    let can_7: bool = brute_prob.player_can_have_cards(2, &vec![Card::Assassin, Card::Duke]);
    let can_8: bool = brute_prob.player_can_have_cards(2, &vec![Card::Assassin, Card::Contessa]);
    let can_9: bool = brute_prob.player_can_have_cards(2, &vec![Card::Captain, Card::Captain]);
    let can_10: bool = brute_prob.player_can_have_cards(2, &vec![Card::Captain, Card::Duke]);
    let can_11: bool = brute_prob.player_can_have_cards(2, &vec![Card::Captain, Card::Contessa]);
    let can_12: bool = brute_prob.player_can_have_cards(2, &vec![Card::Duke, Card::Duke]);
    let can_13: bool = brute_prob.player_can_have_cards(2, &vec![Card::Duke, Card::Contessa]);
    let can_14: bool = brute_prob.player_can_have_cards(2, &vec![Card::Contessa, Card::Contessa]);
    log::info!("AA: {can_0}, AB: {can_1}, AC: {can_2}, AD: {can_3}, AE: {can_4}, BB: {can_5}, BC: {can_6}, BD: {can_7}, BE: {can_8}, CC: {can_9}, CD: {can_10}, CE: {can_11}, DD: {can_12}, DE: {can_13}, EE: {can_14}");
}
pub fn instant_delete() {
}
// pub fn logger(level: LevelFilter){
//     // let log_file = File::create("app.log").unwrap();

//     let log_file = File::create("card_count_validation.log").expect("Failed to create log file");

//     // Initialize the env_logger builder with custom format
//     Builder::from_env(Env::default().default_filter_or("info"))
//         .format(|buf, record| {
//             // Custom format: Timestamp, Level, and Message
//             writeln!(
//                 buf,
//                 "{} [{}] - {}",
//                 chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
//                 record.level(),
//                 record.args()
//             )
//         })
//         // Set log level filter; this line is optional if using default_filter_or in from_env
//         // .filter(None, LevelFilter::Trace) // Adjust the log level as needed
//         .filter(None, level) // Adjust the log level as needed
//         // Direct logs to the file
//         .target(Target::Pipe(Box::new(log_file)))
//         // Apply the configuration
//         .init();
// }
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

fn test_brute(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool) {
    if log_bool{
        logger(LOG_LEVEL);
    }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
    let mut test_prob: BruteCardCountManagerGeneric<CardStateu64> = BruteCardCountManagerGeneric::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    while game < game_no {
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
            prob.printlog();
            test_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            // new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                log::info!("Choice: {:?}", output);
                if output.name() == AOName::Discard{
                    let true_legality = if output.no_cards() == 1 {
                        // let start_time = Instant::now();
                        prob.player_can_have_card_alive(output.player_id(), output.cards()[0])
                    } else {
                        prob.player_can_have_cards(output.player_id(), output.cards())
                    };
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::RevealRedraw {
                    let true_legality: bool = prob.player_can_have_card_alive(output.player_id(), output.card());
                    if !true_legality{
                        break    
                    } 
                } else if output.name() == AOName::ExchangeDraw {
                    let true_legality: bool = prob.player_can_have_cards(6, output.cards());
                    if !true_legality {
                        break    
                    }
                } 
                hh.push_ao(output);
                prob.push_ao(&output, bool_know_priv_info);
                test_prob.push_ao_public(&output);
                let validated_public_constraints = prob.validated_public_constraints();
                let validated_inferred_constraints = prob.validated_inferred_constraints();
                let validated_impossible_constraints = prob.validated_impossible_constraints();
                let test_public_constraints = test_prob.validated_public_constraints();
                let test_inferred_constraints = test_prob.validated_inferred_constraints();
                let test_impossible_constraints = test_prob.validated_impossible_constraints();
                log::info!("=== Prob ===");
                prob.printlog();
                prob.print_legal_states();
                log::info!("=== Test Prob ===");
                test_prob.printlog();
                test_prob.print_legal_states();
                let pass_public_constraints: bool = validated_public_constraints == test_public_constraints;
                let pass_inferred_constraints: bool = validated_inferred_constraints == test_inferred_constraints;
                let pass_impossible_constraints: bool = validated_impossible_constraints == test_impossible_constraints;
                let pass_brute_prob_validity = prob.validate();
                if !pass_inferred_constraints {
                    let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    panic!()
                }
                if !pass_brute_prob_validity{
                    let replay = hh.get_history(hh.store_len());
                    // replay_game_constraint(replay, bool_know_priv_info, log_bool);
                    panic!()
                }
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
        hh.print_replay_history_braindead();
        prob.reset();
        test_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Public Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Inferred Constraints Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Impossible Cases Correct: {}/{}", public_constraints_correct, total_tries);
    println!("Total Tries: {}", total_tries);
}

pub fn speed(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    // if log_bool{
    //     logger(LOG_LEVEL);
    // }
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut bit_prob = BitCardCountManager::new();
    let mut total_tries: usize = 0;
    while game < game_no {
        let mut hh = History::new(0);
        let mut step: usize = 0;
        let mut new_moves: Vec<ActionObservation>;
        if game % print_frequency == 0 {
            println!("Game: {}", game);
        }
        log::trace!("Game Made:");
        while !hh.game_won() {
            
            // log::info!("{}", format!("Step : {:?}",step));
            hh.log_state();
            bit_prob.printlog();
            new_moves = hh.generate_legal_moves(None);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw && m.name() != AOName::Exchange);
            // new_moves.retain(|m| m.name() != AOName::RevealRedraw);
            // new_moves.retain(|m| m.name() != AOName::Exchange);
            
            if let Some(output) = new_moves.choose(&mut thread_rng()).cloned(){
                hh.push_ao(output);
                bit_prob.push_ao(&output, bool_know_priv_info);
                total_tries += 1;
            } else {
                log::trace!("Pushed bad move somewhere earlier!");
                break;
            }
            bit_prob.debug_panicker();
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
        bit_prob.reset();
        game += 1;
    }
    println!("Most Steps: {}", max_steps);
    println!("Total Tries: {}", total_tries);
}