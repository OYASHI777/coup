
use rustapp::prob_manager::bitconstraint::{CompressedGroupConstraint};
use rustapp::history_public::Card;
fn main() {
    // for player in 0..6 {

        // println!("Player: {player}");
        // let mut test = CompressedGroupConstraint::new(player, Card::Contessa, 0, 0);
        // let mut test = CompressedGroupConstraint::new_list([false, false, true, false, true, false, false], Card::Contessa, 2, 1);
        // let mut test = CompressedGroupConstraint::new_bit(0b01001100, Card::Contessa, 2, 1);
        let mut test = CompressedGroupConstraint::new_bit(0b0101_1100, Card::Duke, 1, 2);
        let mut group = CompressedGroupConstraint::new_bit(0b0101_1100, Card::Captain, 2, 1);
        // test.update_total_count();
        println!("Before:");
        for i in 0..7 {
            println!("P{i}: {:?}", test.get_player_flag(i));
        }
        println!("Flags: {:?}", test.get_set_players());
        println!("Card: {:?}", test.get_card());
        println!("Dead Count: {:?}", test.count_dead());
        println!("Alive Count: {:?}", test.count_alive());
        println!("Total Count: {:?}", test.count());

        // test.group_add_list([true, false, false, true, false ,false, true]);
        // test.group_subtract(1);

        println!("Subset: {}", test.is_subset_of(&group));
        println!("Part list subset: {}", test.part_list_is_subset_of(&group));
        println!("Mut Excl: {}", test.part_list_is_mut_excl(&group));
    // }
}