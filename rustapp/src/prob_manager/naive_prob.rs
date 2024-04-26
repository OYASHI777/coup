// Journey here
// Tried to iteratively find naive probability by filtering
// Concurrent and normal iteration times are around 0.1 s calculation of belief is around 0.1 seconds
// This is too long
// Tried instead to save into hashmap and store in bson

use crate::history_public::{Card, AOName, ActionObservation};
use super::permutation_generator::{gen_table_combinations, gen_bag_combinations};
use super::coup_const::{BAG_SIZES, TOKENS, MAX_PERM_STATES};
use std::collections::{HashMap, HashSet};
// use core::hash::Hasher;
use std::hash::{Hash, Hasher};
use std::usize;
use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use serde_json;
use super::loader::{load_initial_hashmap, save_bson_hashmap};
use rand::prelude::SliceRandom;
use std::sync::Mutex;
use core::sync::atomic::AtomicBool;
#[derive(Clone, Debug, Eq)]
pub struct GroupConstraint {
    // String not &str as it will not be known at compile time
    // Either 0 or 1, indicates if a particular index is part of a group that shares a card
    // e.g. [1,1,0,0,0,0,0] indicates that in the union of hands 0 and hand 1 collectively contain some card
    participation_list: [u8; 7],
    card: Card,
    count_dead: usize,
    count_alive: usize,
    count: usize,
}

impl GroupConstraint {
    pub fn new(player_id: usize, card: Card, count_dead: usize, count_alive: usize) -> Self {
        // Default includes the pile (player_id = 7) because all player mixes are with the pile and not with other players
        let participation_list: [u8; 7] = [0, 0, 0, 0, 0, 0, 1];
        let count: usize = count_dead + count_alive;
        let mut output = GroupConstraint{
            participation_list,
            card,
            count_dead,
            count_alive,
            count,
        };
        output.group_add(player_id);
        output
    }
    pub fn new_list(participation_list: [u8; 7], card: Card, count_dead: usize, count_alive: usize) -> Self {
        let count: usize = count_dead + count_alive;
        GroupConstraint{
            participation_list,
            card,
            count_dead,
            count_alive,
            count,
        }
    }
    pub fn group_add(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        self.participation_list[player_id] = 1;
    }
    pub fn group_add_list(&mut self, list: [u8; 7]){
        self.participation_list = list;
    }
    pub fn group_subtract(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        self.participation_list[player_id] = 0;
    }
    pub fn count_add(&mut self, num: usize){
        self.count += num;
    }
    pub fn count_dead_add(&mut self, num: usize){
        self.count_dead += num;
        self.count += num;
    }
    pub fn count_alive_add(&mut self, num: usize){
        self.count_alive += num;
        self.count += num;
    }
    pub fn count_subtract(&mut self, num: usize){
        if self.count < num {
            self.count = 0;
        } else {
            self.count -= num;
        }
    }
    pub fn count_dead_subtract(&mut self, num: usize){
        if self.count_dead < num {
            self.count_dead = 0;
        } else {
            self.count_dead -= num;
            self.count -= num;
        }
    }
    pub fn count_alive_subtract(&mut self, num: usize){
        if self.count_alive < num {
            self.count_alive = 0;
        } else {
            self.count_alive -= num;
            self.count -= num;
        }
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn count_dead(&self) -> usize {
        self.count_dead
    }
    pub fn count_alive(&self) -> usize {
        self.count_alive
    }
    pub fn indicator(&self, player_id : usize) -> u8 {
        self.participation_list[player_id]
    }
    pub fn all_in(&self) -> bool {
        let sum: u8 = self.participation_list.iter().sum();
        if sum == 7 {
            true
        } else {
            false
        }
    }
    pub fn get_list(&self) -> &[u8; 7]{
        &self.participation_list
    }
    pub fn card(&self) -> &Card {
        &self.card
    } 
    pub fn is_subset_of(&self, group: &Self) -> bool {
        // Returns true if group makes self redundant
        // Its redundant if they have the same participation list and their counts are equal
        // Its also redundant if they have the same participation list and group has a higher count
        //      If a group has at least 2 Dukes, it also fulfils having at least 1 Duke.
        //      Therefore having at least 1 Duke is redundant
        if self.card() != group.card(){
            return false;
        }
        // if self.get_list() == group.get_list() && self.count() <= group.count() {
        if self.get_list() == group.get_list() && self.count() <= group.count() && self.count_dead() <= group.count_dead() && self.count_alive() <= group.count_alive() {
            true
        } else {
            false
        }
    }
    pub fn part_list_is_subset_of(&self, group: &Self) -> bool {
        // Checks if self participation list is a subset of group's participation list
        // This means that Any 1 in self list must have a 1 in group's list
        GroupConstraint::list1_is_subset_list2(&self.get_list(), &group.get_list())
    }
    pub fn list1_is_subset_list2(list1: &[u8; 7], list2: &[u8; 7]) -> bool {
        // Checks if list1 is subset of list2
        let mut index: usize = 0;
        while index < list1.len(){
            if list1[index] == 1 && list2[index] == 0 {
                    return false;
                }
            index += 1;
        }
        true
    }
    pub fn part_list_is_mut_excl(&self, group: &Self) -> bool {
        // Checks if the groups are mutually exclusive
        GroupConstraint::lists_are_mut_excl(self.get_list(), group.get_list())
    }
    pub fn lists_are_mut_excl(list1: &[u8; 7], list2: &[u8; 7]) -> bool {
        let mut index: usize = 0;
        while index < list1.len(){
            if list1[index] == 1 && list2[index] == 1{
                return false;
            } 
            index += 1;
        }
        true
    }
    pub fn list_union<'a>(list1: &'a mut [u8; 7], list2: &[u8; 7]) -> &'a [u8; 7] {
        let mut index: usize = 0;
        while index < list1.len() {
            if list1[index] == 0 && list2[index] == 1 {
                list1[index] = 1;
            }
            index += 1;
        }
        list1
    }
    pub fn printlog(&self) {
        log::trace!("{}", format!("Participation List: {:?}", self.participation_list));
        log::trace!("{}", format!("Card: {:?}", self.card));
    }
}
impl PartialEq for GroupConstraint {
    fn eq(&self, other: &Self) -> bool {
        self.participation_list == other.participation_list && self.card == other.card && self.count == other.count && self.count_dead == other.count_dead && self.count_alive == other.count_alive
    }
}
impl Hash for GroupConstraint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.participation_list.hash(state);
        self.card.hash(state);
        self.count.hash(state);
    }
}
#[derive(Clone)]
pub struct CollectiveConstraint {
    // pc => PublicConstraint => Player i has card A
    // jc => JointConstraint => Player i has card AB
    // gc => GroupConstraint
    pc_hm: HashMap<usize, Card>,
    jc_hm: HashMap<usize, Vec<Card>>,
    //TODO:
    gc_vec: Vec<GroupConstraint>,
    dead_card_count: HashMap<Card, u8>,

}
impl CollectiveConstraint{
    pub fn new() -> Self {
        let pc_hm: HashMap<usize, Card> = HashMap::new();
        let jc_hm: HashMap<usize, Vec<Card>> = HashMap::new();
        let gc_vec = Vec::new();

        let mut dead_card_count: HashMap<Card, u8> = HashMap::new();
        dead_card_count.insert(Card::Ambassador, 0);
        dead_card_count.insert(Card::Assassin, 0);
        dead_card_count.insert(Card::Captain, 0);
        dead_card_count.insert(Card::Duke, 0);
        dead_card_count.insert(Card::Contessa, 0);

        CollectiveConstraint{
            pc_hm,
            jc_hm,
            gc_vec,
            dead_card_count,

        }
    }
    pub fn is_empty(&self) -> bool {
        if self.pc_hm.is_empty() && self.jc_hm.is_empty() && self.gc_vec.is_empty(){
            true
        } else {
            false
        }
    }
    pub fn is_complement_of_pcjc(&self, group: &GroupConstraint) -> bool{
        // Tells us if information in a group is mutually exclusive from information in pc_hm and jc_hm
        let mut indicators: [u8; 7] = group.get_list().clone();
        for (key, value) in &self.pc_hm {
            if value == group.card() {
                indicators[*key] = 1;
            }
        }
        for (key, value) in &self.jc_hm {
            if value.contains(group.card()) {
                indicators[*key] = 1;
            }
        }
        if indicators.iter().sum::<u8>() == 7 {
            true
        } else {
            false
        }
    }
    pub fn jc_hm(&self) -> &HashMap<usize, Vec<Card>> {
        &self.jc_hm
    }
    pub fn pc_hm(&self) -> &HashMap<usize, Card> {
        &self.pc_hm
    }
    pub fn gc_vec(&self) -> &Vec<GroupConstraint>{
        &self.gc_vec
    }
    pub fn add_raw_public_constraint(&mut self, player_id: usize, card: Card){
        // This is useful for testing whether a player can have a particular card
        // This does not prune groups 
        // Therefore its useful for determining if a player can have a card
        // It does this by adding the constraint without pruning
        // So the check becomes, if we add this constraint, can the old constraints still be fulfilled?
        if let Some(value) = self.dead_card_count.get_mut(&card){
            *value += 1;
        }
        if let Some(pc_hm_card) = self.pc_hm.remove(&player_id){
            // If pc_hm has a card already, remove it and add to jc_hm
            debug_assert!(!self.jc_hm.contains_key(&player_id), "jc_hm also contains key! Should not happen!");
            let mut card_vec: Vec<Card> = vec![pc_hm_card, card];
            card_vec.sort();
            self.jc_hm.insert(player_id, card_vec.clone());
            
        } else {
            self.pc_hm.insert(player_id, card);
        }
        // [Test] - is this needed for add public constraints?
        let mut i: usize = 0;
        while i < self.gc_vec.len(){
            let group = &mut self.gc_vec[i];
            if *group.card() == card && group.get_list()[player_id] == 1 {
                group.count_dead_add(1);
                group.count_alive_subtract(1);
                if group.count_alive() == 0 {
                    self.gc_vec.swap_remove(i);
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

    }
    pub fn add_raw_group(&mut self, group: GroupConstraint){
        self.gc_vec.push(group);
    }
    // Technically can remove too but its not in Coup so..
    pub fn add_public_constraint(&mut self, player_id: usize, card: Card){
        if let Some(value) = self.dead_card_count.get_mut(&card){
            *value += 1;
        }
        if let Some(pc_hm_card) = self.pc_hm.remove(&player_id){
            // If pc_hm has a card already, remove it and add to jc_hm
            debug_assert!(!self.jc_hm.contains_key(&player_id), "jc_hm also contains key! Should not happen!");
            let mut card_vec: Vec<Card> = vec![pc_hm_card, card];
            card_vec.sort();
            self.jc_hm.insert(player_id, card_vec.clone());

            self.group_dead_player_prune(player_id, &card_vec);            

        } else {
            self.pc_hm.insert(player_id, card);

            let mut index = 0;
            while index < self.gc_vec.len(){
                let group: &mut GroupConstraint = &mut self.gc_vec[index];
                // if group.indicator(player_id) == 1 && group.count() == 1{
                //     // Public Constraint is subset of group so prune group because group will definitely be fulfilled
                //     // [SUBSET PRUNE]
                //     self.gc_vec.swap_remove(index);
                // } else if self.dead_card_count[group.card()] == 3 {
                if self.dead_card_count[group.card()] == 3 {
                    // [DEAD PRUNE] Prune group if all cards have been shown dead for some card. There are only 3 of each card
                    self.gc_vec.swap_remove(index);
                } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                    // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                    self.gc_vec.swap_remove(index);
                } else {
                    index += 1;
                }
            }
        }
        // self.add_inferred_groups();
        self.group_redundant_prune();
    }
    pub fn remove_public_constraint(&mut self, player_id: usize, card: Card){
        // self.gc_initial_hm.entry(card).and_modify(|arr| arr[player_id] = 0);
        if let Some(pc_hm_card) = self.pc_hm.get(&player_id){
            // card in pc_hm?
            if *pc_hm_card == card {
                self.pc_hm.remove(&player_id);
            } else {
                debug_assert!(false, "Removing card constraint that does not exist in pc_hm");
            }
        } else if let Some(jc_hm_vec) = self.jc_hm.get_mut(&player_id){
            // Gets joint constraint in jc_hm
            // Removes it and adds remainder to pc_hm
            if let Some(index) = jc_hm_vec.iter().position(|c| *c == card){
                jc_hm_vec.remove(index);
                if jc_hm_vec.len() == 1 {
                    if let Some(remaining_card) = jc_hm_vec.pop(){
                        self.pc_hm.insert(player_id, remaining_card);
                        self.jc_hm.remove(&player_id);
                    } else {
                        debug_assert!(false, "No remaining card found in jc_hm_vec!");
                    }
                } else {
                    debug_assert!(false, "jc_hm_vec len was too long!");
                }
            } else {
                debug_assert!(false, "Removing card constraint that does not exist in jc_hm");
            }
        }
    }
    pub fn add_joint_constraint(&mut self, player_id: usize, card_vec: &Vec<Card>){
        debug_assert!(self.pc_hm.get(&player_id).is_none(), "Player already half dead, how to die again??");
        debug_assert!(self.jc_hm.get(&player_id).is_none(), "Player already dead, how to die again??");
        for card in card_vec.iter() {
            if let Some(value) = self.dead_card_count.get_mut(card){
                *value += 1;
            }
        }
        self.jc_hm.insert(player_id, card_vec.clone());
        // Pruning
        // self.add_inferred_groups();
        self.group_dead_player_prune(player_id, card_vec);
    }
    // You will never remove from group only pop to reverse it because you cannot unmix cards
    pub fn group_initial_prune(&mut self, player_id: usize, card: &Card, count: usize, bool_card_dead: bool){
        // Pruning for revealredraw or exchangedraw
        // Modifies dead_count and alive_count of groups that have player indicator == 1 given the knowledge of a card being dead
        // bool_cards_dead should only be true for discard and not revealredraw not exchangedraw
        // Should be true if card revealed is dead, and false if card revealed is eventually reshuffled or alive
        // count here is the number of cards revealed
        debug_assert!(player_id <= 6, "Player ID Wrong");
        // new_count is the total dead cards in hand + cards revealed
        let mut new_count: usize = count;
        // Considering the case where another card is dead, adds to card count for pruning
        if let Some(card_dead) = self.pc_hm.get(&player_id) {
            if card_dead == card {
                // RevealRedraw Case
                // Increase the count if player who revealed the card already has a dead card that is the same!
                // Player & Pile will have at least 2 of the card
                new_count += 1;
            }
        } else if let Some(cards) = self.jc_hm.get(&player_id){
            // Should not be able to revealredraw or exchangedraw if jc_hm has player as the player would be dead!
            debug_assert!(false, "Impossible Case Reached");
        }
        let mut index: usize = 0;
        while index < self.gc_vec.len() {
            // [INITIAL PRUNE] if player 0 revealredraws a Duke, we prune the groups that had player 0 duke and oldcount <= newcount
            // This special case is for the original ambassadar version of coup only you might need to initial prune for the inquisitor
            let group = &mut self.gc_vec[index];
            // if group.card() == card && group.get_list()[player_id] == 1 && group.count() <= count{

            if group.card() == card && group.get_list()[player_id] == 1 && group.count() <= new_count{
                // Prune If same card and is subset if old group count <= revealed count
                // Because the group is redundant
                log::trace!("Initial Prune Remove");
                self.gc_vec.swap_remove(index);
            } else if group.card() == card && group.get_list()[player_id] == 1 && group.count() > new_count && bool_card_dead{
                // Adjusting figures appropriately to reflect number alive in the group and number dead
                // In the case of Discard Groups are updated to reflect the value
                // group.count() has to be > new_count else the group would have to be pruned as all information would be represented by public_constraint
                log::trace!("initial prune add subtract");
                log::trace!("Group: {:?}", group);
                group.count_dead_add(count);
                group.count_alive_subtract(count);
                index += 1;
            } else {
                index += 1;
            }
        }
        // if its one dead one alive?
    }
    pub fn group_dead_player_prune(&mut self, player_id: usize, card_vec: &Vec<Card>){
        // Group initial prune should have been used before this, so the dead counts should be correct in group
        // Prunes relevant groups in gc_vec when player loses all their cards
        log::info!("DEAD PLAYER PRUNE");
        let mut index: usize = 0;
        let mut bool_subtract: bool = false;
        while index < self.gc_vec.len(){
            let group: &mut GroupConstraint = &mut self.gc_vec[index];

            if group.indicator(player_id) == 1 {
                if group.card() != &card_vec[0] && group.card() != &card_vec[1]{
                    // Modify Group whose cards are different!
                    // Their card will not be in the indicator because it is full!
                    log::trace!("SUBTRACT NO COUNT");
                    group.group_subtract(player_id);
                    bool_subtract = true;
                } else if &card_vec[0] == &card_vec[1]{
                    if group.card() == &card_vec[0] {
                        log::trace!("SUBTRACT 2");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(2);
                        bool_subtract = true;
                    }
                    if group.count() == 0 {
                        log::trace!("Unexpected 0 A");
                    }
                } else {
                    if group.card() == &card_vec[0] {
                        log::trace!("SUBTRACT 1");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(1);
                        bool_subtract = true;
                    }else if group.card() == &card_vec[1] {
                        log::trace!("SUBTRACT 1");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(1);
                        bool_subtract = true;
                    }
                    if group.count() == 0 {
                        log::trace!("Unexpected 0 B");
                    }
                }
                // Other cases PRUNED in Initial PRUNE
            }
            if group.count_alive() == 0{
                // This part is technically redundant as this case should be handled in group_initial_prune
                // No longer handled in group_initial_prune
                self.gc_vec.swap_remove(index);
            } else if self.dead_card_count[group.card()] == 3 {
                // [DEAD PRUNE] Prune group if all cards have been shown dead for some card. There are only 3 of each card
                self.gc_vec.swap_remove(index);
            } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                self.gc_vec.swap_remove(index);
            } else {
                index += 1;
            }
        }
        self.group_redundant_prune();
    }
    pub fn group_redundant_prune(&mut self){
        // Loops through gc_vec and removes redundant groups
        if self.gc_vec.len() >= 2 {
            let mut i: usize = 0;
            let mut j: usize;
            let mut i_incremented: bool;
            while i < self.gc_vec.len() - 1 {
                j = i + 1;
                i_incremented = false;
                while j < self.gc_vec.len() {
                    let group_i = &self.gc_vec[i];
                    let group_j = &self.gc_vec[j];
                    if self.is_redundant(group_i, group_j){
                        // group i is redundant
                        log::trace!("Redundant Group i < j i= {:?}", group_i);
                        log::trace!("Redundant Group i < j j= {:?}", group_j);
                        self.gc_vec.swap_remove(i);
                        i_incremented  = true;
                        break;
                    } else if self.is_redundant(group_j, group_i) {
                        // group j is redundant
                        log::trace!("Redundant Group j < i i= {:?}", group_i);
                        log::trace!("Redundant Group j < i j= {:?}", group_j);
                        self.gc_vec.swap_remove(j);
                    } else {
                        j += 1;
                    }
                }
                if !i_incremented {
                    i += 1;
                }
            }
        }
    }
    pub fn is_redundant(&self, group1: &GroupConstraint, group2: &GroupConstraint) -> bool {
        // Returns true if GROUP 1 is redundant and false if not
        if group1.is_subset_of(group2){
            return true;
        }
        if group1.card() != group2.card(){
            return false;
        }
        if group1.count_alive() == 0 {
            return true;
        }
        let card: &Card = group1.card();
        let remaining_count: usize = 3 - self.dead_card_count[card] as usize;

        if group1.count_alive() == remaining_count && group2.count_alive() == remaining_count {
        // if group1.count() == remaining_count && group2.count() == remaining_count {
            // Keep the subset as it is smaller!
            if group2.part_list_is_subset_of(group1) {
                // e.g. where group 2 part list is subset of group 1 part list
                // Group 1 [0 1 0 0 1 0 1] : count = remaining_count
                // Group 2 [0 1 0 0 0 0 1] : count = remaining_count
                // We know both conditions must be true
                // All remaining cards are within the set denoted by group 1 and the set denoted by group 2
                // For both conditions to be true, player 4 must not have the card
                // Therefore group 1 is redundant
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn add_group_constraint(&mut self, player_id: usize, card: &Card, count: usize){
        // Adds a new constraint used for RevealRedraw
        // There is an [INITIAL PRUNE] because we know player had a card before the swap!
        // let mut new_count: usize = count;
        let mut dead_count: usize = 0;
        if let Some(card_dead) = self.pc_hm.get(&player_id) {
            if card_dead == card {
                // RevealRedraw Case
                // Increase the count if player who revealed the card already has a dead card that is the same!
                // Player & Pile will have at least 2 of the card
                // new_count += 1;
                dead_count += 1;
            }
        } else if !self.jc_hm.get(&player_id).is_none(){
            // Should not be able to revealredraw or exchangedraw if jc_hm has player as the player would be dead!
            debug_assert!(false, "Impossible Case Reached");
        }
        let mut index: usize = 0;
        // Iterate and update current groups
        while index < self.gc_vec.len() {

            // RevealRedraw Version
            let group = &mut self.gc_vec[index];
            if group.get_list()[player_id] == 0 {
                group.group_add(player_id);
                // Add one because the new player shows he has a card and swaps it into the pile & his hand
                // So the past group + new player has an additional card
                
                
                // Old Group: Duke [0 1 0 0 0 0 1] Count 1
                // Move RevealRedraw a Duke is revealed and shuffled into pile
                // Add Group: Duke [0 0 1 0 0 0 1] Count 1
                // Intuitively there will be 1 Duke at first with player 2 now shuffled among himself and the pile
                // And 1 Duke that was originally with Player 1 & pile
                // After RevealRedraw there are in total 2 Dukes among player 1,2 and Pile
                // So we must increment the old counter by new_count
                if group.card() == card {
                    // log::trace!("RevealRedraw add group 1");
                    group.count_alive_add(count);
                    group.count_dead_add(dead_count);
                    if group.count() > 3 {
                        // Its not possible to reveal a card that the player should not even be able to have!
                        // debug_assert!(group.count() <= 3, "Impossible case reached!");
                        log::trace!("GROUP COUNT HIGH! FIX WITH LEGAL MOVE PRUNE");
                    }
                } else {
                    if let Some(card_dead) = self.pc_hm.get(&player_id) { 
                        if card_dead == group.card() {
                            // log::trace!("reveal redraw dead card addition");
                            group.count_dead_add(1);
                        }
                    }
                }
                if group.all_in() == true {
                    // [FULL PRUNE] because group constraint just means there could be a Duke anywhere (anyone or the pile might have it)
                    self.gc_vec.swap_remove(index);
                } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                    // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                    self.gc_vec.swap_remove(index);
                } else {
                    index += 1;
                }
            } else {
                index += 1;
            }
        }
        let addition: GroupConstraint = GroupConstraint::new(player_id, *card, dead_count, count);
        if !self.is_complement_of_pcjc(&addition) {
            // [COMPLEMENT PRUNE] We push only if it isnt a complement
            self.gc_vec.push(addition);
        }
        // self.add_inferred_groups();
        self.group_redundant_prune();
    }
    pub fn add_group_constraint_exchange(&mut self, player_id: usize, card: &Card, count: usize){
        // Adds a new constraint used for exchangedraw
        // Here Some Pile cards are revealed before a player chooses whether they want it
        // There is an [INITIAL PRUNE] because we know player had a card before the swap!
        // let mut new_count: usize = count;
        let mut dead_count: usize = 0;
        if let Some(card_dead) = self.pc_hm.get(&player_id) {
            if card_dead == card {
                // RevealRedraw Case
                // Increase the count if player who revealed the card already has a dead card that is the same!
                // Player & Pile will have at least 2 of the card
                // new_count += 1;
                dead_count += 1;
            }
        } else if !self.jc_hm.get(&player_id).is_none(){
            // Should not be able to revealredraw or exchangedraw if jc_hm has player as the player would be dead!
            debug_assert!(false, "Impossible Case Reached");
        }
        let mut index: usize = 0;
        // Iterate and update current groups
        while index < self.gc_vec.len() {
            // New Version
            let group = &mut self.gc_vec[index];
            if group.card() == card {
                if group.get_list()[6] == 1{
                    // Add one because the new player shows he has a card and swaps it into the pile & his hand
                    // So the past group + new player has an additional card
                    // group is now mixed based on the swapping move
                    // [0 0 0 0 0 0 1] -> [0 0 0 1 0 0 1]
                    // [0 0 0 1 0 0 1] -> [0 0 0 1 0 0 1]
                    if group.get_list()[player_id] == 0 {
                        group.group_add(player_id);
                        group.count_dead_add(dead_count);
                        log::trace!("ExchangeDraw add group 1");
                    } 
                    if group.count_alive() < count {
                        log::trace!("ExchangeDraw add alive 1");
                        group.count_alive_add(count - group.count_alive());
                    }
                    // Exchange draw count is handled differently because pile is the card that is revealed
                    // if group.count() < new_count {
                        // group.count_add(new_count - group.count());
                        // }
                    debug_assert!(group.count() < 4, "GROUP COUNT IN EXCHANGEDRAW TOO HIGH");
                } 
            } else {
                if group.get_list()[6] == 1{
                    if group.get_list()[player_id] == 0 {
                        log::trace!("Exchange constraint do nothing");
                    }
                    // Just mixes the set
                    // Because this is private information, if this group considered is not tracking the card that was drawn
                    // We know that the group.card() could not have been swapped to player unless they drew the group.card() from the pile in which case
                    // group.card() == *card
                    // So we do nothing
                    // if group.get_list()[player_id] == 0{
                    //     group.group_add(player_id);
                    //     if let Some(dead_card) = self.pc_hm.get(&player_id) {
                    //         if dead_card == group.card() {
                    //             // Is this already done in initial prune?
                    //             group.group_add(player_id);
                    //             group.count_dead_add(1);
                    //             log::trace!("ExchangeDraw add group 2");
                    //         }
                    //     }
                    // }
                }
            }
            if group.all_in() == true {
                // [FULL PRUNE] because group constraint just means there could be a Duke anywhere (anyone or the pile might have it)
                self.gc_vec.swap_remove(index);
            } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                self.gc_vec.swap_remove(index);
            } else {
                index += 1;
            }
        }
        // let addition: GroupConstraint = GroupConstraint::new(player_id, *card, new_count);
        let addition: GroupConstraint = GroupConstraint::new(player_id, *card, dead_count, count);
        if !self.is_complement_of_pcjc(&addition) {
            // [COMPLEMENT PRUNE] We push only if it isnt a complement
            self.gc_vec.push(addition);
        }
        // self.add_inferred_groups();
        self.group_redundant_prune();
    }

    pub fn add_inferred_groups(&mut self) {
        // e.g. Player 1 has 1 life left
        // Duke [0 1 0 0 0 0 1] : count = 2
        // If player 1's dead card is not Duke
        // We know that there must be at least 1 Duke in the Pile
        // Duke [0 0 0 0 0 0 1] : count = 1
        // This functions adds all possible of such groups in!
        let mut index: usize = 0;
        let mut union_part_list_checked: Vec<[u8; 7]> = Vec::new();
        let mut union_group_counts: HashMap<Card, usize> = HashMap::new();
        let mut union_group_card_sets: HashMap<Card, [u8; 7]> = HashMap::new();
        while index < self.gc_vec.len() {
            let group_count: usize = self.gc_vec[index].count();
            let group_list: [u8; 7] = self.gc_vec[index].get_list().clone();
            let group_card: Card = self.gc_vec[index].card().clone();
            if group_count > 1 {
                for (player_id, indicator) in group_list.iter().enumerate() {
                    // Skipping player_id 6 because its the pile and pile always has 3 lives same as max number of a card
                    if player_id == 6 {
                        continue;
                    }
                    if *indicator == 1 {
                        // The maximum amount of a card the player may hold
                        let mut max_possible_holdings: usize;
                        let mut dead_count: usize = 0;
                        if let Some(dead_card) = self.pc_hm.get(&player_id) {
                            // Player has 1 Life

                            if *dead_card == group_card {
                                max_possible_holdings = 2;
                                dead_count += 1;
                            } else {
                                max_possible_holdings = 1;
                            }

                        } else if let Some(dead_card_vec) = self.jc_hm.get(&player_id) {
                            debug_assert!(false, "This should not happen");
                            // I have implemented this for extensibility options
                            max_possible_holdings = 0;
                            for card in dead_card_vec.iter() {
                                if *card == group_card {
                                    max_possible_holdings += 1;
                                    dead_count += 1;
                                }
                            }
                        } else {
                            // Player has 2 lives
                            max_possible_holdings = 2;
                        }
                        if max_possible_holdings < group_count {
                            let mut inferred_group: GroupConstraint = self.gc_vec[index].clone();
                            log::trace!("INFERRED GROUP CLONED!: {:?}", inferred_group);
                            inferred_group.group_subtract(player_id);
                            // inferred_group.count_subtract(max_possible_holdings);
                            inferred_group.count_dead_subtract(dead_count);
                            inferred_group.count_alive_subtract(max_possible_holdings - dead_count);
                            log::trace!("INFERRED GROUP ADDED!: {:?}", inferred_group);
                            self.gc_vec.push(inferred_group);
                        }
                    }
                }
            }
            // This second part adds:
            // Adding discovery of sets via subsets
            // Checking if subset has been checked already
            // if union_part_list_checked.contains(self.gc_vec[index].get_list()) {
            //     index += 1;
            //     continue;
            // } else {
            //     union_part_list_checked.push(self.gc_vec[index].get_list().clone());
            // }
            // // Resetting Array
            // for icard in [Card::Ambassador, Card::Assassin ,Card::Captain, Card::Duke, Card::Contessa].iter(){
            //     if let Some(value) = union_group_counts.get_mut(icard){
            //         *value = 0;
            //     }
            //     if let Some(arr) = union_group_card_sets.get_mut(icard) {
            //         *arr = [0; 7];
            //     }
            // }
            // let mut total_union_group_count: usize = 0;
            // let mut full_count: usize = 0;
            // // Filling up HashMap with dead cards on ALIVE players
            // for (player_id, indicator) in self.gc_vec[index].get_list().iter().enumerate() {
            //     if *indicator == 1 {
            //         if let Some(card_value) = self.pc_hm.get(&player_id) {
            //             if let Some(count_value) = union_group_counts.get_mut(card_value){
            //                 *count_value += 1;
            //                 total_union_group_count += 1;
            //             }
            //             // Filling up union_group_card_sets
            //             if let Some(arr) = union_group_card_sets.get_mut(card_value) {
            //                 arr[player_id] = 1;
            //             }
            //         }
            //         if player_id != 6 {
            //             full_count += 2;
            //         } else {
            //             full_count += 3;
            //         }
            //     }
            // }
            // // Filling up union_part_list and union_part_count
            // let mut j: usize = 0;
            // while j < self.gc_vec.len() {
            //     if self.gc_vec[j].part_list_is_subset_of(&self.gc_vec[index]) {
            //         // update based on whether mutually exclusive or not if ME add count else make it higher of both
            //         if let Some(union_part_list) = union_group_card_sets.get_mut(&self.gc_vec[j].card()) {
            //             let mut bool_mutual_exclusive: bool = true;
            //             for icheck in 0..7 {
            //                 if union_part_list[icheck] != self.gc_vec[j].get_list()[icheck]{
            //                     bool_mutual_exclusive = false;
            //                     break;
            //                 }
            //             }
            //             if let Some(count_value) = union_group_counts.get_mut(&self.gc_vec[j].card()) {
            //                 if bool_mutual_exclusive {
            //                     *count_value += self.gc_vec[j].count();
            //                     total_union_group_count += self.gc_vec[j].count();
            //                 } else {
            //                     if self.gc_vec[j].count() > *count_value {
            //                         *count_value = self.gc_vec[j].count();
            //                         total_union_group_count += self.gc_vec[j].count() - *count_value;
            //                     }
            //                 }
            //             }

            //             for iupdate in 0..7{
            //                 if union_part_list[iupdate] == 0 && self.gc_vec[j].get_list()[iupdate] == 1 {
            //                     union_part_list[iupdate] = 1;
            //                 }
            //             }

            //         }
            //     }
            //     j += 1;
            // }
            // // Evaluating new information based on union_part_list and union_part_count
            // // Inferring for indicator == 1 and indicator == 0
            // // Alive cards remaining for indicator == 0 is 3 - union_part_count - all jc_hm cards
            // // Check if full
            // // Update list to be the union of new group and all old combined groups
            // // find list that is complement and players are alive add remaining possible cards into list
            // if total_union_group_count == full_count {
            //     // Convert to cards left for groups that indicators are 0 aka the complement group
            //     log::trace!("union_group_counts: {:?}", union_group_counts);
            //     for value in union_group_counts.values_mut(){
            //         *value = 3 - *value;
            //     }
            //     // Subtracting all the dead cards on dead players
            //     for card_vec in self.jc_hm.values() {
            //         for vcard in card_vec.iter(){
            //             if let Some(value) = union_group_counts.get_mut(vcard) {
            //                 *value -= 1;
            //             }
            //         }
            //     }
            //     // [Test 1]
            //     // union_group_counts are now the counts of cards that can be held in the alive players within the complement group
            //     // That can be dead cards and alive cards
            //     let mut complement_group: [u8; 7] = [0; 7];
            //     for (player_id, val) in complement_group.iter_mut().enumerate() {
            //         if self.gc_vec[index].get_list()[player_id] == 0 {
            //             *val = 1;
            //         }
            //     }
            //     // Adding information that is new
            //     for (vcard, vcount) in union_group_counts.iter() {
            //         if *vcount > 0 {
            //             self.gc_vec.push(GroupConstraint::new_list(complement_group.clone(), *vcard, *vcount));
            //         }
            //     }
            //     // [Test 2] Find all groups that the non complement group is a subset of take the subtraction and see what card they can have
            //     // Say we know group [1 0 0 0 0 0 1] has 2 Ambassadors 1 Capt 1 Duke 1 Cont
            //     // Say we know group [1 1 0 0 0 0 1] has 3 Ambassadors
            //     // Then we know [0 1 0 0 0 0 0] has at least 1 ambassador
            //     let mut igroup: usize = 0;
            //     while igroup < self.gc_vec.len() {
            //         if GroupConstraint::list1_is_subset_list2(&self.gc_vec[index].get_list(), self.gc_vec[igroup].get_list()) {
            //             if union_group_counts[self.gc_vec[igroup].card()] > self.gc_vec[igroup].count() {
            //                 let mut new_info: [u8; 7] = [0; 7];
            //                 for i in 0..7 {
            //                     if self.gc_vec[index].get_list()[i] == 0 && self.gc_vec[igroup].get_list()[i] == 1{
            //                         new_info[i] = 1;
            //                     }
            //                 }
            //                 self.gc_vec.push(GroupConstraint::new_list(new_info, *self.gc_vec[igroup].card(), union_group_counts[self.gc_vec[igroup].card()] - self.gc_vec[igroup].count()));
            //             }

            //         }
            //         igroup += 1;
            //     }
            // }
            index += 1;
        }
        // Unsure if need to prune?
        log::trace!("Ended inferred group");
        self.group_redundant_prune();
    }
    pub fn update_group_constraint(&mut self, player_id: usize){
        // For each gc, if player is not a participant, they now become one
        // If all players are participants, remove from tracking
        // This is only used in ExchangeChoice where players do not reveal their card but share but swap cards into and out of the pile
        // Might already be done in exchangedraw
        let mut index: usize = 0;
        while index < self.gc_vec.len() {
            let group = &mut self.gc_vec[index];
            if group.get_list()[6] == 1 {
                if group.get_list()[player_id] == 0 {
                    group.group_add(player_id);
                    // Test to see if redundant
                }
                if group.all_in() {
                    // [FULL PRUNE] because group constraint just means there could be a Duke anywhere (anyone or the pile might have it)
                    self.gc_vec.swap_remove(index);
                } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                    // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                    self.gc_vec.swap_remove(index);
                } else {
                    index += 1;
                }
            }
        }
        self.group_redundant_prune();
    }
    pub fn remove_duplicate_groups(&mut self){
        let mut seen: HashSet<GroupConstraint> = HashSet::new();
        let mut i: usize = 0;
        while i < self.gc_vec.len() {
            if seen.insert(self.gc_vec[i].clone()) {
                i += 1;
            } else {
                self.gc_vec.swap_remove(i);
                log::trace!("GROUP DUPES REMOVED");
            }
        }
    }
    pub fn player_dead_card_count(&self, player_id: usize, card: &Card) -> usize {
        let mut output: usize = 0;
        if let Some(_value) = self.pc_hm.get_key_value(&player_id){
            output += 1;
        } else if let Some(card_vec) = self.jc_hm.get(&player_id){
            for icard in card_vec.iter() {
                if icard == card {
                    output += 1;
                }
            }
        }
        return output;
    }
    pub fn dead_card_count(&self) -> &HashMap<Card, u8> {
        &self.dead_card_count
    }
    pub fn player_can_have_active_card(&self, player_id: usize, card: &Card) -> bool {
        // TODO: Consider making a version where you clone self here and let function take &mut constraint and let it modify it in function
        let mut constraint: CollectiveConstraint = self.clone();
        
        // if CollectiveConstraint::player_can_have_active_card_pub(&constraint, player_id, card) {
        //     // Doing this because there is a case where adding inferred groups leads to miscounting of card_count and \
        //     // Excluding inferred groups information is helpful
        //     constraint.add_inferred_groups();
        //     CollectiveConstraint::player_can_have_active_card_pub(&constraint, player_id, card)
        // } else {
            //     false
            // }
        // constraint.add_inferred_groups();
        CollectiveConstraint::player_can_have_active_card_pub(&constraint, player_id, card)
    }
    pub fn player_can_have_active_card_pub(input_constraint: &CollectiveConstraint, player_id: usize, card: &Card) -> bool {
        // Returns true if player can have a card in his hand that is alive
        // TODO: If this works, modify others to not clone before adding as input?
        let mut constraint: CollectiveConstraint = input_constraint.clone();
        log::trace!("Entered Recursion");
        constraint.printlog();

        // log::trace!("legality dead_card_count: {:?}", constraint.dead_card_count);
        if constraint.dead_card_count[card] == 3 {
            return false;
        }
        // === Section 1 ===
        // If a particular group has all the remaining alive cards of interest and the player of interested is not part of it, return false
        
        // Number of cards that can be alive remaining
        let remaining_alive_count: usize = 3 - constraint.dead_card_count[card] as usize;
        // log::trace!("card: {:?}", card);
        // log::trace!("Initial Remaining_alive count: {remaining_alive_count}");

        for group in constraint.gc_vec.iter() {
            // log::trace!("Considering group: {:?}", group);
            // Player may be alive and have one of the cards dead.
            // In this case card: Duke
            // Public Constraint HM: {2: Ambassador, 5: Duke, 3: Ambassador, 4: Ambassador}
            // Joint Constraint HM: {0: [Assassin, Contessa], 1: [Contessa, Contessa]}
            // Group Constraint VEC: [GroupConstraint { participation_list: [0, 0, 0, 0, 1, 1, 1], card: Captain, count: 1 }, GroupConstraint { participation_list: [0, 0, 0, 0, 0, 1, 1], card: Duke, count: 2 }]
            // remaining_alive_count = 2 but player 3 still can reveal a Duke
            // So we adjust the group count downward to prevent double counting the cards that are certain

            // This count represents the number of cards within the participation list set that are ALIVE
            if group.card() == card {
                let mut group_alive_count = group.count();
                for (iplayer_id, indicator) in group.get_list().iter().enumerate(){
                    if *indicator == 1 {
                        if let Some(value) = constraint.pc_hm.get(&iplayer_id) {
                            if value == card {
                                group_alive_count -= 1;
                            }
                        }
                        // Need this as add_inferred_groups can add groups that have dead players inside
                        if let Some(card_vec) = constraint.jc_hm.get(&iplayer_id){
                            for vcard in card_vec.iter(){
                                if *vcard == *card {
                                    group_alive_count -= 1;
                                }
                            }
                        }
                    }
                    // I do not subtract for jc_hm as group should not have indicator 1 for a dead player due to initial pruning
                }
                // If group has all of a particular card and player_id is not within the group
                // log::trace!("group_alive_count: {group_alive_count}");
                // log::trace!("remaining_alive_count: {remaining_alive_count}");
                if group.card() == card && group_alive_count == remaining_alive_count && group.get_list()[player_id] == 0 {
                    log::trace!("Basic False");
                    return false;
                }
            }
            // I reckon you don't have to check every possible union of groups because in coup when someone touches the middle pile
            // All groups are updated
        }

        // === Section 2 ===
        // Finds all possible cards for a group of players, if the player of interest is within the group and the possible cards
        // does not include the card of interest, return false

        // Checks if some combination of the group constraint tells you that a group of players have X number of some cards
        // Where the total number == the total unknown cards
        // If this is known, we know a player cant have Duke if Duke is not included the the cards the whole group can have
        // Issue with this!
        let mut i = 0;

        while i < constraint.gc_vec.len() {
            // if constraint.gc_vec[i].get_list()[player_id] == 0 {
            //     // We are looking for cases where player i is part of a group and all cards for the group is known
            //     // We continue if player i is not part of the group
            //     i += 1;
            //     continue;
            // }
            // card count is meant to store the dead and alive cards known to be within a group of players
            let mut card_count: HashMap<Card, usize> = HashMap::with_capacity(5);
            card_count.insert(Card::Ambassador, 0);
            card_count.insert(Card::Assassin, 0);
            card_count.insert(Card::Captain, 0);
            card_count.insert(Card::Duke, 0);
            card_count.insert(Card::Contessa, 0);
            let mut card_count_alive: HashMap<Card, usize> = HashMap::with_capacity(5);
            card_count_alive.insert(Card::Ambassador, 0);
            card_count_alive.insert(Card::Assassin, 0);
            card_count_alive.insert(Card::Captain, 0);
            card_count_alive.insert(Card::Duke, 0);
            card_count_alive.insert(Card::Contessa, 0);
            let mut card_count_dead: HashMap<Card, usize> = HashMap::with_capacity(5);
            card_count_dead.insert(Card::Ambassador, 0);
            card_count_dead.insert(Card::Assassin, 0);
            card_count_dead.insert(Card::Captain, 0);
            card_count_dead.insert(Card::Duke, 0);
            card_count_dead.insert(Card::Contessa, 0);
            // Initialise card_count_dead and card_count_alive
            let mut card_group_sets: HashMap<Card, [u8; 7]> = HashMap::new();
            for icard in [Card::Ambassador, Card::Assassin ,Card::Captain, Card::Duke, Card::Contessa].iter(){
                // if let Some(value) = card_count.get_mut(icard){
                //     *value = 0;
                // }
                // log::trace!("Outside Adding array");
                card_group_sets.insert(*icard, [0; 7]);
            }
            let mut total_card_count: usize = 0;
            // Initialise cap for item i, because all other items will be subsumed only if part_list are subsets
            let mut total_card_capacity: usize = 0;
            // Add current into the subsumed pool
            for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                if *indicator == 1 {
                    // Filling up total_card_capacity includes dead cards for now
                    if iplayer_id < 6 {
                        total_card_capacity += 2;
                    } else if iplayer_id == 6 {
                        total_card_capacity += 3;
                    } else {
                        debug_assert!(false, "Impossible State Reached");
                    }
                    // Filling up total_card_count for dead cards that are in group
                    // Filling up card_count to include dead cards of alive players
                    // Dont need to count dead card of dead players as DEAD PRUNE prevents group from having 1 if player is dead
                    if let Some(vcard) = constraint.pc_hm.get(&iplayer_id) {
                        total_card_count += 1;
                        if let Some(value) = card_count.get_mut(vcard) {
                            *value += 1;
                            // Update card_count_dead
                        }
                        if let Some(value) = card_count_dead.get_mut(vcard){
                            *value += 1;
                        }
                        // [Addition to test]
                        if let Some(arr) = card_group_sets.get_mut(vcard){
                            arr[iplayer_id] = 1;
                        }
                    }
                    // Include this just in case
                    // if let Some(card_vec) = self.jc_hm.get(&i) {
                    //     total_card_count += 2;
                    //     for card in card_vec.iter() {
                    //         if let Some(value) = card_count.get_mut(card) {
                    //             *value += 1;
                    //         }
                    //     }
                    // }
                }
            }
            // log::trace!("Starting Group: {:?}", constraint.gc_vec[i]);
            // log::trace!("Total Capacity: {}", total_card_capacity);
            // log::trace!("Filled Card Count dead: {:?}", card_count);
            // log::trace!("Total Card Count: {}", total_card_count);
            let mut j = 0;
            while j < constraint.gc_vec.len() {
                // log::trace!("i: {} j: {}, start card_count, {:?}", i, j, card_count);
                // if i == j {
                //     j += 1;
                //     continue;
                // }
                // subsume if possible
                // Checks if j group is subset of i group
                // log::trace!("Considering: {:?}", constraint.gc_vec[j]);
                // log::trace!("j GC: {:?}", constraint.gc_vec[j]);
                // log::trace!("i GC: {:?}", constraint.gc_vec[i]);
                if constraint.gc_vec[j].part_list_is_subset_of(&constraint.gc_vec[i]) {
                    // log::trace!("{j} is subset of {i}!");
                    // No need to store participation_list as j is a subset
                    // i part_list Subsuming j will just be i
                    let vcard: &Card = constraint.gc_vec[j].card();
                    let mut count: usize = constraint.gc_vec[j].count();

                    // === OLD FILLING CARD_COUNT
                    // If group card capacity has been reached
                    // To loop through the current and add to it for every dead card in an alive player from pc_hm
                    // count represents the total number of a card known in a group constraint + dead cards from indicators which are 0
                    // if constraint.gc_vec[i].get_list()[player_id] == 0 {
                    //     for (index, indicator) in constraint.gc_vec[j].get_list().iter().enumerate(){
                    //         if *indicator == 0 && constraint.gc_vec[i].get_list()[index] == 1{
                    //             if let Some(value) = constraint.pc_hm.get(&index){
                    //                 if *value == *vcard {
                    //                     count += 1;
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    // if let Some(count_hm) = card_count.get_mut(vcard){
                    //     if count > *count_hm {
                    //         total_card_count += count - *count_hm;
                    //         // Adding increase to total_card_count
                    //         *count_hm = count;
                    //     }
                    // }
                    // === END OLD FILLING CARD_COUNT
                    // === TODO: Modify to fill HashMap like inferred groups
                    // === START NEW
                    // log::trace!("card_group_sets: {:?}", card_group_sets);
                    if let Some(union_part_list) = card_group_sets.get_mut(&constraint.gc_vec[j].card()) {
                        let mut bool_mutual_exclusive: bool = true;
                        for icheck in 0..7 {
                            if union_part_list[icheck] == 1 && constraint.gc_vec[j].get_list()[icheck] == 1{
                                bool_mutual_exclusive = false;
                                break;
                            }
                        }
                        // log::trace!("bool ME: {}", bool_mutual_exclusive);
                        // if let Some(count_value) = card_count.get_mut(&constraint.gc_vec[j].card()) {
                        //     // TODO: Add dead/alive split
                        //     // Create another hashmap for dead/alive too
                        //     if bool_mutual_exclusive {
                        //         // Consider storing dead and alive in group constraint
                        //         // Add to both card_count dead and card_count alive based on how many there are
                        //         *count_value += constraint.gc_vec[j].count();
                        //         total_card_count += constraint.gc_vec[j].count();
                        //     } else {
                        //         if constraint.gc_vec[j].count() > *count_value {
                        //             total_card_count += constraint.gc_vec[j].count() - *count_value;
                        //             *count_value = constraint.gc_vec[j].count();
                        //         }
                        //     }
                        // }
                        if let Some(count_value) = card_count_alive.get_mut(&constraint.gc_vec[j].card()) {
                            // TODO: Add dead/alive split
                            // Create another hashmap for dead/alive too
                            if bool_mutual_exclusive {
                                // Consider storing dead and alive in group constraint
                                // Add to both card_count dead and card_count alive based on how many there are
                                *count_value += constraint.gc_vec[j].count_alive();
                                total_card_count += constraint.gc_vec[j].count_alive();
                            } else {
                                if constraint.gc_vec[j].count_alive() > *count_value {
                                    total_card_count += constraint.gc_vec[j].count_alive() - *count_value;
                                    *count_value = constraint.gc_vec[j].count_alive();
                                }
                            }
                        }
                        if let Some(count_value) = card_count_dead.get_mut(&constraint.gc_vec[j].card()) {
                            // TODO: Add dead/alive split
                            // Create another hashmap for dead/alive too
                            if bool_mutual_exclusive {
                                // Consider storing dead and alive in group constraint
                                // Add to both card_count dead and card_count alive based on how many there are
                                *count_value += constraint.gc_vec[j].count_dead();
                                total_card_count += constraint.gc_vec[j].count_dead();
                            } else {
                                if constraint.gc_vec[j].count_dead() > *count_value {
                                    total_card_count += constraint.gc_vec[j].count_dead() - *count_value;
                                    *count_value = constraint.gc_vec[j].count_dead();
                                }
                            }
                        }
                        if let Some(count_value) = card_count.get_mut(&constraint.gc_vec[j].card()){
                            *count_value = card_count_dead[&constraint.gc_vec[j].card()] + card_count_alive[&constraint.gc_vec[j].card()];
                        }
                        for iupdate in 0..7{
                            if union_part_list[iupdate] == 0 && constraint.gc_vec[j].get_list()[iupdate] == 1 {
                                union_part_list[iupdate] = 1;
                            }
                        }

                    }
                    // === END NEW
                    // log::trace!("Card Count: {:?}", card_count);
                    // log::trace!("Total Capacity: {}", total_card_capacity);
                    // log::trace!("Total Card Count: {}", total_card_count);
                    if total_card_count == total_card_capacity && constraint.gc_vec[i].get_list()[player_id] == 1 {
                        log::trace!("Group i: {:?}", constraint.gc_vec[i]);
                        log::trace!("card_count_alive: {:?}", card_count_alive);
                        log::trace!("card_count_dead: {:?}", card_count_dead);
                        log::trace!("card_count: {:?}", card_count);
                        let alive_count: usize = card_count_alive[card];
                        // If all cards for a group is known
                        // let mut alive_count: usize = card_count[card];
                        // Find all possible alive cards by subtracting dead cards for alive players
                        // We do so only for the card of interest!
                        // for (index, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                        //     if *indicator == 1 {
                        //         if let Some(value) = constraint.pc_hm.get(&index) {
                        //             if *value == *card {
                        //                 // Subtract all the dead card if an alive player had a dead card that was of interest
                        //                 if alive_count == 0 {
                        //                     debug_assert!(false, "Alive_Count Should not be 0 as we have added dead cards in!");
                        //                 } else {
                        //                     alive_count -= 1;
                        //                 }
                        //             }
                        //         }
                        //         // [NEW]
                        //         // if let Some(card_vec) = constraint.jc_hm.get(&index) {
                        //         //     for vcard in card_vec.iter(){
                        //         //         if *vcard == *card {
                        //         //             if alive_count == 0 {
                        //         //                 debug_assert!(false, "Alive_Count Should not be 0 as we have added dead cards in!");
                        //         //             } else {
                        //         //                 alive_count -= 1;
                        //         //             }
                        //         //         }
                        //         //     }
                        //         // }
                        //     }
                        // }
                        // log::trace!("ALIVE COUNT: {}", alive_count);
                        if alive_count == 0 {
                            // If full group does not contain card of interest, its impossible for a player of that group to have the card
                            // See start of fn => Group will always have player_id indicator as 1 so player_id will be part of the group
                            log::trace!("Section 2A False");
                            return false
                        } else {
                            // Since group is full, subsuming more groups will not provide more information
                            // This should continue to next i
                            j += 1;
                            log::trace!("Full Break A");
                            break;
                        }
                    } else if constraint.gc_vec[i].get_list()[player_id] == 0{
                        // Finding if some group excluding player of interest has all cards in that group known and the cards include card of interest
                        // log::trace!("Section 2B");
                        // log::trace!("Group: {:?}", constraint.gc_vec[i]);
                        // Count max number of cards that a group of alive players can possess (including dead cards)
                        let mut remaining_card_count: usize = 3;
                        for temp_player_id in 0..6 as usize {
                            // [NEW]
                            if constraint.gc_vec[i].get_list()[temp_player_id] == 0 {
                                // Removing the dead cards of alive players outside group
                                if let Some(c) = constraint.pc_hm.get(&temp_player_id){
                                    if *c == *card {
                                        remaining_card_count -= 1;
                                    }
                                }
                                // == END NEW
                                if let Some(temp_card_vec) = constraint.jc_hm.get(&temp_player_id){
                                    for c in temp_card_vec.iter(){
                                        if *c == *card {
                                            remaining_card_count -= 1;
                                        }
                                    }
                                }
                            }
                        }
                        // log::trace!("REMAINING COUNT: {}", remaining_card_count);
                        // log::trace!("GROUP COUNT: {}", card_count[card]);
                        // If all remaining cards of interest are in the group the player of interest is not in 
                        if card_count[card] == remaining_card_count {
                            log::trace!("Section 2B False");
                            return false;
                        } else {
                            // j += 1;
                            // log::trace!("Full Break B");
                            // break;
                            // [NEW] Dont break as group not full!
                        }  
                    } else if total_card_count > total_card_capacity {
                        debug_assert!(false, "Impossible State Reached");
                    }
                }
                j += 1;
            }
            // End of filling counting cards
            
            
            // Checking if group i is subset of another group that has 1 more indicator
            // Group i [1 0 0 0 0 0 1] all 5 cards known, e.g. Duke: 2
            // Group j [1 1 0 0 0 0 1] Duke 3
            // Then we know Player 1 has Duke 
            // In Addition
            // if for some player with indicator 0 outside the group, they can only have one card, add and recurse
            // log::trace!("i: {i} card_count before subset inferrence: {:?}", card_count);
            for group in constraint.gc_vec.iter() {
                // log::trace!("total_card_capacity: {total_card_capacity}");
                // log::trace!("total_card_count: {total_card_count}");
                // log::trace!("condition 1: {}", total_card_capacity == total_card_count);
                // log::trace!("Group i list: {:?}", constraint.gc_vec[i].get_list());
                // log::trace!("Group j list: {:?}", group.get_list());
                // log::trace!("condition 2: {}", constraint.gc_vec[i].is_subset_of(group));
                // log::trace!("group.card(): {:?}", group.card());
                // log::trace!("group.count(): {:?}", group.count());
                // log::trace!("card_count[group.card()]: {:?}", card_count[group.card()]);
                // log::trace!("condition 3: {}", group.count() > card_count[group.card()]);
                // log::trace!("Sum group: {}", group.get_list().iter().sum::<u8>());
                // log::trace!("Sum i: {}", constraint.gc_vec[i].get_list().iter().sum::<u8>());
                // log::trace!("condition 4: {}", (group.get_list().iter().sum::<u8>() - constraint.gc_vec[i].get_list().iter().sum::<u8>()) == 1);
                if total_card_capacity == total_card_count && constraint.gc_vec[i].part_list_is_subset_of(group) && group.count() > card_count[group.card()] && (group.get_list().iter().sum::<u8>() - constraint.gc_vec[i].get_list().iter().sum::<u8>()) == 1 {
                        // Always > 0
                        // let mut alive_count: usize = 3 - card_count[group.card()];
                        let mut alive_count: usize = group.count() - card_count[group.card()];
                        // log::trace!("alive_count: {alive_count}");
                        for iplayer in 0..7 as usize {
                            if group.get_list()[iplayer] == 1 && constraint.gc_vec[i].get_list()[iplayer] == 0 && !constraint.jc_hm.contains_key(&iplayer){
                                if let Some(vcard) = constraint.pc_hm.get(&iplayer){
                                    if *vcard == *group.card() {
                                        alive_count -= 1;
                                    }
                                }
                                if alive_count > 0 {
                                    // log::trace!("Determined that Player {} has {:?}", iplayer, *group.card());
                                    if iplayer == player_id && *group.card() == *card {
                                        // Player has card
                                        return true;
                                    } else if iplayer == player_id && constraint.pc_hm.contains_key(&player_id){
                                        // Player full hand known and it isnt card of interest
                                        log::trace!("Section 2 asd False");
                                        return false;
                                    }
                                    //ENsuring player is alive

                                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                                    new_constraint.add_raw_public_constraint(iplayer, *group.card());
                                    
                                    log::trace!("Section 2 Opposite Subset Recurse");
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        log::trace!("Section 2 Opposite Subset False");
                                        return false;
                                    } else {
                                        return true;
                                    }

                                }
                            }
                        }

                }  
                //
            }
            // Checking if another group is subset of another group i that has 1 more indicator
            // Group i [1 1 0 0 0 0 1] all 5 cards (full) known, e.g. Duke: 2 Captain 2 Assassin 3
            // Group j [1 0 0 0 0 0 1] Duke 2
            // Another [1 0 0 0 0 0 1] Captain 2
            // Then we know Player 1 must have Assassin
            //
            if total_card_capacity == total_card_count {
                //Ensuring full!

                let mut player_i_possible_cards: HashMap<Card, usize> = HashMap::new();
                // subset_union_card_list stores cards in groups that are subset of group i and exclude i_player-id
                
                let mut subset_union_card_list: HashMap<Card, [u8; 7]> = HashMap::new();
                let mut bool_reset: bool = true;
                let mut bool_mutually_exclusive: bool = false;
                let mut total_cards_remaining: usize = 0;
                let mut total_unique_cards_remaining: usize = 0;
                log::trace!("Main list : {:?}", constraint.gc_vec[i]);
                for iplayer_id in 0..6 {
                    // Copying values again
                    if bool_reset {
                        total_cards_remaining = 0;
                        total_unique_cards_remaining = 0;
                        for vcard in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa].iter() {
                            player_i_possible_cards.insert(*vcard, 0);
                            subset_union_card_list.insert(*vcard, [0; 7]);
                        }
                    }
                    // Creating a union_part_list for every card
                    if constraint.gc_vec[i].get_list()[iplayer_id] == 0 {
                        bool_reset = false;
                        continue;
                    }
                    // players_left store the group of players that can have player_i_possible_cards
                    let mut players_left: [u8; 7] = constraint.gc_vec[i].get_list().clone();
                    // log::trace!("Currently checking for player: {iplayer_id}");
                    for group in constraint.gc_vec.iter() {
                        if group.part_list_is_subset_of(&constraint.gc_vec[i]) && group.get_list()[iplayer_id] == 0 {
                            // update the subset_union_card_list and alive card counts
                            log::trace!("Checking group: {:?}", group);
                            bool_mutually_exclusive = GroupConstraint::lists_are_mut_excl(group.get_list(), &subset_union_card_list[group.card()]);
                            if let Some(value) = player_i_possible_cards.get_mut(group.card()){
                                if bool_mutually_exclusive {
                                    *value += group.count_alive();
                                } else {
                                    if group.count_alive() > *value {
                                        *value = group.count_alive();
                                    }
                                }
                                if let Some(card_part_list) = subset_union_card_list.get_mut(group.card()){
                                    for iupdate in 0..7 {
                                        if card_part_list[iupdate] == 0 && group.get_list()[iupdate] == 1 {
                                            card_part_list[iupdate] = 1;
                                        }
                                    }
                                }
                            }
                            // updating players_left to exclude players that are in current group
                            for ind_index in 0..7 as usize {
                                if players_left[ind_index] == 1 && group.get_list()[ind_index] == 1 {
                                    players_left[ind_index] = 0;
                                }
                            }
                            // log::trace!("player_i_possible_cards now: {:?}", player_i_possible_cards);
                        }
                    }
                    log::trace!("players_left: {:?}", players_left);
                    let group_player_count: u8 = players_left.iter().sum::<u8>();
                    if group_player_count > 2 {
                        bool_reset = true;
                        continue;
                    }
                    log::trace!("Currently checking for player: {iplayer_id}");
                    log::trace!("Possible Cards before subtraction: {:?}", player_i_possible_cards);
                    // update the player_i_possible_cards to be the alive cards i_player_id can possibly have
                    for (icard, ivalue) in player_i_possible_cards.iter_mut() {
                        *ivalue = card_count_alive[icard] - *ivalue;
                        total_cards_remaining += *ivalue;
                        if *ivalue > 0 {
                            total_unique_cards_remaining += 1;
                        }
                    }
                    log::trace!("subset_union_card_list: {:?}", subset_union_card_list);
                    log::trace!("Possible Cards for iplayer_id: {:?}", player_i_possible_cards);
                    if iplayer_id == player_id && player_i_possible_cards[card] == 0 {
                        log::trace!("Section Subset 0 Immediate return");
                        return false;
                    }
                    if group_player_count == 1 {
                        if total_unique_cards_remaining == 1 && !constraint.jc_hm.contains_key(&iplayer_id){
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            for (vcard, value) in player_i_possible_cards.iter(){
                                if *value > 0 && constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id){
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            log::trace!("Section Subset 0 Known A True");
                                            return true;
                                        } else {
                                            // The last card that could be alive is not the card of interest!
                                            log::trace!("Section Subset 0 Known B False");
                                            return false;
                                        }
                                    } 
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        log::trace!("Section Subset 0 Recurse A False");
                                        return false;
                                    } else {
                                        return true;
                                    }
                                } else if *value > 1 && !constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id){
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            log::trace!("Section Subset 0 Known C True");
                                            return true;
                                        } else {
                                            log::trace!("Section Subset 0 Known D False");
                                            return false;
                                        }
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        log::trace!("Section Subset 0 Recurse B False");
                                        return false;
                                    } else {
                                        return true;
                                    }
                                } else if *value == 1 && !constraint.jc_hm.contains_key(&iplayer_id){
                                    // implicitly pc_hm does not contain a key here! because of the first if condition
                                    // iplayer_id has 2 lives
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            log::trace!("Section Subset 0 Known E True");
                                            return true;
                                        }
                                        // No return false because they have another card we dont know about!
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        log::trace!("Section Subset 0 Recurse C False");
                                        return false;
                                    } else {
                                        return true;
                                    }
                                }
                            }
                        } else if total_unique_cards_remaining == 2 && total_cards_remaining == 2 {
                            if !constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id){
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                let mut counter: usize = 0;
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value > 0 {
                                        if iplayer_id == player_id {
                                            if *vcard == *card {
                                                log::trace!("Section Subset 0 Known F True");
                                                return true;
                                            } else if counter == 1 {
                                                // return false only if both cards are not card of interest!
                                                log::trace!("Section Subset 0 Known G False");
                                                return false;
                                            }
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        counter += 1;
                                    }
                                }
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    log::trace!("Section Subset 0 Recurse D False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } else if total_unique_cards_remaining == 2 && total_cards_remaining == 3{
                            // if player has 2 lives add card with value = 2
                            if !constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id) {
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value > 1 {
                                        if iplayer_id == player_id {
                                            if *vcard == *card {
                                                log::trace!("Section Subset 0 Known H True");
                                                return true;
                                            } 
                                            // No return false here because only 1 card will be added!
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            log::trace!("Section Subset 0 Recurse E False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                }
                            }
                        } else if total_unique_cards_remaining == 2 && total_cards_remaining == 4 {
                            // if player has 2 lives add card with value = 3
                            // In the case of a 3 1 card distribution of each kind of card, if a player has 2 slots they definitely have the card with count 3
                            if !constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id) {
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value > 2 {
                                        if iplayer_id == player_id {
                                            if *vcard == *card {
                                                log::trace!("Section Subset 0 Known I True");
                                                return true;
                                            } 
                                            // No return false here because only 1 card will be added!
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            log::trace!("Section Subset 0 Recurse E False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // May consider adding case where there are 2 players left but yknow maybe its not needed. just send it to recurse somewhere!
                    // 1 unique card 3 total cards 2 players -> both players have the 1 of card
                    // 1 unique card 3 total cards 2 players -> if players have 3 lives collectively, all their empty cards are the card
                    // 1 unique card 2 total cards 2 players -> if both players have 1 life only both players have the 1 of card
                    // 1 unique card 1 total cards 2 players -> unknown skip
                    // 2 unique cards 4 total cards 2 2 Split 2 players -> both players have 4 lives collectively -> unknown skip
                    // 2 unique cards 4 total cards 3 1 Split 2 players -> both players have 4 lives collectively -> both players have 1 of the card with 3 total cards
                    // 2 unique cards 3 total cards 2 1 Split 2 players -> both players have 3 lives collectively -> player with 2 lives has 1 of the card with 2 total cards
                    // 3 unique cards 4 total cards 2 1 1 Split -> both players have 4 lives collectively -> unknown skip
                    // 3 unique cards 3 total cards 1 1 1 Split -> both players have 3 lives collectively -> unknown skip
                    // if required, add_raw_constraints and recurse
    
                    bool_reset = true;
                }
            }


            // Next algo begins here!
            // log::trace!("Initial card_count: {:?}", card_count);
            // Check if new information can be obtained
            // I can, add push_raw new information and recurse

            // Modifying card_count for a different purpose!
            // for every 0 in group i add dead cards of dead players in. This was not counted at the start of while loop for i
            // log::trace!("card_count before: {:?}", card_count);
            for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                // [TESTING] if need to comment out or not ANS: Needs to be commented out
                // Trying to make it work with this
                if *indicator == 0 {
                    // if let Some(vcard) = constraint.pc_hm.get(&iplayer_id){
                    //     if let Some(value) = card_count.get_mut(vcard){
                    //         *value += 1;
                    //     }
                    // }
                    if let Some(temp_card_vec) = constraint.jc_hm.get(&iplayer_id){
                        for vcard in temp_card_vec.iter(){
                            if let Some(value) = card_count.get_mut(vcard){
                                *value += 1;
                            }
                        }
                    }
                }
            }

            // We Count number of cards left dead and alive for alive players group outside group i
            // [NEW] We count number of cards left alive for alive players outside group i
            // log::trace!("card_count after: {:?}", card_count);
            let mut no_cards_remaining: usize = 0;
            let mut no_unique_cards_remaining: usize = 0;
            for value in card_count.values_mut(){
                *value = 3 - *value;
                no_cards_remaining += *value;
                if *value != 0 {
                    no_unique_cards_remaining += 1;
                }
            }
            // [NEW]
            for (iplayer, indicator) in constraint.gc_vec[i].get_list().iter().enumerate() {
                if *indicator == 0 {
                    if let Some(vcard) = constraint.pc_hm.get(&iplayer){
                        if let Some(count_value) = card_count.get_mut(vcard){
                            *count_value -= 1;
                            no_cards_remaining -= 1;
                        }
                    }
                }
            }
            // [NEW END]

            
            // log::trace!("no_remaining_cards: {}", no_cards_remaining);
            // log::trace!("card_count subtracted: {:?}", card_count);
            // card_count is now all possible cards alive and outside group i
            // [WHAT IS HAPPENING] We try to infer what a player's hand must have, add a new constraint and recurse!
            if constraint.gc_vec[i].get_list()[player_id] == 0 && card_count[card] == 0{
                log::trace!("False when player outside group and cant have card!");
                return false;
            }
            else if no_cards_remaining == 2 {
                // Check if we can infer a player's entire hand
                // There should only be one player alive outside the group that fulfils this
                let mut recurse_bool: bool = false;
                let mut new_constraint: CollectiveConstraint = constraint.clone();
                for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                    if *indicator == 0 && iplayer_id == 6 {
                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm > 0 {
                                new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card_hm, 0, *value_hm));
                                recurse_bool = true;
                            }
                        }     
                    }
                    if *indicator == 0 && !constraint.jc_hm.contains_key(&iplayer_id) && !constraint.pc_hm.contains_key(&iplayer_id) {
                        // if player is fully alive and outside the group, we know their entire hand

                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm == 1{
                                if iplayer_id == player_id && *card_hm == *card {
                                    return true;
                                // } else if iplayer_id == player_id {
                                //     // Player full hand known and it isnt card of interest
                                //     return false;
                                }
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                recurse_bool = true;
                            } else if *value_hm == 2 {
                                if iplayer_id == player_id && *card_hm == *card {
                                    return true;
                                // } else if iplayer_id == player_id && constraint.pc_hm.contains_key(&player_id){
                                } else if iplayer_id == player_id {
                                    // Player full hand known and it isnt card of interest
                                    return false;
                                }
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                recurse_bool = true;
                                break;
                            }
                        }
                    } else if *indicator == 0 && !constraint.jc_hm.contains_key(&iplayer_id) && constraint.pc_hm.contains_key(&iplayer_id) {
                        // if player has 1 life and both cards remaining are the same
                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm == 2 {
                                if iplayer_id == player_id && *card_hm == *card {
                                    return true;
                                } else if iplayer_id == player_id {
                                    // Player full hand known and it isnt card of interest
                                    return false;
                                }
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                recurse_bool = true;
                                break;
                            }
                        }
                        
                    }
                }
                if recurse_bool {
                    // new_constraint.printlog();
                    log::trace!("Section 2 Recurse A Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        log::trace!("Section 2 Recurse A False");
                        return false;
                    } else {
                        return true;
                    }
                }
            } else if no_cards_remaining == 1 {
                let mut recurse_bool: bool = false;
                let mut new_constraint: CollectiveConstraint = constraint.clone();
                for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                    if *indicator == 0 && iplayer_id == 6 {
                        // Consider player 6 just add as group constraint
                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm > 0 {
                                new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card_hm, 0, *value_hm));
                                recurse_bool = true;
                                break;
                            }
                        }
                    }
                    if *indicator == 0 && !constraint.jc_hm.contains_key(&iplayer_id) && constraint.pc_hm.contains_key(&iplayer_id) {
                        // if player has 1 life left and is outside the group, we know their entire hand
                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm == 1{
                                if iplayer_id == player_id && *card_hm == *card {
                                    return true;
                                } else if iplayer_id == player_id {
                                    // Player full hand known and it isnt card of interest
                                    return false;
                                }
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                recurse_bool = true;
                                break;
                            } else if *value_hm == 2 {
                                debug_assert!(false, "Impossible Case");
                                if iplayer_id == player_id && *card_hm == *card {
                                    return true;
                                } else if iplayer_id == player_id {
                                    // Player full hand known and it isnt card of interest
                                    return false;
                                }
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                recurse_bool = true;
                            }
                        }
                    }
                }
                if recurse_bool {
                    // new_constraint.printlog();
                    log::trace!("Section 2 Recurse B Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        log::trace!("Section 2 Recurse B False");
                        return false;
                    } else {
                        return true;
                    }
                }
            } else if no_cards_remaining == 3 {
                // If we know the ambassador's hand add constraints and recurse
                // if constraint.gc_vec[i].get_list()[6] == 0 && !constraint.pc_hm.contains_key(&6) && !constraint.pc_hm.contains_key(&6){
                let mut recurse_bool: bool = false;
                let mut new_constraint: CollectiveConstraint = constraint.clone();
                for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                    if *indicator == 0 && iplayer_id == 6 {
                        for (card_hm, value_hm) in card_count.iter(){
                            if *value_hm > 0 {
                                new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card_hm, 0, *value_hm));
                                recurse_bool = true;
                            }
                        }
                    } else if *indicator == 0 && !new_constraint.pc_hm.contains_key(&iplayer_id) && !new_constraint.jc_hm.contains_key(&iplayer_id){
                        if no_unique_cards_remaining == 2 {
                            // 2 unique cards and a player with 2 lives remaining
                            for (card_hm, value_hm) in card_count.iter(){
                                if *value_hm > 1 {
                                    if iplayer_id == player_id && *card_hm == *card {
                                        return true;
                                    } else if iplayer_id == player_id && constraint.pc_hm.contains_key(&player_id){
                                        // Player full hand known and it isnt card of interest
                                        return false;
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                    recurse_bool = true;
                                }
                            }
                        } else if no_unique_cards_remaining == 1 {
                            // 1 unique card and a player with 2 lives remaining
                            for (card_hm, value_hm) in card_count.iter(){
                                if *value_hm > 1 {
                                    if iplayer_id == player_id && *card_hm == *card {
                                        return true;
                                    } else if iplayer_id == player_id && constraint.pc_hm.contains_key(&player_id){
                                        // Player full hand known and it isnt card of interest
                                        return false;
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                    new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                    recurse_bool = true;
                                }
                            }
                        }
                    } else if *indicator == 0 && new_constraint.pc_hm.contains_key(&iplayer_id) && !new_constraint.jc_hm.contains_key(&iplayer_id){
                        // 1 unique card and a player with 1 life remaining
                        if no_unique_cards_remaining == 1 {
                            for (card_hm, value_hm) in card_count.iter(){
                                if *value_hm > 1 {
                                    if iplayer_id == player_id && *card_hm == *card {
                                        return true;
                                    } else if iplayer_id == player_id && constraint.pc_hm.contains_key(&player_id){
                                        // Player full hand known and it isnt card of interest
                                        return false;
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                    recurse_bool = true;
                                }
                            }
                        }
                    }
                }

                if recurse_bool {
                    // new_constraint.printlog();
                    log::trace!("Section 2 Recurse C Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        log::trace!("Section 2 Recurse C False");
                        return false;
                    } else {
                        return true;
                    }
                }
            } else if no_cards_remaining == 4 {
                let mut recurse_bool: bool = false;
                let mut new_constraint: CollectiveConstraint = constraint.clone();
                for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                    if *indicator == 0 && iplayer_id == 6 {
                        // Pile case
                        if no_unique_cards_remaining == 2 {
                            // Either a 2 2 split or a 1 3 split of each type of card
                            // Since pile has 3 cards
                            // 2 2 split add 1 of each since it will definitely have 1 of each card
                            // 1 3 split add 2 of the 3 card type since it will definitely have at least 2 of that
                            for (card_hm, value_hm) in card_count.iter(){
                                if *value_hm == 3 {
                                    new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card_hm, 0, 2));
                                    recurse_bool = true;
                                } else if *value_hm == 2 {
                                    new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card_hm, 0, 1));
                                    recurse_bool = true;
                                }
                            }
                        }
                    } else if *indicator == 0 && !new_constraint.pc_hm.contains_key(&iplayer_id) && !new_constraint.jc_hm.contains_key(&iplayer_id){
                        // player has 2 lives
                        if no_unique_cards_remaining == 2 {
                            // If its a 1 3 Split, we know player must have at least 1 of the card with 3 left
                            for (card_hm, value_hm) in card_count.iter(){
                                if *value_hm == 3 {
                                    if iplayer_id == player_id && *card_hm == *card {
                                        return true;
                                    } else if iplayer_id == player_id && constraint.pc_hm.contains_key(&player_id){
                                        // Player full hand known and it isnt card of interest
                                        return false;
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *card_hm);
                                    recurse_bool = true;
                                }
                            }
                        }
                    }
                    // (We skip player with 1 life as they could have literally any card)
                }
                if recurse_bool {
                    // new_constraint.printlog();
                    log::trace!("Section 2 Recurse D Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        log::trace!("Section 2 Recurse D False");
                        return false;
                    } else {
                        return true;
                    }
                }
            }
            // [Whats Happening] Checking if we can infer more information about players outside 
            // Say we know group [1 0 0 0 0 0 1] has 2 Ambassadors 1 Capt 1 Duke 1 Cont
            // Say we know group [1 1 0 0 0 0 1] has 3 Ambassadors
            // Then we know [0 1 0 0 0 0 0] has at least 1 ambassador
            // If that player already has a dead card we can return false
            // else we can add that in and recurse?

            // Modify card_count now to be only alive cards outside group i
            // Instead of both alive and dead cards
            // Actually it seems like this is a repeat of the above
            // Wonder what is wrong?
            // [NEW] commented out
            // for (iplayer, indicator) in constraint.gc_vec[i].get_list().iter().enumerate() {
            //     if *indicator == 0 {
            //         if let Some(vcard) = constraint.pc_hm.get(&iplayer){
            //             if let Some(count_value) = card_count.get_mut(vcard){
            //                 *count_value -= 1;
            //                 no_cards_remaining -= *count_value;
            //             }
            //         }
            //     }
            // }

            i += 1;
        }

        // === Section 3 ===
        // [MUST HAVE CARD CHECK]
        // Check if player must have certain cards
        // if player_id must have some cards and its not card of interest then false
        // if some other players must have card of interest, and there are no more remaining, return false
        // recursive? if player must have some card add to pc_hm? (not raw add but add with prune because its not a query but a deduction!)
        // Store impossible active cards for the player i includes all jc_hm cards and player_i pc_hm
        let mut i: usize = 0;
        while i < 7 {
            // skip if player i is dead because we know all their cards
            if constraint.jc_hm.contains_key(&i) {
                i += 1;
                continue;
            }
            let mut excl_list: [u8; 7] = [0; 7];
            let mut excl_card_count: HashMap<Card, usize> = HashMap::with_capacity(5);
            excl_card_count.insert(Card::Ambassador, 0);
            excl_card_count.insert(Card::Assassin, 0);
            excl_card_count.insert(Card::Captain, 0);
            excl_card_count.insert(Card::Duke, 0);
            excl_card_count.insert(Card::Contessa, 0);
            let mut j: usize = 0;
            // Collect hashmap of what cards must exist outside of player hand
            while j < constraint.gc_vec.len() {
                if j == i {
                    j += 1;
                    continue;
                }
                if constraint.gc_vec[j].get_list()[i] == 1 {
                    j += 1;
                    continue;
                } else {
                    // update counts and list by making a union of all groups that do not include player i in part_list
                    // modifies excl_list to include list of group j
                    // Assume group constraints include dead card of alive players (pc_hm) if their indicator is 1
                    // Assume group constraints exclude dead players [DEAD PLAYER PRUNE]
                    if GroupConstraint::lists_are_mut_excl(&excl_list, &constraint.gc_vec[j].get_list()) {
                        if let Some(value) = excl_card_count.get_mut(&constraint.gc_vec[j].card()) {
                            *value += constraint.gc_vec[j].count();
                        }
                    } else {
                        // Either Subset or Some intersection exists
                        if let Some(value) = excl_card_count.get_mut(&constraint.gc_vec[j].card()) {
                            if constraint.gc_vec[j].count() > *value {
                                *value = constraint.gc_vec[j].count();
                            }
                        }
                    }
                    GroupConstraint::list_union(&mut excl_list, &constraint.gc_vec[j].get_list());
                }
                j += 1;
            }
            // Adding dead cards if not already included
            let mut k: usize = 0;
            while k < excl_list.len() {
                if excl_list[k] == 0 {
                    // Include dead players card if they are not in the excl_list group and the player is fully dead
                    // This will include player i as player i excl_list value will be 0 but player i should be alive so...
                    if let Some(temp_card_vec) = constraint.jc_hm.get(&k) {
                        for vcard in temp_card_vec.iter() {
                            // update Hashmap
                            if let Some(value) = excl_card_count.get_mut(&vcard) {
                                *value += 1;
                            }
                        }
                    }
                    // Include dead card of alive player if they are not in excl_list group
                    if let Some(vcard) = constraint.pc_hm.get(&k) {
                        if let Some(value) = excl_card_count.get_mut(&vcard) {
                            *value += 1;
                        }
                    }
                }
                k += 1;
            }
            // Check if excl count implies player i must have a certain card
            let mut possible_cards: Vec<Card> = Vec::with_capacity(5);
            let mut possible_count: usize = 0;
            for vcard in [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa].iter() {
                if excl_card_count[vcard] != 3 {
                    possible_cards.push(*vcard);
                    possible_count += 1;
                }
                // Exiting early because we only care if possible_count == 1
                if possible_count == 2 {
                    break;
                }
            }
            if possible_count == 0 {
                log::trace!("HAVE CARD CASE FOUND DUMMY");
            }
            if player_id == i {
                // If the player of interest has card of interest return true
                if possible_count == 1 && possible_cards[0] == *card {
                    log::trace!("Section 3A True");
                    return true;
                } else if possible_count == 1 && possible_cards[0] != *card {
                    log::trace!("Section 3A False");
                    return false;
                }
                // continue checking if unsure
            } else {
                // Not player of interest
                if possible_count == 1 {
                    // create new constraint and recurse
                    // if recurse value is false return false, else continue
                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                    // Im using the add_public instead of add_public_raw because I want pruning as I treat this 
                    new_constraint.add_public_constraint(i, possible_cards[0]);
                    // let mut temp_arr: [u8; 7] = [0; 7];
                    // temp_arr[i] = 1;
                    // new_constraint.add_raw_group(GroupConstraint::new_list(temp_arr, possible_cards[0], 1));
                    log::trace!("Section 3B Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card) {
                        log::trace!("Section 3B False");
                        return false;
                    } else {
                        return true;
                    }
                }
            }
            // Check if player i must have some card, if they do call a recursive function
            // if false return false
            // Special case for if player i == player_id
            // Then if player i must have some card, and it is not card of interest return false else return true
            i += 1;
        }   
        true
    }
    pub fn printlog(&self) {
        log::trace!("{}", format!("Public Constraint HM: {:?}", self.pc_hm));
        log::trace!("{}", format!("Joint Constraint HM: {:?}", self.jc_hm));
        log::trace!("{}", format!("Group Constraint VEC: {:?}", self.gc_vec));
    }
    pub fn player_can_have_active_cards(&self, player_id: usize, cards: &[Card; 2]) -> bool {
        if self.player_can_have_active_card(player_id, &cards[0]){
            let mut new_constraint: CollectiveConstraint = self.clone();
            if player_id == 6 {
                // Add case for when both cards are the same!
                // If both are the same it will return legal if 1 card works because I dont add [0, 0, 0, 0, 0, 0, 1] count: 2
                if cards[0] != cards[1] {
                    new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 1));
                } else {
                    // When they are same, we put both constraints and see if can have the card? I dont think this should work tho?
                    // CURRENTLY NO PROPER FUNCTIONALITY IMPLEMENTED FOR TESTING IF PILE HAS 2 OF THE SAME CARD
                    // ITS NOT NEEDED FOR ALGO SO THIS DISCLAIMER IS 
                    // THIS IS A TEMP FIX THAT MAY WORK BUT LEADS TO USIZE OVERFLOW
                    // new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 2));
                    return true;
                }
            } else {
                new_constraint.add_raw_public_constraint(player_id, cards[0]);
            }
            CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, &cards[1])
        } else {
            false
        }
    }
}

pub struct NaiveProb {
    // a vec of constraints to push and pop
    // dead cards to push or pop
    // Will not locally store game history, jsut the constraint history
    constraint_history: Vec<Option<CollectiveConstraint>>,
    all_states: Vec<String>,
    // distfromlast tells u how far away to fetch the last constraint_history
    dist_from_last: Vec<usize>,
    calculated_states: Vec<String>,
    index_start_arr: [usize; 7],
    index_end_arr: [usize; 7],
    card_list: [Card; 5],
    set_store: Vec<HashMap<String, HashSet<String>>>,
    belief_hm: HashMap<String, Vec<f64>>,
    unique_2p_hands: Vec<String>,
    unique_3p_hands: Vec<String>,
}
impl NaiveProb {
    pub fn new() -> Self {
        let unique_2p_hands: Vec<String> = gen_bag_combinations(TOKENS, &2);
        let unique_3p_hands: Vec<String> = gen_bag_combinations(TOKENS, &3);
        let mut all_states: Vec<String> = gen_table_combinations(TOKENS, &BAG_SIZES);
        let mut rng = rand::thread_rng();
        all_states.shuffle(&mut rng); // Shuffle in place
        NaiveProb{
            constraint_history: Vec::with_capacity(1500),
            all_states,
            dist_from_last:Vec::with_capacity(1500),
            calculated_states: Vec::with_capacity(MAX_PERM_STATES),
            index_start_arr: [0, 2, 4, 6, 8, 10, 12],
            index_end_arr: [2, 4, 6, 8, 10, 12, 15],
            card_list: [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa],
            set_store: vec![HashMap::new(); 6],
            belief_hm: HashMap::new(),
            unique_2p_hands,
            unique_3p_hands,
        }
    }
    pub fn reset(&mut self) {
        self.constraint_history = Vec::with_capacity(1500);
        self.dist_from_last = Vec::with_capacity(1500);
        self.calculated_states = Vec::with_capacity(MAX_PERM_STATES)
    }
    pub fn set_generation(&mut self) {
        let mut pc_hm: HashMap<usize, char> = HashMap::new();
        let mut set_store: Vec<HashMap<String, HashSet<String>>> = vec![HashMap::new(); 6];
        for i in 0..6 {
            pc_hm.insert(i, 'A');
            pc_hm.insert(i, 'B');
            pc_hm.insert(i, 'C');
            pc_hm.insert(i, 'D');
            pc_hm.insert(i, 'E');
        }
        let char_list: [char; 5] = ['A', 'B', 'C', 'D', 'E'];
        let mut index_start: usize;
        let mut index_end: usize;
        // Initialising pc sets
        for i in 0..6 {
            index_start = self.index_start_arr[i];
            index_end = self.index_end_arr[i];
            for card_char in char_list.iter(){
                let filtered: Vec<String> = self.all_states.par_iter()
                .filter(|state| state[index_start..index_end].contains(*card_char))
                .cloned().collect();
            let hash_set: HashSet<String> = filtered.into_iter().collect();
            set_store[i].insert(card_char.to_string(), hash_set);
        }
        let str_list: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];

        for i in 0..6 {
            index_start = self.index_start_arr[i];
            index_end = self.index_end_arr[i];
            for s in str_list.iter(){
                let card_char_vec: Vec<char> = s.chars().collect();
                let filtered: Vec<String> = self.all_states.par_iter()
                .filter(|state| {
                    let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
                    state.len() >= index_end && state_chars == *card_char_vec
                })
                .cloned().collect();
                let hash_set: HashSet<String> = filtered.into_iter().collect();
                set_store[i].insert(s.to_string(), hash_set);
            }
        }
    }
    // Initialising jc sets

    }
    pub fn prev_index(&self) -> usize {
        self.dist_from_last[self.dist_from_last.len() - 1]
    }
    // pub fn sort_and_serialize_hashmap<K, V>(&self, hashmap: &HashMap<K, V>) -> String
    // where
    //     K: Serialize + Ord + std::hash::Hash,
    //     V: Serialize,
    // {   // Serializes Hashmap
    //     let mut sorted_map: HashMap<&K, &V> = hashmap.iter().collect();
    //     let mut sorted_pairs: Vec<(&K, &V)> = sorted_map.drain().collect();
    //     sorted_pairs.sort_by_key(|&(k, _)| k);
    //     serde_json::to_string(&sorted_pairs).unwrap()
    // }
    // pub fn make_key_belief(&self) -> String {
    //     // Makes key to store the belief probabilities based on constraints
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let public_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.pc_hm);
    //     let joint_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.jc_hm);
    //     let group_constraint_key: String = self.sort_and_serialize_hashmap(&latest_constraint.gc_hm);
    //     let big_key: String = format!("{}_{}_{}", public_constraint_key, joint_constraint_key, group_constraint_key);
    //     big_key
    // }
    // pub fn key_in_bson_hashmap(&mut self, key: String) -> bool {
    //     if self.belief_hm.contains_key(&key){
    //         true
    //     } else {
    //         false
    //     }
    // }
    // pub fn add_to_hashmap(&mut self, key: String, belief_vec: Vec<f64>){
    //     self.belief_hm.entry(key).or_insert(belief_vec);
    // }
    // pub fn load_bson_hashmap(&mut self){
    //     self.belief_hm = load_initial_hashmap("naive_belief_prob_hashmap.bson");
    //     log::trace!("{}", format!("Loaded up with len: {:?}", self.belief_hm.len()));
    // }
    // pub fn save_bson_hashmap(&mut self){
    //     match save_bson_hashmap(&self.belief_hm, "naive_belief_prob_hashmap.bson"){
    //         Ok(_) => log::info!("Saved HashMap of len: {}", self.belief_hm.len()),
    //         Err(_) => log::info!("Failed to save HashMap"),
    //     }
    // }
    pub fn latest_constraint_is_empty(&self) -> bool {
        if self.constraint_history.len() == 0 {
            true
        } else {
            let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.is_empty()
        }
    }
    pub fn print_belief_hm_len(&self){
        println!("HashMap now of len: {}", self.belief_hm.len());
    }
    pub fn printlog(&self) {
        log::trace!("{}", format!("Constraint History Len{}", self.constraint_history.len()));
        if self.constraint_history.len() == 0 {

        } else {
            let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            latest_constraint.printlog();
        }
        // log::info!("{}", format!("Set Size: {}", self.calculated_states.len()));
    }
    pub fn calc_state_len(&self) -> usize {
        self.calculated_states.len()
    }
    pub fn log_calc_state_len(&self){
        log::trace!("{}", format!("Calculated_State Length: {}", self.calculated_states.len()));
    }
    pub fn log_calc_state(&self){
        log::trace!("{}", format!("Calculated_State: {:?}", self.calculated_states));
    }
    // Push()
    pub fn push_ao(&mut self, ao: &ActionObservation){
        // log::trace!("{}", format!("Before Pushing AO {:?}", ao));
        // self.printlog();
        if ao.name() == AOName::Discard {
            match ao.no_cards() {
                1 => {
                    if let Some(temp_card) = ao.cards().first(){
                        self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
                        if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                            last_constraint.group_initial_prune(ao.player_id(), temp_card, 1, true);
                            last_constraint.add_public_constraint(ao.player_id(), *temp_card);
                        } else {
                            debug_assert!(false, "constraint not stored at prev_index!");
                        }
                    } else {
                        debug_assert!(false, "Card does not exist!!");
                    }
                },
                2 => {
                    let temp_cards = ao.cards();
                    self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
                    if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                        if temp_cards[0] == temp_cards[1]{
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[0], 2, true);
                        } else {
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[0], 1, true);
                            last_constraint.group_initial_prune(ao.player_id(), &temp_cards[1], 1, true);
                        }
                        last_constraint.add_joint_constraint(ao.player_id(), &temp_cards.to_vec());
                    } else {
                        debug_assert!(false, "Card does not exist!!");
                    }
                },
                _ => {
                    debug_assert!(false,"Unexpected no_cards case");
                }
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::RevealRedraw{
            self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                // We update when cards a mixed with pile by player_id but no card is revealed
                // last_constraint.update_group_constraint_hm(ao.player_id());
                // last_constraint.add_group_constraint_hm(ao.player_id(), ao.card());
                last_constraint.group_initial_prune(ao.player_id(), &ao.card(), 1, false);
                // last_constraint.update_group_constraint(ao.player_id());
                last_constraint.add_group_constraint(ao.player_id(), &ao.card(), 1);

                // last_constraint.update_group_constraint(ao.player_id());
                // We add because a card is revealed before mixing | We add after updating for minor efficiency
                // TODEBUG: Should only add if not already inside!
                // last_constraint.add_group_constraint(ao.player_id(), ao.card());
            } else {
                // Handle the case where the last element is None or the vector is empty
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeDraw {
            self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
                // Player gets info that the pile has 2 cards, which prunes other groups
                if ao.cards()[0] == ao.cards()[1]{
                    last_constraint.group_initial_prune(6, &ao.cards()[0], 2, false);
                } else {
                    last_constraint.group_initial_prune(6, &ao.cards()[0], 1, false);
                    last_constraint.group_initial_prune(6, &ao.cards()[1], 1, false);
                }

                if ao.cards()[0] == ao.cards()[1] {
                    last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 2);
                } else {
                    last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[0], 1);
                    last_constraint.add_group_constraint_exchange(ao.player_id(), &ao.cards()[1], 1);
                }
            } else {
                debug_assert!(false, "constraint not stored at prev_index!");
            }
            self.dist_from_last.push(1);
        } else if ao.name() == AOName::ExchangeChoice {
            if self.dist_from_last.len() != 0 {
                self.constraint_history.push(self.constraint_history[self.constraint_history.len() - self.prev_index()].clone());
            } else {
                let empty_collective_constraint: CollectiveConstraint = CollectiveConstraint::new();
                self.constraint_history.push(Some(empty_collective_constraint));
            }
            // Obsolete because it is handled in ExchangeDraw
            // update_group_constraint will only be used for ExchangeDraw/ExchangeChoice when looking at past history and private information is not searched
            // if let Some(last_constraint) = self.constraint_history.last_mut().and_then(|opt| opt.as_mut()) {
            //     // We update when cards a mixed with pile by player_id but no card is revealed
            //     last_constraint.update_group_constraint(ao.player_id());
            //     // last_constraint.update_group_constraint_hm(ao.player_id());
            // } else {
            //     // Handle the case where the last element is None or the vector is empty
            //     debug_assert!(false, "constraint not stored at prev_index!");
            // }
            self.dist_from_last.push(1);
        } else {
            // Add new case
            if self.dist_from_last.len() != 0{
                self.constraint_history.push(None);
                self.dist_from_last.push(self.dist_from_last[self.dist_from_last.len() - 1] + 1);
            } else {
                let empty_collective_constraint: CollectiveConstraint = CollectiveConstraint::new();
                self.constraint_history.push(Some(empty_collective_constraint));
                self.dist_from_last.push(1);
            }
        } 
    }
    pub fn pop(&mut self) {
        self.dist_from_last.pop();
        self.constraint_history.pop();
    }
    //TODO: Add union constraints
    //TODO: Calculate Beliefs
    // pub fn filter_state(&mut self) {
    //     // Test for 
    //     // RELEASE MODE TIME           : 0.10~0.14 s for just the public constraints both single and joint
    //     // RELEASE MODE TIME concurrent: 0.13~0.23 s for just the public constraints both single and joint
    //     // RELEASE MODE TIME           : 0.15~0.18 s with single, joint and group
    //     // RELEASE MODE TIME concurrent: 0.10~0.11 s for just the public constraints both single and joint
    //     // if let Some(latest_constraint) = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone();
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     self.calculated_states = self.all_states.clone();
    //     let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             self.calculated_states.retain(
    //                 |state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 }
    //             );
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for JC Filter: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for PC Filter: {:?}", elapsed_time);
    //     //TODO: move these items to self
    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];
    //     let card_list: [Card; 5] = [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa];
    //     for card in card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((index_start_arr[player_id], index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             self.calculated_states.retain(|state| {
    //                 participating_indices.iter().any(|&(start, end)|{
    //                     state.len() >= end && state[start..end].contains(card_char)
    //                 })
    //             });
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for GC Filter: {:?}", elapsed_time);
    //     // for group_constraint in latest_constraint.gc_vec.iter(){
    //     //     // group_constraint.participation_list: [u8; 7]
    //     //     let participation_list: &[u8] = group_constraint.get_list();
    //     //     let card: &Card = group_constraint.card();
    //     //     let card_char: char = self.card_to_char(&card);

    //     //     let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //     //         .filter_map(|(player_id, &participation)| {
    //     //             if participation == 1{
    //     //                 Some((index_start_arr[player_id], index_end_arr[player_id]))
    //     //             } else {
    //     //                 None
    //     //             }
    //     //         }).collect();
    //     //     self.calculated_states.retain(|state| {
    //     //         participating_indices.iter().any(|&(start, end)|{
    //     //             state.len() >= end && state[start..end].contains(card_char)
    //     //         })
    //     //     });
    //     // }
    // }
    // pub fn filter_state_concurrent(&mut self) {
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     self.calculated_states = self.all_states.clone();

    //     // USE NORMAL VERSION FOR THIS, NON CONCURRENT ALWAYS WINS IN TIME WHEN THERE IS NO JOINT
    //     let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 })
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent JC Filter: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| state[index_start..index_end].contains(card_char))
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];

    //     let index_start_arr: [usize; 7] = [0, 2, 4, 6, 8, 10, 12];
    //     let index_end_arr: [usize; 7] = [2, 4, 6, 8, 10, 12, 15];
    //     let card_list: [Card; 5] = [Card::Ambassador, Card::Assassin, Card::Captain, Card::Duke, Card::Contessa];
    //     start_time = Instant::now();
    //     for card in card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((index_start_arr[player_id], index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     participating_indices.iter().any(|&(start, end)| {
    //                         state.len() >= end && state[start..end].contains(card_char)
    //                     })
    //                 })
    //                 .cloned().collect();

    //             // Update calculated_states with filtered results
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Concurrent GC Filter: {:?}", elapsed_time);
    //     // Assuming participation_list and card_char are prepared as before
    //     // for group_constraint in latest_constraint.gc_vec.iter() {
    //     //     // Assuming get_list() and card() methods, similar preparation as in sequential version
    //     //     let participation_list = group_constraint.get_list(); // Example method call, adjust as needed
    //     //     let card_char = self.card_to_char(&group_constraint.card());

    //     //     let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //     //         .filter_map(|(player_id, &participation)| {
    //     //             if participation == 1 {
    //     //                 Some((index_start_arr[player_id], index_end_arr[player_id]))
    //     //             } else {
    //     //                 None
    //     //             }
    //     //         })
    //     //         .collect();

    //     //     // Concurrently filter states based on group constraints
    //     //     let filtered: Vec<String> = self.calculated_states.par_iter()
    //     //         .filter(|state| {
    //     //             participating_indices.iter().any(|&(start, end)| {
    //     //                 state.len() >= end && state[start..end].contains(card_char)
    //     //             })
    //     //         })
    //     //         .cloned().collect();

    //     //     // Update calculated_states with filtered results
    //     //     self.calculated_states = filtered;
    //     // }
    // }
    // pub fn filter_state_set(&mut self) {
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let mut state_set: HashSet<String> = HashSet::new();
    //     let mut temp_set: HashSet<String> = HashSet::new();
    //     let mut bool_first: bool = true;
    //     let mut key: String;
    //     // JC Filter
    //     let start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i) {
    //             key = cards.iter().map(|c| self.card_to_char(c)).collect::<String>();
    //             println!("key, {}", key);
    //             if bool_first {
    //                 state_set = self.set_store[i][&key].clone();
    //                 bool_first = false;
    //             } else {
    //                 temp_set = self.set_store[i][&key].clone();
    //                 state_set = state_set.intersection(&temp_set).cloned().collect::<HashSet<_>>();
    //             }
    //         }
    //     }
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Set JC Filter: {:?}", elapsed_time);
    //     // PC Filter
    //     let start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             key = self.card_to_char(&card).to_string();
    //             if bool_first {
    //                 state_set = self.set_store[i][&key].clone();
    //                 bool_first = false;
    //             } else {
    //                 temp_set = self.set_store[i][&key].clone();
    //                 state_set = state_set.intersection(&temp_set).cloned().collect::<HashSet<_>>();
    //             }
    //         }
    //     }
    //     // Do a check to convert to vec at the end! store in self.calculated_states
    //     // Before GC
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Set CC Filter: {:?}", elapsed_time);
        
    //     self.calculated_states = state_set.into_iter().collect();
    // }
    pub fn card_to_char(&self, card: &Card) -> char {
        // Example implementation, adjust according to your Card enum and logic
        match card {
            Card::Ambassador => 'A',
            Card::Assassin => 'B',
            Card::Captain => 'C',
            Card::Duke => 'D',
            Card::Contessa => 'E',
        }
    }
    pub fn char_to_card(&self, card_char: char) -> Card {
        match card_char {
            'A' => Card::Ambassador,
            'B' => Card::Assassin,
            'C' => Card::Captain,
            'D' => Card::Duke,
            'E' => Card::Contessa,
            _ => panic!("Bad char used!"),
        }
    }
    // pub fn get_latest_beliefs(&mut self) -> Vec<f64>{
    //     // This is microsecond for small calculated states
    //     // Time 0.17 ~ 0.20 s for full state
    //     // I wish to iterate through Vec<String>
    //     // For each
    //     //TODO: Also store total times each move has been played 
    //      // Can make it output the beliefs
    //     // your trainer takes history and naive_prob, uses PMCCFR to search
    //      // When it reaches a node it needs to collect their states to modify a beliefstate

    //     let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
    //     let mut count: usize = 0;
    //     let mut max_count: usize = 0;
    //     for hand in &self.unique_2p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }

    //     max_count = 6 * count;
    //     count = 0;
    //     // Doing the same but for player 7, the pile which has 3 cards
    //     for hand in &self.unique_3p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     // 35 combinations
    //     // How do you speed the bottom up
    //     max_count += count;
    //     let mut card_freq: Vec<f64> = vec![0.0; max_count];
    //     let mut total_sum: u64 = 0;
    //     for state in &self.calculated_states {
    //         for player_index in 0..7 {
    //             let start_index = self.index_start_arr[player_index];
    //             let end_index = self.index_end_arr[player_index];
    //             let player_hand = &state[start_index..end_index];
                
    //             let card_index = match hand_to_index_offset_map.get(player_hand) {
    //                 Some(index) => *index,
    //                 None => panic!("Invalid card combination"),
    //             };
    //             card_freq[player_index * 15 + card_index] += 1.0; 
    //         }
    //         total_sum += 1;
    //     }
    //     card_freq.iter_mut().for_each(|f| *f /= total_sum as f64);
    //     // Up till here
    //     let key: String = self.make_key_belief();
    //     self.add_to_hashmap(key, card_freq.clone());
    //     card_freq
    // }
    // pub fn get_latest_beliefs_concurrent(&mut self) -> Vec<f64>{
    //     //TIME Significantly reduced for large state sets
    //     // E.g 1.5million full set from 173 ms => 51ms
    //     // Using this and filter_optimal, longest time all in is around 150ms tops down from 300-400ms
    //     // OK what if we dont use filter_optimal first, but we just directly iter through the whole thing from here
    //     // I wish to iterate through Vec<String>
    //     // For each
    //     //TODO: Also store total times each move has been played 
    //      // Can make it output the beliefs
    //     // your trainer takes history and naive_prob, uses PMCCFR to search
    //      // When it reaches a node it needs to collect their states to modify a beliefstate

    //     // Move hand_to_index_map out
    //     let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
    //     let mut count: usize = 0;
    //     let mut max_count: usize = 0;
    //     for hand in &self.unique_2p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     max_count = 6 * count;
    //     count = 0;
    //     // Doing the same but for player 7, the pile which has 3 cards
    //     for hand in &self.unique_3p_hands {
    //         hand_to_index_offset_map.insert(hand.clone(), count);
    //         count += 1;
    //     }
    //     max_count += count;
    //     let card_freq: Arc<Vec<AtomicUsize>> = Arc::new((0..max_count).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());
    //     let total_sum = Arc::new(AtomicUsize::new(0));
    //     let total_sum = self.calculated_states.len() as u64;

    //     self.calculated_states.par_iter().for_each(|state| {
    //         let mut local_counts = vec![0usize; max_count];
    
    //         for player_index in 0..7 {
    //             let start_index = self.index_start_arr[player_index];
    //             let end_index = self.index_end_arr[player_index];
    //             let player_hand = &state[start_index..end_index];
    
    //             if let Some(&card_index) = hand_to_index_offset_map.get(player_hand) {
    //                 local_counts[player_index * 15 + card_index] += 1;
    //             } else {
    //                 panic!("Invalid card combination");
    //             }
    //         }
    
    //         for (card_index, &count) in local_counts.iter().enumerate() {
    //             if count > 0 {
    //                 card_freq[card_index].fetch_add(count, Ordering::SeqCst);
    //             }
    //         }
    //     });
    
    //     let beliefs: Vec<f64> = card_freq.iter()
    //         .map(|freq| freq.load(Ordering::SeqCst) as f64 / total_sum as f64)
    //         .collect();

    //     beliefs
    // }
    pub fn filter_state_simple(&mut self){
        let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    pub fn filter_state_simple_test(&mut self, constraint: &CollectiveConstraint){
        let latest_constraint: CollectiveConstraint = constraint.clone();
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    // pub fn filter_state_optimal(&mut self){
    //     let mut start_time = Instant::now();
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Getting latest_constraint: {:?}", elapsed_time);
    //     start_time = Instant::now();
    //     self.calculated_states = self.all_states.clone();
    //     let elapsed_time = start_time.elapsed();
    //     log::info!("Time taken for Cloning all states: {:?}", elapsed_time);


    //     // let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                     state.len() >= index_end && state_chars == *card_char_vec
    //                 })
    //                 .cloned().collect();
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal JC Filter: {:?}", elapsed_time);
    //     // start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     // start_time = Instant::now();

    //     for card in self.card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             let filtered: Vec<String> = self.calculated_states.par_iter()
    //                 .filter(|state| {
    //                     participating_indices.iter().any(|&(start, end)| {
    //                         state.len() >= end && state[start..end].contains(card_char)
    //                     })
    //                 })
    //                 .cloned().collect();

    //             // Update calculated_states with filtered results
    //             self.calculated_states = filtered;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal GC Filter: {:?}", elapsed_time);
    // }
    // pub fn filter_state_optimal2(&mut self){
    //     // This is usually worse than filter_state_optimal
    //     let latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();

    //     self.calculated_states = self.all_states.clone();

    //     let mut first_filter: bool = true;

    //     // let mut start_time = Instant::now();
    //     for i in 0..6{
    //         if let Some(cards) = latest_constraint.jc_hm.get(&i){
    //             // cards is a Vec<Card> of size 2
    //             // Convert to chars like "AB"
    //             // check that at the current_states state[index_start..index_end] == "AB"
    //             let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             let states_source = if first_filter {
    //                 &self.all_states
    //             } else {
    //                 &self.calculated_states
    //             };
    //             let filtered: Vec<String> = states_source.par_iter()
    //             .filter(|state| {
    //                 let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
    //                 state.len() >= index_end && state_chars == *card_char_vec
    //             })
    //             .cloned().collect();
    //             self.calculated_states = filtered;
    //             first_filter = false;
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal JC Filter: {:?}", elapsed_time);
    //     // start_time = Instant::now();
    //     for i in 0..6 {
    //         if let Some(card) = latest_constraint.pc_hm.get(&i){
    //             let card_char: char = self.card_to_char(&card);
    //             let index_start: usize = 2 * i;
    //             let index_end: usize = index_start + 2;
    //             // filter the string where [index_start: index_end] contains card_char
    //             if first_filter {
    //                 self.calculated_states = self.all_states.par_iter()
    //                 .filter(|state| state[index_start..index_end].contains(card_char))
    //                 .cloned().collect();
    //                 first_filter = false;
    //             } else {

    //                 self.calculated_states.retain(|state| state[index_start..index_end].contains(card_char));
    //             }
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal PC Filter: {:?}", elapsed_time);
    //     // Add concurrent version
    //     // start_time = Instant::now();
    //     for card in self.card_list {
    //         if let Some(participation_list) = latest_constraint.gc_hm.get(&card){
    //             let card_char: char = self.card_to_char(&card);

    //             let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
    //             .filter_map(|(player_id, &participation)| {
    //                 if participation == 1{
    //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
    //                 } else {
    //                     None
    //                 }
    //             }).collect();
    //             if first_filter {

    //                 self.calculated_states = self.all_states.par_iter()
    //                     .filter(|state| {
    //                         participating_indices.iter().any(|&(start, end)| {
    //                             state.len() >= end && state[start..end].contains(card_char)
    //                         })
    //                     })
    //                     .cloned().collect();
    //             } else {

    //                 self.calculated_states = self.calculated_states.par_iter()
    //                     .filter(|state| {
    //                         participating_indices.iter().any(|&(start, end)| {
    //                             state.len() >= end && state[start..end].contains(card_char)
    //                         })
    //                     })
    //                     .cloned().collect();
    //             }

    //             // Eventually change this so if the constraints are empty and no filter is required, to just get belief states to use all_states
    //             if first_filter {
    //                 self.calculated_states = self.all_states.clone();
    //             }
    //         }
    //     }
    //     // let elapsed_time = start_time.elapsed();
    //     // log::info!("Time taken for Optimal GC Filter: {:?}", elapsed_time);
    // }
    pub fn compute_beliefs_direct(&mut self) -> Vec<f64> {
        // Very fast but values slightly different 20~30ms
        // The other versions actually produce wrong values when gc_hm is the only criterion... I dont know why but ill just use this

        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
    
        let mut hand_to_index_offset_map: HashMap<String, usize> = HashMap::new();
        let mut count: usize = 0;
        let mut max_count: usize;
        for hand in &self.unique_2p_hands {
            hand_to_index_offset_map.insert(hand.clone(), count);
            count += 1;
        }
        max_count = 6 * count;
        count = 0;
        // Doing the same but for player 7, the pile which has 3 cards
        for hand in &self.unique_3p_hands {
            hand_to_index_offset_map.insert(hand.clone(), count);
            count += 1;
        }
        max_count += count;
    
        let card_freq: Arc<Vec<AtomicUsize>> = Arc::new((0..max_count).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());
        let total_sum = Arc::new(AtomicUsize::new(0));
    
        self.all_states.par_iter().for_each(|state| {
            if !self.state_satisfies_constraints(state, &latest_constraint) {
                return; // Skip states that do not satisfy the constraints
            }
            total_sum.fetch_add(1, Ordering::SeqCst);
            // Update card frequencies for states that satisfy the constraints
            let mut local_counts = vec![0usize; max_count];

            for player_index in 0..7 {
                let start_index = self.index_start_arr[player_index];
                let end_index = self.index_end_arr[player_index];
                let player_hand = &state[start_index..end_index];
    
                if let Some(&card_index) = hand_to_index_offset_map.get(player_hand) {
                    local_counts[player_index * 15 + card_index] += 1;
                } else {
                    panic!("Invalid card combination");
                }
            }
    
            for (card_index, &count) in local_counts.iter().enumerate() {
                if count > 0 {
                    card_freq[card_index].fetch_add(count, Ordering::SeqCst);
                }
            }
        });
    
        // let total_valid_states = card_freq.iter().map(|freq| freq.load(Ordering::SeqCst)).sum::<usize>();
        let total_valid_states = total_sum.load(Ordering::SeqCst);
        log::info!("Total Valid States {}", total_valid_states);
        let beliefs: Vec<f64> = card_freq.iter()
            .map(|freq| freq.load(Ordering::SeqCst) as f64 / total_valid_states as f64)
            .collect();
    
        beliefs
    }
    
    // Helper method to determine if a state satisfies all constraints
    // TODO: Change back to private
    pub fn state_satisfies_constraints(&self, state: &str, latest_constraint: &CollectiveConstraint) -> bool {
        // println!("Check");
        // Check jc_hm constraints
        for i in 0..6 {
            if let Some(cards) = latest_constraint.jc_hm.get(&i) {
                let card_char_vec: Vec<char> = cards.iter().map(|c| self.card_to_char(c)).collect();
                let index_start: usize = 2 * i;
                let index_end: usize;

                index_end = index_start + 2;
 
    
                if state.len() < index_end {
                    return false;
                }
    
                let state_chars: Vec<char> = state[index_start..index_end].chars().collect();
                if state_chars != card_char_vec {
                    return false; // The state does not satisfy this jc_hm constraint
                }
            }
        }
    
        // Check pc_hm constraints
        for i in 0..6 {
            if let Some(card) = latest_constraint.pc_hm.get(&i) {
                let card_char: char = self.card_to_char(&card);
                let index_start: usize = 2 * i;
                let index_end: usize;

                index_end = index_start + 2;

    
                if state.len() < index_end || !state[index_start..index_end].contains(card_char) {
                    return false; // The state does not satisfy this pc_hm constraint
                }
            }
        }
    
        // This should check that there are gc_hm_count of the card.
        // Check gc_hm constraints
        // for card in &self.card_list {
        //     if let Some(participation_list) = latest_constraint.gc_hm.get(&card) {
        //         let card_char: char = self.card_to_char(&card);
    
        //         let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
        //             .filter_map(|(player_id, &participation)| {
        //                 if participation == 1 {
        //                     Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
        //                 } else {
        //                     None
        //                 }
        //             }).collect();
    
        //         let satisfies_gc_hm = participating_indices.iter().any(|&(start, end)| {
        //             state.len() >= end && state[start..end].contains(card_char)
        //         });
    
        //         if !satisfies_gc_hm {
        //             return false; // The state does not satisfy this gc_hm constraint
        //         }
        //     }
        // }
        
        // Check gc_vec constraints
        let mut index: usize = 0;
        // println!("Before While");
        while index < latest_constraint.gc_vec.len(){
            let participation_list: &[u8; 7] = latest_constraint.gc_vec[index].get_list();
            let card_char: char = latest_constraint.gc_vec[index].card().card_to_char();

            let participating_indices: Vec<(usize, usize)> = participation_list.iter().enumerate()
                    .filter_map(|(player_id, &participation)| {
                        if participation == 1 {
                            Some((self.index_start_arr[player_id], self.index_end_arr[player_id]))
                        } else {
                            None
                        }
                    }).collect();
            let mut total_count = 0;
            let required_count = latest_constraint.gc_vec[index].count();
            let mut satisfies_gc_vec: bool = false;
            // println!("Required Count: {}", required_count);
            // println!("Participation List: {:?}", participation_list);
            // println!("Participation Indices: {:?}", participating_indices);
            for &(start, end) in participating_indices.iter() {
                // println!("Start: {}", start);
                // println!("End: {}", end);
                // println!("State len: {}", state.len());
                if state.len() >= end {
                    total_count += state[start..end].matches(card_char).count();
                    // println!("Total Count: {}", total_count);
                    if total_count >= required_count {
                        satisfies_gc_vec = true;
                        break;
                    }
                }
            }
            if !satisfies_gc_vec {
                return false; // The state does not satisfy this gc_vec constraint
            }
            index += 1;
        }

        true // The state satisfies all constraints
    }
    // pub fn get_leaf_belief(&mut self) -> Vec<f64>{
    //     self.filter_state_optimal();
    //     self.get_latest_beliefs_concurrent()
    // }
    // pub fn gen_and_save_belief(&mut self) {
    //     // self.filter_state_optimal();
    //     let beliefs: Vec<f64> = self.compute_beliefs_direct();
    //     let key: String = self.make_key_belief();
    //     self.add_to_hashmap(key, beliefs.clone());
    // }
    pub fn chance_reveal_redraw(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String>{
        // Hand list is a list of possible hands with valid reach probability we wish to search for
        // "AA" "AB" etc. 
        // Return HashMap of transition hand probabilities 
            // "AA" -> "AA" "AB" "AC" "AD" "AE" use card
            // "AB" -> "AB" "BB" "BC" "BD" "BE" (Revealed A)
            // So its Player hand == "AB" A|XXX 
            // First X is card revealed
            // XXX is pile
            // Condition: Hand is "AB" => ["AB" += 1] and if X is "B" "BB" += 1 etc.
        // Need original reach probabilities too?
        // 2 Cases 2 alive cards and 1 alive card -> Check constraint
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let results = Arc::new(Mutex::new(HashMap::new()));

        let start_time = Instant::now();

        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();
        // 22ms
        self.all_states.par_iter().for_each_with(HashSet::new(), |local_set, state| {
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                // Early local check to minimize lock contention
                if temp_vec_set.contains(state_substring) && !local_set.contains(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        local_set.insert(state_substring.to_string());
                        let mut results_lock = results.lock().unwrap();
                        results_lock.entry(state_substring.to_string()).or_insert_with(|| state.clone());
                    }
                }
            }
        });
        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);
        
        Arc::try_unwrap(results).expect("Failed to unwrap Arc").into_inner().expect("Failed to unlock Mutex")
    }
    pub fn chance_reveal_redraw_exit(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String> {
        // Fastest, use this one
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let results = Arc::new(Mutex::new(HashMap::new()));
        let found_count = Arc::new(AtomicUsize::new(0));
        let should_exit = Arc::new(AtomicBool::new(false));
        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();

        let start_time = Instant::now();

        self.all_states.par_iter().for_each_with((HashSet::new(), Arc::clone(&should_exit)), |(local_set, should_exit), state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if all necessary states have been found.
                return;
            }
            
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                if temp_vec_set.contains(state_substring) && !local_set.contains(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        local_set.insert(state_substring.to_string());
                        let mut results_lock = results.lock().unwrap();
                        if !results_lock.contains_key(state_substring) {
                            results_lock.insert(state_substring.to_string(), state.clone());
                            let count = found_count.fetch_add(1, Ordering::SeqCst) + 1;
                            if count >= temp_vec_set.len() {
                                // Signal to other threads to stop processing.
                                should_exit.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        });

        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);

        Arc::try_unwrap(results).expect("Failed to unwrap Arc").into_inner().expect("Failed to unlock Mutex")
    }
    pub fn chance_reveal_redraw_norm(&mut self, player_id: usize, temp_vec: Vec<String>) -> HashMap<String, String> {
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let start_index: usize = player_id * 2;
        let end_index: usize = start_index + 2;
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let mut results: HashMap<String, String> = HashMap::new();
        let temp_vec_set: HashSet<String> = temp_vec.into_iter().collect();
        let mut found_count = 0;

        let start_time = Instant::now();

        for state in &self.all_states {
            if found_count >= temp_vec_set.len() {
                // Exit early if we've found matches for all items in temp_vec_set
                break;
            }
            
            if start_index < state.len() && end_index <= state.len() {
                let state_substring = &state[start_index..end_index];

                if temp_vec_set.contains(state_substring) && !results.contains_key(state_substring) {
                    if self.state_satisfies_constraints(state, &latest_constraint) {
                        results.insert(state_substring.to_string(), state.clone());
                        found_count += 1;
                    }
                }
            }
        }

        let elapsed_time = start_time.elapsed();
        log::info!("Time Taken to check: {:?}", elapsed_time);

        results
    }
    pub fn chance_sample_exit(&mut self) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // let start_time = Instant::now();
        // This takes around 20 ms instead of the 50 ms normally taken to shuffle
        let mut rng = rand::thread_rng();

        self.all_states.shuffle(&mut rng); // Shuffle in place
        // let elapsed_time = start_time.elapsed();
        // println!("Shuffle Time: {:?}", elapsed_time);

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn chance_sample_exit_test(&mut self, constraint: &CollectiveConstraint) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        // let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // let mut rng = rand::thread_rng();
        // self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_card(&mut self, player_id: usize, card: &Card) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let dead_cards: u8 = latest_constraint.dead_card_count()[card];
        if dead_cards == 3 {
            return None;
        }
        if player_id != 6 {
            latest_constraint.add_raw_public_constraint(player_id, *card);
        } else {
            // Cannot treat the card as dead!
            latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card, 0, 1));
        }
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_card_test(&mut self, constraint: &CollectiveConstraint, player_id: usize, card: &Card) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        // let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        let mut latest_constraint = constraint.clone();
        let dead_cards: u8 = latest_constraint.dead_card_count()[card];
        if dead_cards == 3 {
            return None;
        }
        if player_id != 6 {
            latest_constraint.add_raw_public_constraint(player_id, *card);
        } else {
            // Cannot treat the card as dead!
            latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card, 0, 1));
        }
        let mut rng = rand::thread_rng();
        self.all_states.shuffle(&mut rng); // Shuffle in place

        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }
    pub fn can_player_have_cards(&mut self, player_id: usize, cards: &[Card; 2]) -> Option<String> {
        // Returns None if no String could be found
        // Returns Some(String) if a string that satisfies the constraints could be found
        // Randomly Finds the first string that fulfils the criterion
        // Fastest, use this one
        let mut latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        if player_id == 6 {
            if cards[0] == cards[1] {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0, 2));
            } else {
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[0], 0,1));
                latest_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], cards[1], 0, 1));
            }
        } else {
            // Now test with both constraints
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
        }
        let result = Arc::new(Mutex::new(None));
        let should_exit = Arc::new(AtomicBool::new(false));

        self.all_states.par_iter().for_each_with(Arc::clone(&should_exit), |should_exit, state| {
            if should_exit.load(Ordering::SeqCst) {
                // Early exit if a string has been found.
                return;
            }
            
            if self.state_satisfies_constraints(state, &latest_constraint) {
                let mut result_lock = result.lock().unwrap();
                if result_lock.is_none() {
                    *result_lock = Some(state.to_string());
                    should_exit.store(true, Ordering::SeqCst); // Signal to other threads to stop processing
                    return; // Exit this thread's processing early
                }
            }
        });

        let result_lock = result.lock().unwrap();
        result_lock.clone() // Return the first result if available
    }

    pub fn player_can_have_card(&self, player_id: usize, card: &Card) -> bool {
        // This is the ideal set theory version
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        latest_constraint.player_can_have_active_card(player_id, card)
    }
    pub fn player_can_have_cards(&self, player_id: usize, cards: &[Card; 2]) -> bool {
        let latest_constraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        latest_constraint.player_can_have_active_cards(player_id, cards)
    }
    pub fn filter_player_can_have_card(&mut self, player_id: usize, card: &Card){
        let mut latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
        // Use raw because we are wondering if player can have card and do not know if they actually have the card
        latest_constraint.add_raw_public_constraint(player_id, *card);
        self.calculated_states = self.all_states.par_iter()
            .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
            .cloned()
            .collect();
    }
    pub fn filter_player_can_have_cards(&mut self, player_id: usize, cards: &[Card; 2]){
        self.filter_player_can_have_card(player_id, &cards[0]);
        if self.calc_state_len() != 0 {

            let mut latest_constraint: CollectiveConstraint = self.constraint_history[self.constraint_history.len() - self.prev_index()].clone().unwrap();
            // Use raw because we are wondering if player can have card and do not know if they actually have the card
            // Now test with both constraints
            latest_constraint.add_raw_public_constraint(player_id, cards[0]);
            latest_constraint.add_raw_public_constraint(player_id, cards[1]);
            self.calculated_states = self.all_states.par_iter()
                .filter(|state| self.state_satisfies_constraints(state, &latest_constraint))
                .cloned()
                .collect();
        }
    }

}
