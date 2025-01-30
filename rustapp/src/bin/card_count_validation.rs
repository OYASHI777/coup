use log::LevelFilter;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rustapp::history_public::{AOName, ActionObservation, Card, History};
use rustapp::prob_manager::compressed_group_constraint::{CompressedGroupConstraint};
use rustapp::prob_manager::collective_constraint::{CompressedCollectiveConstraint};
use rustapp::prob_manager::brute_prob::BruteCardCountManager;
use rustapp::prob_manager::bit_prob::BitCardCountManager;
use std::fs::{File, OpenOptions};
use std::io::{Write};
use env_logger::{Builder, Env, Target};
pub const LOG_LEVEL: LevelFilter = LevelFilter::Info;
pub const LOG_FILE_NAME: &str = "full_test.log";
// CURRENT BUG: add_subset_group never adds => check all redundant checks => to reconsider what really is redundant
// ANOTHER BUG: ok even if nothing is added, why on earth does it keep panicking
// ANOTHER BUG: 0 dead 0 alive groups are possible for some reason
// ANOTHER BUG: weird all cards [1 1 1 1 1 1 0] 3 at start of game
// ANOTHER BUG: groups_constraints can be empty even if all dead, but needs at least 1 3 dead set.. 3 dead is not redundant
// FIX: adding single group of 3 is ok in the case of pile
fn main() {
    let game_no = 100000;
    let log_bool = true;
    let bool_know_priv_info = false;
    let print_frequency: usize = 10;
    // (DONE) [TEST 1000] Discard + Ambassador Release farm
    // [TEST 1000] Discard + RevealRedraw Release mode
    // (Ran 210) [TEST 1000] Discard + Ambassador Debug mode
    // [TEST 1000] Discard Debug mode
    // [TEST 1000] Discard + RevealRedraw Debug mode
    // [Running] Discard + Ambassador Debug mode
    // [Passed 1100] Discard + Ambassador Release farm
    // game_rnd_constraint(game_no, bool_know_priv_info, print_frequency, log_bool);
    {
        use ActionObservation::*;
        use Card::*;
        // Please test out different redundancies
        let replay = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, card: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }];
        replay_game_constraint(replay, bool_know_priv_info, log_bool);
    }
    // game_rnd(game_no, bool_know_priv_info, print_frequency, log_bool);
    // temp_test_brute();
    // instant_delete();
    test();
}
pub fn test() {
    {
        use ActionObservation::*;
        use Card::*;
        let reveal_redraw_issue_0 = vec![ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, true, true], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, RevealRedraw { player_id: 4, card: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 1, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, card: Captain }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }];
        let reveal_redraw_replay_1 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 1, final_actioner: 0 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, RevealRedraw { player_id: 2, card: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 2, card: Ambassador }, CollectiveChallenge { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 4, final_actioner: 5 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 1 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, card: Duke }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 0, final_actioner: 1 }, RevealRedraw { player_id: 0, card: Duke }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 2, opposing_player_id: 0 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 2, final_actioner: 0 }, RevealRedraw { player_id: 2, card: Assassin }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }];
        let reveal_redraw_replay_2 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 1, final_actioner: 1 }, Steal { player_id: 2, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, true], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, true, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, false, false, false, false], opposing_player_id: 4, final_actioner: 4 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Steal { player_id: 2, opposing_player_id: 5, amount: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, card: Duke }, Discard { player_id: 3, card: [Captain, Captain], no_cards: 1 }];
        let full_test_replay_0 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Duke, Duke], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 5 }, CollectiveBlock { participants: [true, false, false, true, true, false], opposing_player_id: 5, final_actioner: 3 }, CollectiveChallenge { participants: [true, false, false, false, true, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 2, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, RevealRedraw { player_id: 2, card: Captain }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }];
        let full_test_replay_1 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, true], opposing_player_id: 2, final_actioner: 3 }, RevealRedraw { player_id: 2, card: Duke }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, RevealRedraw { player_id: 4, card: Captain }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 0, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [false, false, false, false, false, false], opposing_player_id: 0, final_actioner: 0 }, Steal { player_id: 5, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 2, opposing_player_id: 4 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, false, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [false, false, false, false, true, true], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 1 }, Assassinate { player_id: 4, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }];
        let full_test_replay_2 = vec![Steal { player_id: 0, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, false, true, true, true], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 2, final_actioner: 0 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [true, true, true, false, true, false], opposing_player_id: 3, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 0, final_actioner: 4 }, RevealRedraw { player_id: 0, card: Duke }, Discard { player_id: 4, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [true, false, true, true, false, false], opposing_player_id: 4, final_actioner: 0 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 5, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, false], opposing_player_id: 5, final_actioner: 3 }, Discard { player_id: 5, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, true, true, false, false], opposing_player_id: 4, final_actioner: 1 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, ForeignAid { player_id: 3 }, CollectiveBlock { participants: [false, true, true, false, false, false], opposing_player_id: 3, final_actioner: 2 }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 2, final_actioner: 1 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 3 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }];
        let full_test_replay_3 = vec![Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, true, false, true, false, false], opposing_player_id: 0, final_actioner: 3 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, true, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 4, card: Captain }, Steal { player_id: 2, opposing_player_id: 4, amount: 0 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 2, final_actioner: 5 }, Discard { player_id: 2, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, true, true], opposing_player_id: 3, final_actioner: 2 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, card: Captain }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 4, card: Captain }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, true, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Contessa, Contessa], no_cards: 1 }, Income { player_id: 2 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 0, opposing_player_id: 1 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 0, final_actioner: 1 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Assassinate { player_id: 1, opposing_player_id: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }];
        let full_test_replay_4 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [false, false, true, false, false, false], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, BlockSteal { player_id: 3, opposing_player_id: 3, card: Captain }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, false, false, true, false, true], opposing_player_id: 2, final_actioner: 5 }, CollectiveChallenge { participants: [true, false, true, true, true, false], opposing_player_id: 5, final_actioner: 4 }, RevealRedraw { player_id: 5, card: Duke }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 4, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, true, false, false, true], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Income { player_id: 0 }, Assassinate { player_id: 1, opposing_player_id: 0 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Tax { player_id: 2 }, CollectiveChallenge { participants: [true, true, false, true, false, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Captain, Captain], no_cards: 1 }, Tax { player_id: 3 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 3, final_actioner: 5 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 5 }, Steal { player_id: 0, opposing_player_id: 1, amount: 1 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 1 }, Tax { player_id: 5 }, CollectiveChallenge { participants: [true, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, Discard { player_id: 5, card: [Captain, Captain], no_cards: 1 }];
        let redundancy_replay_0 = vec![Tax { player_id: 0 }, CollectiveChallenge { participants: [false, true, true, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Captain, Captain], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [false, false, true, true, false, true], opposing_player_id: 1, final_actioner: 3 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 1, card: Captain }, CollectiveChallenge { participants: [false, true, false, false, false, false], opposing_player_id: 5, final_actioner: 1 }, RevealRedraw { player_id: 5, card: Captain }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, ForeignAid { player_id: 2 }, CollectiveBlock { participants: [false, true, false, false, false, true], opposing_player_id: 2, final_actioner: 1 }, CollectiveChallenge { participants: [false, false, true, true, true, true], opposing_player_id: 1, final_actioner: 2 }, RevealRedraw { player_id: 1, card: Duke }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, false, false, true, true], opposing_player_id: 3, final_actioner: 1 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Income { player_id: 4 }, Steal { player_id: 5, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [false, true, true, false, false, false], opposing_player_id: 5, final_actioner: 2 }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [false, false, false, false, false, true], opposing_player_id: 0, final_actioner: 5 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Tax { player_id: 1 }, CollectiveChallenge { participants: [false, false, false, false, true, false], opposing_player_id: 1, final_actioner: 4 }, Discard { player_id: 1, card: [Assassin, Assassin], no_cards: 1 }];
        let full_test_overflow_0 = vec![Income { player_id: 0 }, Steal { player_id: 1, opposing_player_id: 5, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, false, true], opposing_player_id: 1, final_actioner: 5 }, RevealRedraw { player_id: 1, card: Captain }, Discard { player_id: 5, card: [Ambassador, Ambassador], no_cards: 1 }, BlockSteal { player_id: 5, opposing_player_id: 5, card: Captain }, Income { player_id: 2 }, Steal { player_id: 3, opposing_player_id: 4, amount: 2 }, CollectiveChallenge { participants: [true, true, true, false, false, false], opposing_player_id: 3, final_actioner: 1 }, RevealRedraw { player_id: 3, card: Captain }, Discard { player_id: 1, card: [Duke, Duke], no_cards: 1 }, BlockSteal { player_id: 4, opposing_player_id: 3, card: Captain }, CollectiveChallenge { participants: [false, false, false, true, false, true], opposing_player_id: 4, final_actioner: 5 }, RevealRedraw { player_id: 4, card: Captain }, Discard { player_id: 5, card: [Contessa, Contessa], no_cards: 1 }, ForeignAid { player_id: 4 }, CollectiveBlock { participants: [false, false, true, true, false, false], opposing_player_id: 4, final_actioner: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 2, final_actioner: 3 }, Discard { player_id: 2, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 0, opposing_player_id: 1, amount: 2 }, CollectiveChallenge { participants: [false, true, true, true, true, false], opposing_player_id: 0, final_actioner: 2 }, Discard { player_id: 0, card: [Contessa, Contessa], no_cards: 1 }, Steal { player_id: 1, opposing_player_id: 2, amount: 2 }, CollectiveChallenge { participants: [true, false, false, true, true, false], opposing_player_id: 1, final_actioner: 3 }, Discard { player_id: 1, card: [Ambassador, Ambassador], no_cards: 1 }, Steal { player_id: 2, opposing_player_id: 3, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 2, final_actioner: 4 }, Discard { player_id: 2, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 3, final_actioner: 4 }, Discard { player_id: 3, card: [Assassin, Assassin], no_cards: 1 }, Income { player_id: 4 }, ForeignAid { player_id: 0 }, CollectiveBlock { participants: [false, false, false, false, true, false], opposing_player_id: 0, final_actioner: 4 }, CollectiveChallenge { participants: [false, false, false, true, false, false], opposing_player_id: 4, final_actioner: 3 }, Discard { player_id: 4, card: [Assassin, Assassin], no_cards: 1 }, Steal { player_id: 3, opposing_player_id: 0, amount: 2 }, CollectiveChallenge { participants: [true, false, false, false, true, false], opposing_player_id: 3, final_actioner: 0 }, Discard { player_id: 3, card: [Duke, Duke], no_cards: 1 }, Tax { player_id: 4 }, CollectiveChallenge { participants: [true, false, false, false, false, false], opposing_player_id: 4, final_actioner: 0 }, Discard { player_id: 4, card: [Ambassador, Ambassador], no_cards: 1 }];
        let mut count = 0;
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(reveal_redraw_issue_0, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(reveal_redraw_replay_1, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(reveal_redraw_replay_2, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(full_test_replay_0, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(full_test_replay_2, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(full_test_replay_3, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(full_test_replay_4, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(redundancy_replay_0, false, false);
        println!("Testing: {}", count); count += 1;
        replay_game_constraint(full_test_overflow_0, false, false);
        println!("ALL PASSED");
    }
}
pub fn game_rnd_constraint(game_no: usize, bool_know_priv_info: bool, print_frequency: usize, log_bool: bool){
    // if log_bool{
    //     logger(LOG_LEVEL);
    // }
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
            new_moves = hh.generate_legal_moves();
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
pub fn replay_game_constraint(replay: Vec<ActionObservation>, bool_know_priv_info: bool, log_bool: bool){
    if log_bool{
        logger(LOG_LEVEL);
    }
    log::info!("REPLAY ID");
    log::info!("vec!{:?};", replay);
    let mut game: usize = 0;
    let mut max_steps: usize = 0;
    let mut prob = BruteCardCountManager::new();
    let mut bit_prob = BitCardCountManager::new();
    let mut public_constraints_correct: usize = 0;
    let mut inferred_constraints_correct: usize = 0;
    let mut impossible_constraints_correct: usize = 0;
    let mut total_tries: usize = 0;
    let game_no = 1;
    let print_frequency = 1;
    while game < game_no {
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
            new_moves = hh.generate_legal_moves();
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
                prob.push_ao(&output, bool_know_priv_info);
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
                let pass_brute_prob_validity = prob.validate();
                if !pass_brute_prob_validity {
                    log::info!("Brute Prob Public Constraint Validity: FAILED");
                } else {
                    log::info!("Brute Prob Public Constraint Validity: PASSED");
                }
                if !pass_inferred_constraints {
                    prob.print_legal_states();
                }
                if !pass_inferred_constraints {
                    hh.print_replay_history_braindead();
                    panic!()
                }
                if !pass_brute_prob_validity{
                    hh.print_replay_history_braindead();
                    panic!()
                }
                bit_prob.check_three();
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
        log::info!("{}", format!("Game Won : {:?}",step));
        hh.print_replay_history_braindead();
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
            new_moves = hh.generate_legal_moves();
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
pub fn temp_test_brute() {
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