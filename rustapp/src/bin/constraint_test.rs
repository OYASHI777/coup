
use rustapp::prob_manager::constraint::{CompressedGroupConstraint};
use rustapp::history_public::Card;
fn main() {
    for player in 0..6 {

        println!("Player: {player}");
        let mut test = CompressedGroupConstraint::new(player, Card::Contessa, 0, 0);
        test.update_total_count();
        // for i in 0..7 {
        //     println!("P{i}: {:?}", test.get_player_flag(i));
        // }
        println!("Flags: {:?}", test.get_set_players());
        println!("Card: {:?}", test.get_card());
        println!("Dead Count: {:?}", test.get_dead_count());
        println!("Alive Count: {:?}", test.get_alive_count());
        println!("Total Count: {:?}", test.get_total_count());
    }
}