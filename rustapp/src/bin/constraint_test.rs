
use rustapp::prob_manager::bitconstraint::{CompressedGroupConstraint};
use rustapp::history_public::Card;
fn main() {
    // for player in 0..6 {

        // println!("Player: {player}");
        // let mut test = CompressedGroupConstraint::new(player, Card::Contessa, 0, 0);
        // let mut test = CompressedGroupConstraint::new_list([false, false, true, false, true, false, false], Card::Contessa, 2, 1);
        println!("{}", 0b01001100 as u16);
        println!("{}", 0b10000000 as u16);
        let mut test = CompressedGroupConstraint::new_bit(0b01001100, Card::Contessa, 2, 1);
        // test.update_total_count();
        for i in 0..7 {
            println!("P{i}: {:?}", test.get_player_flag(i));
        }
        println!("Flags: {:?}", test.get_set_players());
        println!("Card: {:?}", test.get_card());
        println!("Dead Count: {:?}", test.get_dead_count());
        println!("Alive Count: {:?}", test.get_alive_count());
        println!("Total Count: {:?}", test.get_total_count());
    // }
}