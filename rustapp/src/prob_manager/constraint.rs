use crate::history_public::Card;
use std::collections::{HashMap, HashSet};
// use core::hash::Hasher;
use std::hash::{Hash, Hasher};
use std::usize;
use rand::prelude::SliceRandom;
use std::sync::Mutex;
use rand::thread_rng;
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
        log::info!("{}", format!("Participation List: {:?}", self.participation_list));
        log::info!("{}", format!("Card: {:?}", self.card));
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
    pub fn get_jc_hm(&self) -> &HashMap<usize, Vec<Card>> {
        &self.jc_hm
    }
    pub fn get_pc_hm(&self) -> &HashMap<usize, Card> {
        &self.pc_hm
    }
    pub fn get_gc_vec(&self) -> &Vec<GroupConstraint>{
        &self.gc_vec
    }
    pub fn jc_hm(&mut self) -> &mut HashMap<usize, Vec<Card>> {
        &mut self.jc_hm
    }
    pub fn pc_hm(&mut self) -> &mut HashMap<usize, Card> {
        &mut self.pc_hm
    }
    pub fn gc_vec(&mut self) -> &mut Vec<GroupConstraint>{
        &mut self.gc_vec
    }
    pub fn add_raw_public_constraint(&mut self, player_id: usize, card: Card){
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
        }
        let mut index = 0;
        while index < self.gc_vec.len(){
            let group: &mut GroupConstraint = &mut self.gc_vec[index];
            // [Test]
            if *group.card() == card && group.get_list()[player_id] == 1 {
                group.count_dead_add(1);
                group.count_alive_subtract(1);
                if group.count_alive() == 0 {
                    self.gc_vec.swap_remove(index);
                    continue;
                }
            }
            // if group.indicator(player_id) == 1 && group.count() == 1{
            //     // Public Constraint is subset of group so prune group because group will definitely be fulfilled
            //     // [SUBSET PRUNE]
            //     self.gc_vec.swap_remove(index);
            // } else if self.dead_card_count[group.card()] == 3 {
            else if self.dead_card_count[group.card()] == 3 {
                // [DEAD PRUNE] Prune group if all cards have been shown dead for some card. There are only 3 of each card
                self.gc_vec.swap_remove(index);
            } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                // [COMPLEMENT PRUNE] if group union all public union joint constraint is a full set it just means the card could be anywhere
                self.gc_vec.swap_remove(index);
            } else {
                index += 1;
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
                // KIV: NEW LATE ADDITION 13/07/2024
                if let Some(value) = self.dead_card_count.get_mut(&card){
                    *value -= 1;
                }
            } else {
                debug_assert!(false, "Removing card constraint that does not exist in pc_hm");
            }
        } else if let Some(jc_hm_vec) = self.jc_hm.get_mut(&player_id){
            // Gets joint constraint in jc_hm
            // Removes it and adds remainder to pc_hm
            if let Some(index) = jc_hm_vec.iter().position(|c| *c == card){
                jc_hm_vec.remove(index);
                // KIV: NEW LATE ADDITION 13/07/2024
                if let Some(value) = self.dead_card_count.get_mut(&card){
                    *value -= 1;
                }
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
    pub fn remove_constraints(&mut self, player_id: usize) {
        if let Some(card) = self.pc_hm.get_mut(&player_id) {
            // KIV: NEW LATE ADDITION
            if let Some(value) = self.dead_card_count.get_mut(card){
                *value -= 1;
            }
            self.pc_hm.remove(&player_id);
        }
        if let Some(card_vec) = self.jc_hm.get_mut(&player_id) {
            // KIV: NEW LATE ADDITION
            if card_vec[0] == card_vec[1] {
                if let Some(value) = self.dead_card_count.get_mut(&card_vec[0]){
                    *value -= 2;
                }
            } else {
                if let Some(value) = self.dead_card_count.get_mut(&card_vec[0]){
                    *value -= 1;
                }
                if let Some(value) = self.dead_card_count.get_mut(&card_vec[1]){
                    *value -= 1;
                }
            }
            self.jc_hm.remove(&player_id);
        }
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
            let group = &mut self.gc_vec[index];
            // if group.card() == card && group.get_list()[player_id] == 1 && group.count() <= count{

            if group.card() == card && group.get_list()[player_id] == 1 && group.count() <= new_count{
                // Prune If same card and is subset if old group count <= revealed count
                // Because the group is redundant
                self.gc_vec.swap_remove(index);
            } else if group.card() == card && group.get_list()[player_id] == 1 && group.count() > new_count && bool_card_dead{
                // Adjusting figures appropriately to reflect number alive in the group and number dead
                // In the case of Discard Groups are updated to reflect the value
                // group.count() has to be > new_count else the group would have to be pruned as all information would be represented by public_constraint
                group.count_dead_add(count);
                group.count_alive_subtract(count);
                index += 1;
            } else if self.is_complement_of_pcjc(&self.gc_vec[index]) {
                self.gc_vec.swap_remove(index);
            } else {
                index += 1;
            }
        }
        // if its one dead one alive?
    }
    pub fn group_dead_player_prune(&mut self, player_id: usize, card_vec: &Vec<Card>){
        // Group initial prune should have been used before this, so the dead counts should be correct in group
        // Prunes relevant groups in gc_vec when player loses all their cards
        // log::info!("DEAD PLAYER PRUNE");
        let mut index: usize = 0;
        let mut bool_subtract: bool = false;
        while index < self.gc_vec.len(){
            let group: &mut GroupConstraint = &mut self.gc_vec[index];

            if group.indicator(player_id) == 1 {
                if group.card() != &card_vec[0] && group.card() != &card_vec[1]{
                    // Modify Group whose cards are different!
                    // Their card will not be in the indicator because it is full!
                    // log::trace!("SUBTRACT NO COUNT");
                    group.group_subtract(player_id);
                    bool_subtract = true;
                } else if &card_vec[0] == &card_vec[1]{
                    if group.card() == &card_vec[0] {
                        // log::trace!("SUBTRACT 2");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(2);
                        bool_subtract = true;
                    }
                    if group.count() == 0 {
                        // log::trace!("Unexpected 0 A");
                    }
                } else {
                    if group.card() == &card_vec[0] {
                        // log::trace!("SUBTRACT 1");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(1);
                        bool_subtract = true;
                    }else if group.card() == &card_vec[1] {
                        // log::trace!("SUBTRACT 1");
                        group.group_subtract(player_id);
                        group.count_dead_subtract(1);
                        bool_subtract = true;
                    }
                    if group.count() == 0 {
                        // log::trace!("Unexpected 0 B");
                    }
                }
                // Other cases PRUNED in Initial PRUNE
            }
            if group.count_alive() == 0{
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
                        self.gc_vec.swap_remove(i);
                        i_incremented  = true;
                        break;
                    } else if self.is_redundant(group_j, group_i) {
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
                        // log::trace!("GROUP COUNT HIGH! FIX WITH LEGAL MOVE PRUNE");
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
                        // log::trace!("ExchangeDraw add group 1");
                    } 
                    if group.count_alive() < count {
                        // log::trace!("ExchangeDraw add alive 1");
                        group.count_alive_add(count - group.count_alive());
                    }
                    // Exchange draw count is handled differently because pile is the card that is revealed
                    // if group.count() < new_count {
                        // group.count_add(new_count - group.count());
                        // }
                    // debug_assert!(group.count() < 4, "GROUP COUNT IN EXCHANGEDRAW TOO HIGH");
                } 
            } else {
                if group.get_list()[6] == 1{
                    if group.get_list()[player_id] == 0 {
                        // log::trace!("Exchange constraint do nothing");
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
        let mut index: usize = 0;

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
                            // TODO: Review this it was originally not commented out
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
                            // log::trace!("INFERRED GROUP CLONED!: {:?}", inferred_group);
                            inferred_group.group_subtract(player_id);
                            // inferred_group.count_subtract(max_possible_holdings);
                            inferred_group.count_dead_subtract(dead_count);
                            inferred_group.count_alive_subtract(max_possible_holdings - dead_count);
                            // log::trace!("INFERRED GROUP ADDED!: {:?}", inferred_group);
                            self.gc_vec.push(inferred_group);
                        }
                    }
                }
            }
            // Adding discovery of sets via subsets
            // Checking if subset has been checked already
            // let mut union_part_list_checked: Vec<[u8; 7]> = Vec::new();
            // let mut union_group_counts: HashMap<Card, usize> = HashMap::new();
            // let mut union_group_card_sets: HashMap<Card, [u8; 7]> = HashMap::new();
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
        self.group_redundant_prune();
    }
    pub fn update_group_constraint(&mut self, player_id: usize){
        // For each gc, if player is not a participant, they now become one
        // If all players are participants, remove from tracking
        // Might already be done in exchangedraw
        let mut index: usize = 0;
        while index < self.gc_vec.len() {
            let group = &mut self.gc_vec[index];
            if group.get_list()[6] == 1 {
                if group.get_list()[player_id] == 0 {
                    group.group_add(player_id);
                    // Test to see if redundant
                    if let Some(vcard) = self.pc_hm.get(&player_id) {
                        // If adding the player has a dead card
                        // Including into group includes adding dead counts from that player in
                        if vcard == group.card() {
                            group.count_dead_add(1);
                        }
                    }
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
        let constraint: CollectiveConstraint = self.clone();
        
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
        //[TESTING]
        constraint.add_inferred_groups();

        if constraint.dead_card_count[card] == 3 {
            return false;
        }
        // === Section 1 ===
        // If a particular group has all the remaining alive cards of interest and the player of interested is not part of it, return false
        
        // Number of cards that can be alive remaining
        let remaining_alive_count: usize = 3 - constraint.dead_card_count[card] as usize;


        for group in constraint.gc_vec.iter() {
            // Player may be alive and have one of the cards dead.
            // In this case card: Duke
            // Public Constraint HM: {2: Ambassador, 5: Duke, 3: Ambassador, 4: Ambassador}
            // Joint Constraint HM: {0: [Assassin, Contessa], 1: [Contessa, Contessa]}
            // Group Constraint VEC: [GroupConstraint { participation_list: [0, 0, 0, 0, 1, 1, 1], card: Captain, count: 1 }, GroupConstraint { participation_list: [0, 0, 0, 0, 0, 1, 1], card: Duke, count: 2 }]
            // remaining_alive_count = 2 but player 3 still can reveal a Duke
            // So we adjust the group count downward to prevent double counting the cards that are certain
// 
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
                if group.card() == card && group_alive_count == remaining_alive_count && group.get_list()[player_id] == 0 {
                    // log::trace!("Basic False");
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
        let mut i = 0;

        while i < constraint.gc_vec.len() {
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
                    // Include this just in case [Test Include]
                    if let Some(card_vec) = constraint.jc_hm.get(&iplayer_id) {
                        total_card_count += 2;
                        for card in card_vec.iter() {
                            if let Some(value) = card_count_dead.get_mut(card) {
                                *value += 1;
                            }
                        }
                    }
                }
            }
            let mut j = 0;
            while j < constraint.gc_vec.len() {
                // subsume if possible
                // Checks if j group is subset of i group
                if constraint.gc_vec[j].part_list_is_subset_of(&constraint.gc_vec[i]) {
                    // No need to store participation_list as j is a subset
                    // i part_list Subsuming j will just be i

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
                    if total_card_count == total_card_capacity && constraint.gc_vec[i].get_list()[player_id] == 1 {
                        let alive_count: usize = card_count_alive[card];

                        if alive_count == 0 {
                            // If full group does not contain card of interest, its impossible for a player of that group to have the card
                            // See start of fn => Group will always have player_id indicator as 1 so player_id will be part of the group
                            // log::trace!("Section 2A False");
                            return false
                        } else {
                            // Since group is full, subsuming more groups will not provide more information
                            j += 1;
                            // log::trace!("Full Break A");
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
                        // If all remaining cards of interest are in the group the player of interest is not in 
                        if card_count[card] == remaining_card_count {
                            // log::trace!("Section 2B False");
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
                if total_card_capacity == total_card_count && constraint.gc_vec[i].part_list_is_subset_of(group) && group.count() > card_count[group.card()] && (group.get_list().iter().sum::<u8>() - constraint.gc_vec[i].get_list().iter().sum::<u8>()) == 1 {
                        // Always > 0
                        // let mut alive_count: usize = 3 - card_count[group.card()];
                        let mut alive_count: usize = group.count() - card_count[group.card()];
                        log::trace!("alive_count: {alive_count}");
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
                                        // log::trace!("Section 2 asd False");
                                        return false;
                                    }
                                    //ENsuring player is alive

                                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                                    new_constraint.add_raw_public_constraint(iplayer, *group.card());
                                    
                                    // log::trace!("Section 2 Opposite Subset Recurse");
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section 2 Opposite Subset False");
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
            if total_card_capacity == total_card_count {
                //Ensuring full!

                let mut player_i_possible_cards: HashMap<Card, usize> = HashMap::new();
                // subset_union_card_list stores cards in groups that are subset of group i and exclude i_player-id
                
                let mut subset_union_card_list: HashMap<Card, [u8; 7]> = HashMap::new();
                let mut bool_reset: bool = true;
                let mut bool_mutually_exclusive: bool = false;
                let mut total_cards_remaining: usize = 0;
                let mut total_unique_cards_remaining: usize = 0;
                // log::trace!("Main list : {:?}", constraint.gc_vec[i]);

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
                            // log::trace!("Checking group: {:?}", group);
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
                        }
                    }

                    let group_player_count: u8 = players_left.iter().sum::<u8>();
                    if group_player_count > 2 {
                        bool_reset = true;
                        continue;
                    }
                    // update the player_i_possible_cards to be the alive cards i_player_id can possibly have
                    for (icard, ivalue) in player_i_possible_cards.iter_mut() {
                        *ivalue = card_count_alive[icard] - *ivalue;
                        total_cards_remaining += *ivalue;
                        if *ivalue > 0 {
                            total_unique_cards_remaining += 1;
                        }
                    }
                    if iplayer_id == player_id && player_i_possible_cards[card] == 0 {
                        // log::trace!("Section Subset 0 Immediate return");
                        return false;
                    }
                    if group_player_count == 1 {
                        if total_unique_cards_remaining == 1 && !constraint.jc_hm.contains_key(&iplayer_id){
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            for (vcard, value) in player_i_possible_cards.iter(){
                                if *value > 0 && constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id){
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            // log::trace!("Section Subset 0 Known A True");
                                            return true;
                                        } else {
                                            // The last card that could be alive is not the card of interest!
                                            // log::trace!("Section Subset 0 Known B False");
                                            return false;
                                        }
                                    } 
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section Subset 0 Recurse A False");
                                        return false;
                                    } else {
                                        // log::trace!("Section Subset 0 Recurse A True");
                                        return true;
                                    }
                                } else if *value > 1 && !constraint.pc_hm.contains_key(&iplayer_id) && !constraint.jc_hm.contains_key(&iplayer_id){
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            // log::trace!("Section Subset 0 Known C True");
                                            return true;
                                        } else {
                                            // log::trace!("Section Subset 0 Known D False");
                                            return false;
                                        }
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section Subset 0 Recurse B False");
                                        return false;
                                    } else {
                                        // log::trace!("Section Subset 0 Recurse B True");
                                        return true;
                                    }
                                } else if *value == 1 && !constraint.jc_hm.contains_key(&iplayer_id){
                                    // implicitly pc_hm does not contain a key here! because of the first if condition
                                    // iplayer_id has 2 lives
                                    if iplayer_id == player_id {
                                        if *vcard == *card {
                                            // log::trace!("Section Subset 0 Known E True");
                                            return true;
                                        }
                                        // No return false because they have another card we dont know about!
                                    }
                                    new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section Subset 0 Recurse C False");
                                        return false;
                                    } else {
                                        // log::trace!("Section Subset 0 Recurse C True");
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
                                                // log::trace!("Section Subset 0 Known F True");
                                                return true;
                                            } else if counter == 1 {
                                                // return false only if both cards are not card of interest!
                                                // log::trace!("Section Subset 0 Known G False");
                                                return false;
                                            }
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        counter += 1;
                                    }
                                }
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 0 Recurse D False");
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
                                                // log::trace!("Section Subset 0 Known H True");
                                                return true;
                                            } 
                                            // No return false here because only 1 card will be added!
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 0 Recurse E False");
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
                                                // log::trace!("Section Subset 0 Known I True");
                                                return true;
                                            } 
                                            // No return false here because only 1 card will be added!
                                        }
                                        new_constraint.add_raw_public_constraint(iplayer_id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 0 Recurse E False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    } else if group_player_count == 2 {
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
                        if total_unique_cards_remaining == 1 {
                            if total_cards_remaining == 3 {
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value == 3 {
                                        let mut total_lives: usize = 0;
                                        for (id, indicator) in players_left.iter().enumerate() {
                                            if *indicator == 1 {
                                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                    total_lives += 2;
                                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                    total_lives += 1;
                                                }
                                            }
                                        }
                                        if total_lives == 3 {
                                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                                            for (id, indicator) in players_left.iter().enumerate() { 
                                                // 1 unique card 3 total cards 2 players -> if players have 3 lives collectively, all their empty cards are the card
                                                if *indicator == 1 {
                                                    if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            } else {
                                                                return false;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    } else if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard {
                                                                return true;
                                                            } else {
                                                                return false;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    }
                                                } 
                                            }

                                            // log::trace!("Entering Section Subset 2P Recurse A");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 2P Recurse A False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            } else if total_cards_remaining == 2 {
                                // 1 unique card 2 total cards 2 players -> if both players have 1 life only both players have the 1 of card
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value == 2 {
                                        let mut total_lives: usize = 0;
                                        for (id, indicator) in players_left.iter().enumerate() {
                                            if *indicator == 1 {
                                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                    total_lives += 2;
                                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                    total_lives += 1;
                                                }
                                            }
                                        }
                                        if total_lives == 2 {
                                            // constraint.printlog();
                                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                                            for (id, indicator) in players_left.iter().enumerate() { 
                                                // 1 unique card 3 total cards 2 players -> if players have 3 lives collectively, all their empty cards are the card
                                                if *indicator == 1 {
                                                    if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            } else {
                                                                return false;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    } else if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            } else {
                                                                return false;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    }
                                                } 
                                            }
                                            // log::trace!("Entering Section Subset 2P Recurse B");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 2P Recurse B False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        } else if total_unique_cards_remaining == 2 {
                            // 2 unique cards 4 total cards 2 2 Split 2 players -> both players have 4 lives collectively -> unknown skip
                            // 2 unique cards 4 total cards 3 1 Split 2 players -> both players have 4 lives collectively -> both players have 1 of the card with 3 total cards
                            // 2 unique cards 3 total cards 2 1 Split 2 players -> both players have 3 lives collectively -> player with 2 lives has 1 of the card with 2 total cards
                            if total_cards_remaining == 3 {
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value == 2 {
                                        let mut total_lives: usize = 0;
                                        for (id, indicator) in players_left.iter().enumerate() {
                                            if *indicator == 1 {
                                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                    total_lives += 2;
                                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                    total_lives += 1;
                                                }
                                            }
                                        }
                                        if total_lives == 3 {
                                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                                            for (id, indicator) in players_left.iter().enumerate() { 
                                                // 1 unique card 3 total cards 2 players -> if players have 3 lives collectively, all their empty cards are the card
                                                if *indicator == 1 {
                                                    if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                        break;
                                                    } 
                                                } 
                                            }
                                            // log::trace!("Entering Section Subset 2P Recurse C");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 2P Recurse C False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            } else if total_cards_remaining == 4 {
                                for (vcard, value) in player_i_possible_cards.iter(){
                                    if *value == 3 {
                                        let mut total_lives: usize = 0;
                                        for (id, indicator) in players_left.iter().enumerate() {
                                            if *indicator == 1 {
                                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                    total_lives += 2;
                                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                    total_lives += 1;
                                                }
                                            }
                                        }
                                        if total_lives == 4 {
                                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                                            for (id, indicator) in players_left.iter().enumerate() { 
                                                if *indicator == 1 {
                                                    if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            } else {
                                                                return false;
                                                            }
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    } else if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                        if id == player_id {
                                                            if *card == *vcard{
                                                                return true;
                                                            } 
                                                        }
                                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                                    }
                                                } 
                                            }
                                            // log::trace!("Entering Section Subset 2P Recurse D");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 2P Recurse D False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if group_player_count == 3 {
                        if total_unique_cards_remaining == 1 {
                            // 1 unique card 3 total cards 3 players each have the same card
                            for (vcard, value) in player_i_possible_cards.iter(){
                                if *value == 3 {
                                    let mut total_lives: usize = 0;
                                    for (id, indicator) in players_left.iter().enumerate() {
                                        if *indicator == 1 {
                                            if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                total_lives += 1;
                                            }
                                        }
                                    }
                                    if total_lives == 3 {
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        for (id, indicator) in players_left.iter().enumerate() { 
                                            if *indicator == 1 {
                                                if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                                    if id == player_id {
                                                        if *card == *vcard{
                                                            return true;
                                                        } else {
                                                            return false;
                                                        }
                                                    }
                                                    new_constraint.add_raw_public_constraint(id, *vcard);
                                                } else if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                                    // its possible that not all players are alive anymore
                                                    if id == player_id {
                                                        if *card == *vcard{
                                                            return true;
                                                        } 
                                                    }
                                                    new_constraint.add_raw_public_constraint(id, *vcard);
                                                    new_constraint.add_raw_public_constraint(id, *vcard);
                                                }
                                            } 
                                        }
                                        // log::trace!("Entering Section Subset 2P Recurse E");
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 2P Recurse E False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                }
                            }

                        }
                    }
    
                    bool_reset = true;
                }
            }

            // Next algo begins here!

            // Modifying card_count for a different purpose!
            for (iplayer_id, indicator) in constraint.gc_vec[i].get_list().iter().enumerate(){
                // [TESTING] if need to comment out or not ANS: Needs to be commented out
                // Trying to make it work with this
                if *indicator == 0 {

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

            
            // card_count is now all possible cards alive and outside group i
            // [WHAT IS HAPPENING] We try to infer what a player's hand must have, add a new constraint and recurse!
            if constraint.gc_vec[i].get_list()[player_id] == 0 && card_count[card] == 0{
                // log::trace!("False when player outside group and cant have card!");
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
                    // log::trace!("Section 2 Recurse A Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        // log::trace!("Section 2 Recurse A False");
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
                    // log::trace!("Section 2 Recurse B Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        // log::trace!("Section 2 Recurse B False");
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
                    // log::trace!("Section 2 Recurse C Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        // log::trace!("Section 2 Recurse C False");
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
                    // log::trace!("Section 2 Recurse D Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                        // log::trace!("Section 2 Recurse D False");
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
                            *value += constraint.gc_vec[j].count_alive();
                        }
                    } else {
                        // Either Subset or Some intersection exists
                        if let Some(value) = excl_card_count.get_mut(&constraint.gc_vec[j].card()) {
                            if constraint.gc_vec[j].count_alive() > *value {
                                *value = constraint.gc_vec[j].count_alive();
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
                // if excl_list[k] == 0 {
                    // Include dead players card if they are not in the excl_list group and the player is fully dead
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
                // }
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
            // if possible_count == 0 {
            //     log::trace!("HAVE CARD CASE FOUND DUMMY");
            // }
            if player_id == i {
                // If the player of interest has card of interest return true
                if possible_count == 1 && possible_cards[0] == *card {
                    // log::trace!("Section 3A True");
                    return true;
                } else if possible_count == 1 && possible_cards[0] != *card {
                    // log::trace!("Section 3A False");
                    return false;
                }
                // continue checking if unsure
            } else {
                // Not player of interest
                if possible_count == 1 {
                    // create new constraint and recurse
                    // if recurse value is false return false, else continue
                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                    new_constraint.add_raw_public_constraint(i, possible_cards[0]);

                    // log::trace!("Section 3B Recurse");
                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card) {
                        // log::trace!("Section 3B False");
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
        // === Section 4 ===
        // Checking if the union of some group leads to all cards being known
        // Group i [0, 1, 1, 0, 0, 0, 1]
        // Group new [0, 0, 1, 1, 0, 0, 1]
        // Combine all group information and see if it provides information
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
        let mut card_group_sets: HashMap<Card, [u8; 7]> = HashMap::new();
        let mut union_group_set: [u8; 7] = [0; 7];
        for icard in [Card::Ambassador, Card::Assassin ,Card::Captain, Card::Duke, Card::Contessa].iter(){
            card_group_sets.insert(*icard, [0; 7]);
        }
        let mut total_card_count: usize = 0;
        // Initialise cap for item i, because all other items will be subsumed only if part_list are subsets
        let mut total_card_capacity: usize = 0;
        // Filling them up with everything in the gc_vec!
        for group in constraint.gc_vec.iter(){

            if let Some(union_part_list) = card_group_sets.get_mut(&group.card()) {
                let mut bool_mutual_exclusive: bool = true;
                for icheck in 0..7 {
                    if union_part_list[icheck] == 1 && group.get_list()[icheck] == 1{
                        bool_mutual_exclusive = false;
                        break;
                    }
                }

                if let Some(count_value) = card_count_alive.get_mut(&group.card()) {
                    // TODO: Add dead/alive split
                    // Create another hashmap for dead/alive too
                    if bool_mutual_exclusive {
                        // Consider storing dead and alive in group constraint
                        // Add to both card_count dead and card_count alive based on how many there are
                        *count_value += group.count_alive();
                        total_card_count += group.count_alive();
                    } else {
                        if group.count_alive() > *count_value {
                            total_card_count += group.count_alive() - *count_value;
                            *count_value = group.count_alive();
                        }
                    }
                }

                if let Some(count_value) = card_count.get_mut(&group.card()){
                    *count_value = card_count_alive[&group.card()];
                    // card_count_dead is all 0 so not updated
                }
                for iupdate in 0..7{
                    if union_part_list[iupdate] == 0 && group.get_list()[iupdate] == 1 {
                        union_part_list[iupdate] = 1;
                        union_group_set[iupdate] = 1;
                    }
                }
            }
        }
        // Filling with dead players
        if union_group_set.iter().sum::<u8>() != 0 && union_group_set.iter().sum::<u8>() != 7 {
            for (iplayer_id, indicator) in union_group_set.iter().enumerate(){
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
                    }                    
                    if let Some(card_vec) = constraint.jc_hm.get(&iplayer_id) {
                        total_card_count += 2;
                        for card in card_vec.iter() {
                            if let Some(value) = card_count_dead.get_mut(card) {
                                *value += 1;
                            }
                        }
                    }
                }
            }

            // End of filling card_count 
            // Check if cards for some player outside group (indicator == 0) can be deduced
            if total_card_count == total_card_capacity && union_group_set[player_id] == 1 {
                // If full group does not contain card of interest and player is in it
                if card_count_alive[card] == 0 {
                    return false
                }
            } else if union_group_set[player_id] == 0 {
                // Check for excluded group, what cards are remaining
                // if no remaining card of interest for player
                let mut remaining_card_count_alive: HashMap<Card, usize> = HashMap::with_capacity(5);
                for (vcard, vcount) in card_count.iter(){
                    remaining_card_count_alive.insert(*vcard, 3 - vcount);
                }
                // count of total life remaining for the group outside/complement_to union_group_set
                let mut total_life_remaining: usize = 0;
                for (id, indicator) in union_group_set.iter().enumerate() {
                    let mut temp_life_counter: usize = 2;
                    if *indicator == 0 {
                        if let Some(vcard) = constraint.pc_hm.get(&id){
                            if let Some(vcount) = remaining_card_count_alive.get_mut(vcard){
                                *vcount -= 1;
                                temp_life_counter -= 1;
                            }
                        }
                        if let Some(card_vec) = constraint.jc_hm.get(&id){
                            for vcard in card_vec.iter(){
                                if let Some(vcount) = remaining_card_count_alive.get_mut(vcard){
                                    *vcount -= 1;
                                    temp_life_counter -= 1;
                                }
                            }
                        }
                        total_life_remaining += temp_life_counter;
                    } 
                }
                // count of alive cards remaining for the group outside/complement_to union_group_set
                let mut total_unique_card_remaining: usize = 0;
                let mut total_cards_remaining: usize = 0;
                let mut total_players_remaining: usize = 0; // That are alive
                for (id, indicator) in union_group_set.iter().enumerate() {
                    if *indicator == 0 {
                        if !constraint.jc_hm.contains_key(&id) {
                            total_players_remaining += 1;
                        }
                    }
                }
                for vcount in remaining_card_count_alive.values() {
                    if *vcount > 0 {
                        total_unique_card_remaining += 1;
                        total_cards_remaining += *vcount;
                    }
                }
                if total_players_remaining == 1 {
                    if total_life_remaining == 1 && total_cards_remaining == 1{
                        for (id, ind) in union_group_set.iter().enumerate() {
                            if *ind == 0 && !constraint.jc_hm.contains_key(&id){
                                for (vcard, vcount) in remaining_card_count_alive.iter(){
                                    if *vcount == 1 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        // log::trace!("Entering Section 4 Out Recurse A");
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4 Out Recurse A False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    } else if total_life_remaining == 2 && total_cards_remaining == 2{
                        if total_unique_card_remaining == 1 {
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 2 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            }
                                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            // log::trace!("Entering Section 4 Out Recurse B");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 4 Out Recurse B False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    // if false return false
                                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                                    let mut temp_counter: usize = 0;
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 1 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    if temp_counter == 1 {
                                                        // Both cards arent card of interest!
                                                        return false;
                                                    }
                                                    temp_counter += 1;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                        }
                                    }
                                    // log::trace!("Entering Section 4 Out Recurse C");
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section Subset 4 Out Recurse C False");
                                        return false;
                                    } else {
                                        return true;
                                    }
                                }
                            }

                        }

                    }
                } else if total_players_remaining == 2 {
                    if total_unique_card_remaining == 1 {
                        if total_cards_remaining == 2 && total_life_remaining == 2{
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            let mut bool_recurse: bool = false;
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 2 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            bool_recurse = true;
                                        }
                                    }
                                }
                            }
                            if bool_recurse{
                                // log::trace!("Entering Section 4 Out Recurse D");
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4 Out Recurse D False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } else if total_cards_remaining == 3 && total_life_remaining == 3{
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            let mut bool_recurse: bool = false;
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 3 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            bool_recurse = true;
                                        }
                                    }
                                } else if *ind == 0 && !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 3 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            bool_recurse = true;
                                        }
                                    }
                                }
                            }
                            if bool_recurse{
                                // log::trace!("Entering Section 4 Out Recurse E");
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4 Out Recurse E False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        }

                    } else if total_unique_card_remaining == 2 {
                        if total_cards_remaining == 3 && total_life_remaining == 3 {
                            // 2 1 split, player with 2 lives has the card with count of 2
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 2 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } 
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            // log::trace!("Entering Section 4 Out Recurse F");
                                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                                // log::trace!("Section Subset 4 Out Recurse F False");
                                                return false;
                                            } else {
                                                return true;
                                            }
                                        }
                                    }
                                } 
                            }

                        } else if total_cards_remaining == 4 && total_life_remaining == 4{
                            // 3 1 Split, both players have 1 of card with count of 3
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            let mut bool_recurse: bool = false;
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 3 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            bool_recurse = true;
                                            break;
                                        }
                                    }
                                } 
                            }
                            if bool_recurse{
                                // log::trace!("Entering Section 4 Out Recurse G");
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4 Out Recurse G False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        }
                    }
                } else if total_players_remaining == 3 {
                    if total_unique_card_remaining == 1 && total_cards_remaining == 3 && total_life_remaining == 3{
                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                            let mut bool_recurse: bool = false;
                            for (id, ind) in union_group_set.iter().enumerate() {
                                if *ind == 0 && constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id){
                                    for (vcard, vcount) in remaining_card_count_alive.iter(){
                                        if *vcount == 3 {
                                            if id == player_id {
                                                if *vcard == *card {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            }
                                            new_constraint.add_raw_public_constraint(id, *vcard);
                                            bool_recurse = true;
                                        }
                                    }
                                } 
                            }
                            if bool_recurse{
                                // log::trace!("Entering Section 4 Out Recurse H");
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4 Out Recurse H False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                    }
                }
            }
            // === Section 4B ===
            // Check for each player with indicator group if excluding all groups except for the current player
            // Whether the current player card can be deduced
            // for each player with union set indicator == 1 make a new hashmap and remove all except for that player
            // See player must have a certain card
            if total_card_capacity == total_card_count {
                // Create new HashMap
                for id in 0..6 as usize {
                    if union_group_set[id] == 0 {
                        continue;
                    }
                    let mut possible_cards: HashMap<Card, usize> = HashMap::with_capacity(5);
                    possible_cards.insert(Card::Ambassador, 0);
                    possible_cards.insert(Card::Assassin, 0);
                    possible_cards.insert(Card::Captain, 0);
                    possible_cards.insert(Card::Duke, 0);
                    possible_cards.insert(Card::Contessa, 0);
                    let mut card_group_sets: HashMap<Card, [u8; 7]> = HashMap::new();
                    for icard in [Card::Ambassador, Card::Assassin ,Card::Captain, Card::Duke, Card::Contessa].iter(){
                        card_group_sets.insert(*icard, [0; 7]);
                    }
                    // Adding cards for groups that have id == 1
                    // To find cards possible for player id
                    for group in constraint.gc_vec.iter(){
                        if group.get_list()[id] == 0 {
                            continue;
                        }
            
                        if let Some(union_part_list) = card_group_sets.get_mut(&group.card()) {
                            let mut bool_mutual_exclusive: bool = true;
                            for icheck in 0..7 {
                                if union_part_list[icheck] == 1 && group.get_list()[icheck] == 1{
                                    bool_mutual_exclusive = false;
                                    break;
                                }
                            }
            
                            if let Some(count_value) = possible_cards.get_mut(&group.card()) {
                                // TODO: Add dead/alive split
                                // Create another hashmap for dead/alive too
                                if bool_mutual_exclusive {
                                    // Consider storing dead and alive in group constraint
                                    // Add to both card_count dead and card_count alive based on how many there are
                                    *count_value += group.count_alive();
                                    total_card_count += group.count_alive();
                                } else {
                                    if group.count_alive() > *count_value {
                                        total_card_count += group.count_alive() - *count_value;
                                        *count_value = group.count_alive();
                                    }
                                }
                            }
            
                            if let Some(count_value) = card_count.get_mut(&group.card()){
                                *count_value = possible_cards[&group.card()];
                                // card_count_dead is all 0 so not updated
                            }
                            for iupdate in 0..7{
                                if union_part_list[iupdate] == 0 && group.get_list()[iupdate] == 1 {
                                    union_part_list[iupdate] = 1;
                                    // union_group_set[iupdate] = 1;
                                }
                            }
                        }
                    }
                    // // Storing no of cards alive remaining for the player
                    let mut alive_remaining: usize = 0;
                    let mut unique_alive_remaining: usize = 0;
                    for val in possible_cards.values(){
                        if *val > 0 {
                            unique_alive_remaining += 1;
                            alive_remaining += *val;
                        }
                    }
                    // if id == player_id cannot have card of interest return false
                    if id == player_id && possible_cards[card] == 0{
                        // log::trace!("Section 4B Initial Return False");
                        return false;
                    }
                    if unique_alive_remaining == 1 {
                        if alive_remaining == 1 && constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                            for (vcard, vcount) in possible_cards.iter() {
                                if *vcount >= 1 {
                                    if id == player_id {
                                        if *vcard == *card {
                                            return true;
                                        } else {
                                            return false;
                                        }
                                    }
                                    let mut new_constraint: CollectiveConstraint = constraint.clone();
                                    // log::trace!("Entering Section 4B Recurse A");
                                    new_constraint.add_raw_public_constraint(id, *vcard);
                                    if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                        // log::trace!("Section Subset 4B Recurse A False");
                                        return false;
                                    } else {
                                        return true;
                                    }
                                }
                            }
                        } else if alive_remaining == 2 {
                            for (vcard, vcount) in possible_cards.iter() {
                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                    if *vcount >= 2 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse B");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse B False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                    if *vcount >= 2 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse C");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse C False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }

                                }
                            }
                        } else if alive_remaining == 3 {
                            for (vcard, vcount) in possible_cards.iter() {
                                if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                    if *vcount == 3 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse D");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse D False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                    if *vcount == 3 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse E");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse E False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }

                                }
                            }

                        }
                    } else if unique_alive_remaining == 2 {
                        if alive_remaining == 2 {
                            if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                let mut counter: usize = 0;
                                for (vcard, value) in possible_cards.iter(){
                                    if *value == 1 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                // log::trace!("Section Subset 0 Known F True");
                                                return true;
                                            } else if counter == 1 {
                                                // return false only if both cards are not card of interest!
                                                // log::trace!("Section Subset 0 Known G False");
                                                return false;
                                            }
                                        }
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        counter += 1;
                                    }
                                }
                                // log::trace!("Entering Section 4B Recurse F");
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section 4B Recurse F False");
                                    return false;
                                } else {
                                    return true;
                                }
                            } 
                        } else if alive_remaining == 3 {
                            // 2 1 split => player with 2 lives has card with count of 2
                            if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                for (vcard, vcount) in possible_cards.iter() {
                                    if *vcount == 2 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } 
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse G");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse G False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                } 
                            }
                        } else if alive_remaining == 4 {
                            // 3 1 split => if player has 2 lives they have card with count of 3
                            if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                                for (vcard, vcount) in possible_cards.iter() {
                                    if *vcount == 3 {
                                        if id == player_id {
                                            if *vcard == *card {
                                                return true;
                                            } 
                                        }
                                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                                        // log::trace!("Entering Section 4B Recurse H");
                                        new_constraint.add_raw_public_constraint(id, *vcard);
                                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                            // log::trace!("Section Subset 4B Recurse H False");
                                            return false;
                                        } else {
                                            return true;
                                        }
                                    }
                                } 
                            }
                        }
                    }
                }
            }
        }
        // === Section 4C ===
        // Fill up group by finding the complement group of all dead players
        // initialise card count
        // card_count_alive stores only the alive cards i the group union_group_sets
        let mut card_count_alive: HashMap<Card, usize> = HashMap::with_capacity(5);
        let mut union_group_set: [u8; 7] = [1; 7];
        card_count_alive.insert(Card::Ambassador, 3);
        card_count_alive.insert(Card::Assassin, 3);
        card_count_alive.insert(Card::Captain, 3);
        card_count_alive.insert(Card::Duke, 3);
        card_count_alive.insert(Card::Contessa, 3);
        for id in 0..6 as usize {
            if let Some(card_vec) = constraint.jc_hm.get(&id) {
                for vcard in card_vec.iter() {
                    if let Some(value) = card_count_alive.get_mut(vcard) {
                        *value -= 1;
                    }
                }
                union_group_set[id] = 0;
            }
            if let Some(vcard) = constraint.pc_hm.get(&id) {
                if let Some(value) = card_count_alive.get_mut(vcard) {
                    *value -= 1;
                }
            }
        }
        // End of filling card_count 
        // Check if cards for some player outside group (indicator == 0) can be deduced
        if union_group_set[player_id] == 1 {
            // If full group does not contain card of interest and player is in it
            // Pretty sure this is the case where all teh cards are dead, but its left here for completion sake
            if card_count_alive[card] == 0 {
                return false
            }
        }
        for id in 0..6 as usize {
            if union_group_set[id] == 0 {
                continue;
            }
            // not possible_cards stores the alive cards that are in the groups that do not have id ind == 1
            let mut not_possible_cards: HashMap<Card, usize> = HashMap::with_capacity(5);
            not_possible_cards.insert(Card::Ambassador, 0);
            not_possible_cards.insert(Card::Assassin, 0);
            not_possible_cards.insert(Card::Captain, 0);
            not_possible_cards.insert(Card::Duke, 0);
            not_possible_cards.insert(Card::Contessa, 0);
            let mut card_group_sets: HashMap<Card, [u8; 7]> = HashMap::new();
            for icard in [Card::Ambassador, Card::Assassin ,Card::Captain, Card::Duke, Card::Contessa].iter(){
                card_group_sets.insert(*icard, [0; 7]);
            }
            for group in constraint.gc_vec.iter() {
                if group.get_list()[id] == 1 {
                    continue;
                }
                if let Some(union_part_list) = card_group_sets.get_mut(&group.card()) {
                    let mut bool_mutual_exclusive: bool = true;
                    for icheck in 0..7 {
                        if union_part_list[icheck] == 1 && group.get_list()[icheck] == 1{
                            bool_mutual_exclusive = false;
                            break;
                        }
                    }
    
                    if let Some(count_value) = not_possible_cards.get_mut(&group.card()) {
                        // TODO: Add dead/alive split
                        // Create another hashmap for dead/alive too
                        if bool_mutual_exclusive {
                            // Consider storing dead and alive in group constraint
                            // Add to both card_count dead and card_count alive based on how many there are
                            *count_value += group.count_alive();
                            total_card_count += group.count_alive();
                        } else {
                            if group.count_alive() > *count_value {
                                total_card_count += group.count_alive() - *count_value;
                                *count_value = group.count_alive();
                            }
                        }
                    }
    
                    if let Some(count_value) = card_count.get_mut(&group.card()){
                        *count_value = not_possible_cards[&group.card()];
                        // card_count_dead is all 0 so not updated
                    }
                    for iupdate in 0..7{
                        if union_part_list[iupdate] == 0 && group.get_list()[iupdate] == 1 {
                            union_part_list[iupdate] = 1;
                            // union_group_set[iupdate] = 1;
                        }
                    }
                }
            }
            let mut alive_remaining: usize = 0;
            let mut unique_alive_remaining: usize = 0;
            let mut possible_cards: HashMap<Card, usize> = HashMap::with_capacity(5);
            for (vcard, vcount) in not_possible_cards.iter(){
                let insertion_val: usize = card_count_alive[vcard] - *vcount;
                possible_cards.insert(*vcard, card_count_alive[vcard] - *vcount);
                if insertion_val > 0 {
                    unique_alive_remaining += 1;
                    alive_remaining += insertion_val;
                }
            }
            if id == player_id && possible_cards[card] == 0{
                // log::trace!("Section 4B Copy Initial Return False");
                return false;
            }
            if unique_alive_remaining == 1 {
                if alive_remaining == 1 && constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                    for (vcard, vcount) in possible_cards.iter() {
                        if *vcount >= 1 {
                            if id == player_id {
                                if *vcard == *card {
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                            let mut new_constraint: CollectiveConstraint = constraint.clone();
                            // log::trace!("Entering Section 4B Copy Recurse A");
                            new_constraint.add_raw_public_constraint(id, *vcard);
                            if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                // log::trace!("Section Subset 4B Copy Recurse A False");
                                return false;
                            } else {
                                return true;
                            }
                        }
                    }
                } else if alive_remaining == 2 {
                    for (vcard, vcount) in possible_cards.iter() {
                        if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                            if *vcount >= 2 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } else {
                                        return false;
                                    }
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Copy Recurse B");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Copy Recurse B False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                            if *vcount >= 2 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } else {
                                        return false;
                                    }
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Copy Recurse C");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Copy Recurse C False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }

                        }
                    }
                } else if alive_remaining == 3 {
                    for (vcard, vcount) in possible_cards.iter() {
                        if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                            if *vcount == 3 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } else {
                                        return false;
                                    }
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Copy Recurse D");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Copy Recurse D False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } else if constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                            if *vcount == 3 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } else {
                                        return false;
                                    }
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Copy Recurse E");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Copy Recurse E False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }

                        }
                    }

                }
            } else if unique_alive_remaining == 2 {
                if alive_remaining == 2 {
                    if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                        let mut new_constraint: CollectiveConstraint = constraint.clone();
                        let mut counter: usize = 0;
                        for (vcard, value) in possible_cards.iter(){
                            if *value == 1 {
                                if id == player_id {
                                    if *vcard == *card {
                                        // log::trace!("Section Subset 0 Known F True");
                                        return true;
                                    } else if counter == 1 {
                                        // return false only if both cards are not card of interest!
                                        // log::trace!("Section Subset 0 Known G False");
                                        return false;
                                    }
                                }
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                counter += 1;
                            }
                        }
                        // log::trace!("Entering Section 4B Copy Recurse F");
                        if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                            // log::trace!("Section 4B Copy Recurse F False");
                            return false;
                        } else {
                            return true;
                        }
                    } 
                } else if alive_remaining == 3 {
                    // 2 1 split => player with 2 lives has card with count of 2
                    if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                        for (vcard, vcount) in possible_cards.iter() {
                            if *vcount == 2 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } 
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Recurse G");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Recurse G False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } 
                    }
                } else if alive_remaining == 4 {
                    // 3 1 split => if player has 2 lives they have card with count of 3
                    if !constraint.pc_hm.contains_key(&id) && !constraint.jc_hm.contains_key(&id) {
                        for (vcard, vcount) in possible_cards.iter() {
                            if *vcount == 3 {
                                if id == player_id {
                                    if *vcard == *card {
                                        return true;
                                    } 
                                }
                                let mut new_constraint: CollectiveConstraint = constraint.clone();
                                // log::trace!("Entering Section 4B Recurse H");
                                new_constraint.add_raw_public_constraint(id, *vcard);
                                if !CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card){
                                    // log::trace!("Section Subset 4B Recurse H False");
                                    return false;
                                } else {
                                    return true;
                                }
                            }
                        } 
                    }
                }
            }
        }

        true
    }
    pub fn printlog(&self) {
        log::info!("{}", format!("Public Constraint HM: {:?}", self.pc_hm));
        log::info!("{}", format!("Joint Constraint HM: {:?}", self.jc_hm));
        log::info!("{}", format!("Group Constraint VEC: {:?}", self.gc_vec));
    }
    pub fn print(&self) {
        println!("{}", format!("Public Constraint HM: {:?}", self.pc_hm));
        println!("{}", format!("Joint Constraint HM: {:?}", self.jc_hm));
        println!("{}", format!("Group Constraint VEC: {:?}", self.gc_vec));
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
    pub fn player_can_have_active_cards_str(&self, player_id: usize, cards: &str) -> bool {
        assert!(cards.len() == 2, "Please only use len of 2");
        let card0: &Card = &Card::char_to_card(cards.chars().nth(0).unwrap());
        let card1: &Card = &Card::char_to_card(cards.chars().nth(1).unwrap());
        if self.player_can_have_active_card(player_id, card0){
            let mut new_constraint: CollectiveConstraint = self.clone();
            if player_id == 6 {
                // Add case for when both cards are the same!
                // If both are the same it will return legal if 1 card works because I dont add [0, 0, 0, 0, 0, 0, 1] count: 2
                if *card0 != *card1 {
                    new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], *card0, 0, 1));
                } else {
                    // When they are same, we put both constraints and see if can have the card? I dont think this should work tho?
                    // CURRENTLY NO PROPER FUNCTIONALITY IMPLEMENTED FOR TESTING IF PILE HAS 2 OF THE SAME CARD
                    // ITS NOT NEEDED FOR ALGO SO THIS DISCLAIMER IS 
                    // THIS IS A TEMP FIX THAT MAY WORK BUT LEADS TO USIZE OVERFLOW
                    // new_constraint.add_raw_group(GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], card0, 2));
                    return true;
                }
            } else {
                new_constraint.add_raw_public_constraint(player_id, *card0);
            }
            CollectiveConstraint::player_can_have_active_card_pub(&new_constraint, player_id, card1)
        } else {
            false
        }
    }
}

mod constrainttest {
    use crate::prob_manager::constraint::{CollectiveConstraint, GroupConstraint, Card};
    #[test]
    fn test0() {
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
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Assassin);
        assert!(output);
        // if output {
        //     println!("Test 7 Legal Correct");
        // } else {
        //     println!("Test 7 Illegal Wrong");
        // }
    }
    #[test]
    fn test1() {
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
    
        // log::info!(" === Test 8 === ");
        // colcon.printlog();
        let dead_hm = colcon.dead_card_count();
        // log::trace!("dead_card_count: {:?}", dead_hm);
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Ambassador);
        assert!(!output);
    
        // if output {
        //     println!("Test 8 Legal Wrong");
        // } else {
        //     println!("Test 8 Illegal Correct");
        // }
    }
    #[test]
    fn test2() {
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
    
        // log::info!(" === Test 9 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 5, &Card::Contessa);
        assert!(output);
        // if output {
        //     println!("Test 9 Legal Correct");
        // } else {
        //     println!("Test 9 Illegal Wrong");
        // }
    }
    #[test]
    fn test3() {
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
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Assassin);
        assert!(output);
    
        // if output {
        //     println!("Test 10 Legal Correct");
        // } else {
        //     println!("Test 10 Illegal Wrong");
        // }
    }
    #[test]
    fn test4() {
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
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Ambassador);
        assert!(!output);
        // if output {
        //     println!("Test 11 Legal Wrong");
        // } else {
        //     println!("Test 11 Illegal Correct");
        // }
    }
    #[test]
    fn test5() {
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
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Assassin);
        assert!(!output);
        // if output {
        //     println!("Test 12 Legal Wrong");
        // } else {
        //     println!("Test 12 Illegal Correct");
        // }
    }
    #[test]
    fn test6() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(5, Card::Captain);
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(4, Card::Captain);
        colcon.add_public_constraint(4, Card::Assassin);
        colcon.add_public_constraint(0, Card::Assassin);
        colcon.add_public_constraint(0, Card::Contessa);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Captain);
        colcon.add_public_constraint(2, Card::Assassin);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Ambassador, 0, 1 );
        let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Contessa, 1, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
    
    
        log::info!(" === Test 13 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Duke);
        assert!(output);
        // if output {
        //     println!("Test 13 Legal Correct");
        // } else {
        //     println!("Test 13 Illegal Wrong");
        // }
    }
    #[test]
    fn test7() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(5, Card::Captain);
    
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(4, Card::Assassin);
        colcon.add_public_constraint(4, Card::Assassin);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 3 );
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Assassin, 0, 1);
        let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Contessa, 0, 2);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
    
        log::info!(" === Test 14 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Contessa);
        assert!(output);
        // if output {
        //     println!("Test 14 Legal Correct");
        // } else {
        //     println!("Test 14 Illegal Wrong");
        // }
    }
    #[test]
    fn test8() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(1, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Contessa);
        colcon.add_public_constraint(3, Card::Assassin);
    
        colcon.add_public_constraint(0, Card::Assassin);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(4, Card::Assassin);
        colcon.add_public_constraint(4, Card::Captain);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Captain, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Duke, 0, 1 );
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 2);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Ambassador, 0, 1);
        let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Contessa, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
    
        log::info!(" === Test 15 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Duke);
        assert!(!output);
        // if output {
        //     println!("Test 15 Legal Wrong");
        // } else {
        //     println!("Test 15 Illegal Correct");
        // }
    }
    #[test]
    fn test9() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(0, Card::Ambassador);
        colcon.add_public_constraint(5, Card::Captain);
        colcon.add_public_constraint(2, Card::Assassin);
    
        colcon.add_public_constraint(3, Card::Assassin);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(1, Card::Assassin);
        colcon.add_public_constraint(1, Card::Ambassador);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Duke, 0, 1);
        let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Captain, 0, 2 );
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 2);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
    
        log::info!(" === Test 16 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 0, &Card::Duke);
        assert!(!output);
        // if output {
        //     println!("Test 16 Legal Wrong");
        // } else {
        //     println!("Test 16 Illegal Correct");
        // }
    }
    #[test]
    fn test10() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(2, Card::Duke);
        colcon.add_public_constraint(3, Card::Contessa);
    
        colcon.add_public_constraint(0, Card::Assassin);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(1, Card::Duke);
        colcon.add_public_constraint(1, Card::Ambassador);
        colcon.add_public_constraint(5, Card::Ambassador);
        colcon.add_public_constraint(5, Card::Assassin);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 3);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 2 );
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
    
        log::info!(" === Test 17 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Ambassador);
        assert!(!output)
        // if output {
        //     println!("Test 17 Legal Wrong");
        // } else {
        //     println!("Test 17 Illegal Correct");
        // }
    }
    #[test]
    fn test11() {
        let mut colcon = CollectiveConstraint::new();
    
    
        colcon.add_public_constraint(2, Card::Contessa);
        colcon.add_public_constraint(4, Card::Duke);
        colcon.add_public_constraint(5, Card::Captain);
        colcon.add_public_constraint(3, Card::Assassin);
        colcon.add_public_constraint(0, Card::Assassin);
    
        colcon.add_public_constraint(1, Card::Duke);
        colcon.add_public_constraint(1, Card::Ambassador);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Duke, 0, 1);
        let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Assassin, 1, 1);
        let group5: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Contessa, 1, 1);
        let group6: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
        colcon.add_raw_group(group6);
    
        log::info!(" === Test 18 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Contessa);
        assert!(!output);
        // if output {
        //     println!("Test 18 Legal Wrong");
        // } else {
        //     println!("Test 18 Illegal Correct");
        // }
    }
    #[test]
    fn test12() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(3, Card::Assassin);
        colcon.add_public_constraint(5, Card::Contessa);
    
        colcon.add_public_constraint(2, Card::Captain);
        colcon.add_public_constraint(2, Card::Duke);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(1, Card::Captain);
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(4, Card::Captain);
        colcon.add_public_constraint(4, Card::Assassin);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Contessa, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Assassin, 1, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
    
        // log::info!(" === Test 19 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Ambassador);
        assert!(output);
        // if output {
        //     println!("Test 19 Legal Correct");
        // } else {
        //     println!("Test 19 Illegal Wrong");
        // }
    }
    #[test]
    fn test13() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(5, Card::Assassin);
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(2, Card::Captain);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(4, Card::Assassin);
    
        colcon.add_public_constraint(0, Card::Assassin);
        colcon.add_public_constraint(0, Card::Duke);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Ambassador, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Contessa, 1, 2);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Captain, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
    
        // log::info!(" === Test 20 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Captain);
        assert!(!output);
        // if output {
        //     println!("Test 20 Legal Wrong");
        // } else {
        //     println!("Test 20 Illegal Correct");
        // }
    }
    #[test]
    fn test14() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(5, Card::Ambassador);
        
        colcon.add_public_constraint(0, Card::Ambassador);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(2, Card::Contessa);
        colcon.add_public_constraint(2, Card::Contessa);
        colcon.add_public_constraint(1, Card::Captain);
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(4, Card::Ambassador);
        colcon.add_public_constraint(4, Card::Duke);
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Duke, 0, 1);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 2);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Assassin, 0, 2);
         
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
    
        // log::info!(" === Test 21 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Assassin);
        assert!(output);
        // if output {
        //     println!("Test 21 Legal Correct");
        // } else {
        //     println!("Test 21 Illegal Wrong");
        // }
    }
    #[test]
    fn test15() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(2, Card::Ambassador);
        colcon.add_public_constraint(0, Card::Captain);
        colcon.add_public_constraint(3, Card::Duke);
    
        colcon.add_public_constraint(5, Card::Contessa);
        colcon.add_public_constraint(5, Card::Assassin);
    
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(4, Card::Duke);
        
        let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 1);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
        let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Contessa, 0, 1);
        let group6: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
        colcon.add_raw_group(group6);
    
        log::info!(" === Test 22 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 0, &Card::Assassin);
        assert!(!output);
        // if output {
        //     println!("Test 22 Legal Wrong");
        // } else {
        //     println!("Test 22 Illegal Correct");
        // }
    }
    #[test]
    fn test16() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(5, Card::Contessa);
    
        colcon.add_public_constraint(0, Card::Ambassador);
        colcon.add_public_constraint(0, Card::Assassin);
        colcon.add_public_constraint(1, Card::Ambassador);
        colcon.add_public_constraint(1, Card::Assassin);
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(2, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Assassin);
    
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 3);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke, 0, 2);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
    
        // log::info!(" === Test 23 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 3, &Card::Duke);
        assert!(output);
        // if output {
        //     println!("Test 23 Legal Correct");
        // } else {
        //     println!("Test 23 Illegal Wrong");
        // }
    }
    #[test]
    fn test17() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(0, Card::Ambassador);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Duke);
        colcon.add_public_constraint(2, Card::Assassin);
        colcon.add_public_constraint(5, Card::Assassin);
        colcon.add_public_constraint(5, Card::Duke);
    
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Captain, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Assassin, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Contessa, 1, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
    
        // log::info!(" === Test 24 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 1, &Card::Captain);
        assert!(output);
        // if output {
        //     println!("Test 24 Legal Correct");
        // } else {
        //     println!("Test 24 Illegal Wrong");
        // }
    }
    #[test]
    fn test18() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(0, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Contessa);
        colcon.add_public_constraint(5, Card::Duke);
    
        colcon.add_public_constraint(1, Card::Contessa);
        colcon.add_public_constraint(1, Card::Assassin);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(3, Card::Contessa);
    
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Duke, 1, 1);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Assassin, 0, 2);
        let group3: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 1);
        let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 2);
        let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
    
        // log::info!(" === Test 25 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 5, &Card::Captain);
        assert!(!output);
        // if output {
        //     println!("Test 25 Legal Wrong");
        // } else {
        //     println!("Test 25 Illegal Correct");
        // }
    }
    #[test]
    fn test19 () {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(5, Card::Contessa);
    
        colcon.add_public_constraint(2, Card::Assassin);
        colcon.add_public_constraint(2, Card::Assassin);
        colcon.add_public_constraint(4, Card::Ambassador);
        colcon.add_public_constraint(4, Card::Duke);
        colcon.add_public_constraint(3, Card::Ambassador);
        colcon.add_public_constraint(3, Card::Assassin);
        colcon.add_public_constraint(1, Card::Ambassador);
        colcon.add_public_constraint(1, Card::Contessa);
    
        
        let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Contessa, 0, 1);
        let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Duke, 1, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Captain, 0, 2);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
    
        // log::info!(" === Test 26 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 0, &Card::Captain);
        assert!(output);
        // if output {
        //     println!("Test 26 Legal Correct");
        // } else {
        //     println!("Test 26 Illegal Wrong");
        // }
    }
    #[test]
    fn test20() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(2, Card::Ambassador);
        colcon.add_public_constraint(5, Card::Captain);
    
        colcon.add_public_constraint(1, Card::Captain);
        colcon.add_public_constraint(1, Card::Duke);
        colcon.add_public_constraint(0, Card::Captain);
        colcon.add_public_constraint(0, Card::Duke);
        colcon.add_public_constraint(4, Card::Contessa);
        colcon.add_public_constraint(4, Card::Duke);
        
        
        let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 1, 1], Card::Ambassador, 0, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 2);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Assassin, 0, 2);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
    
        // log::info!(" === Test 27 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 2, &Card::Assassin);
        assert!(!output);
        // if output {
        //     println!("Test 27 Legal Wrong");
        // } else {
        //     println!("Test 27 Illegal Correct");
        // }
    }
    #[test]
    fn test21() {
        let mut colcon = CollectiveConstraint::new();
    
        colcon.add_public_constraint(4, Card::Ambassador);
        colcon.add_public_constraint(2, Card::Assassin);
        colcon.add_public_constraint(5, Card::Captain);
    
       
        
        let group1: GroupConstraint = GroupConstraint::new_list([1, 1, 0, 0, 1, 0, 1], Card::Ambassador, 1, 2);
        let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Ambassador, 1, 1);
        let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Duke, 0, 2);
        let group4: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Assassin, 0, 1);
        let group5: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 1, 1], Card::Contessa, 0, 2);
        let group6: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Captain, 0, 1);
        
        colcon.add_raw_group(group1);
        colcon.add_raw_group(group2);
        colcon.add_raw_group(group3);
        colcon.add_raw_group(group4);
        colcon.add_raw_group(group5);
        colcon.add_raw_group(group6);
    
        // log::info!(" === Test 28 === ");
        // colcon.printlog();
    
        let output: bool = CollectiveConstraint::player_can_have_active_card_pub(&colcon, 4, &Card::Contessa);
        assert!(!output);
        // if output {
        //     println!("Test 28 Inferred Group needed Legal Wrong");
        // } else {
        //     println!("Test 28 Inferred Group needed Illegal Correct");
        // }
    }
}