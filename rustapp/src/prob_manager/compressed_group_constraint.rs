
use crate::history_public::Card;
use core::fmt;
use std::{fmt::Display, fmt::Debug};

// TODO: public constraint as a u32 3 bits per card x 6 players? Or probably better to just have a Vec that reserves space and reduces need for conversion
// TODO: joint constraint as u64 3bits per card x 6 players?
// TODO: but this is helpful for storeing the game state and sending it to a neural network...

/// From right to left, where 0 represents the last item
/// Bits 0..=6 represent boolean flags indicating which players are involved
/// Bits 7..=9 represent the Card number from 0..=4
/// Bits 10..=11 represent the dead count 0..=2
/// Bits 12..=13 represent the alive count 1..=3
/// Bits 14..=15 represent total count = dead + alive 1..=3
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CompressedGroupConstraint(u16);

impl Display for CompressedGroupConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Card: {:?}, Flags: [{} {} {} {} {} {} {}], {} dead {} alive {} total}}", 
            self.card(), 
            self.get_player_flag(0) as u8,
            self.get_player_flag(1) as u8,
            self.get_player_flag(2) as u8,
            self.get_player_flag(3) as u8,
            self.get_player_flag(4) as u8,
            self.get_player_flag(5) as u8,
            self.get_player_flag(6) as u8,
            self.count_dead(), 
            self.count_alive(), 
            self.count(),
        )
    }
}

impl Debug for CompressedGroupConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Card: {:?}, Flags: [{} {} {} {} {} {} {}], {} dead {} alive {} total}}", 
            self.card(), 
            self.get_player_flag(0) as u8,
            self.get_player_flag(1) as u8,
            self.get_player_flag(2) as u8,
            self.get_player_flag(3) as u8,
            self.get_player_flag(4) as u8,
            self.get_player_flag(5) as u8,
            self.get_player_flag(6) as u8,
            self.count_dead(), 
            self.count_alive(), 
            self.count(),
        )
    }
}

impl CompressedGroupConstraint {
    const START_FLAGS: [u16; 6] = [0b0000_0000_0100_0001, 0b0000_0000_0100_0010, 0b0000_0000_0100_0100, 0b0000_0000_0100_1000, 0b0000_0000_0101_0000, 0b0000_0000_0110_0000,];
    const PLAYER_BITS: u16 = 0b0000_0000_0111_1111; // Bits 0-6
    const CARD_SHIFT: u8 = 7;
    const CARD_MASK: u16 = 0b0000_0011_1000_0000; // Bits 7-9
    const DEAD_COUNT_SHIFT: u8 = 10;
    const DEAD_COUNT_MASK: u16 = 0b0000_1100_0000_0000; // Bits 10-11
    const ALIVE_COUNT_SHIFT: u8 = 12;
    const ALIVE_COUNT_MASK: u16 = 0b0011_0000_0000_0000; // Bits 12-13
    const TOTAL_COUNT_SHIFT: u8 = 14;
    const TOTAL_COUNT_MASK: u16 = 0b1100_0000_0000_0000; // Bits 14-15
    const MAX_PLAYERS: u8 = 7;
    pub fn new_with_pile(player_id: usize, card: Card, count_dead: u8, count_alive: u8) -> Self {
        debug_assert!(player_id < 6);
        let mut output = CompressedGroupConstraint(Self::START_FLAGS[player_id]);
        output.set_card(card);
        output.set_alive_count(count_alive);
        output.set_dead_count(count_dead);
        output.set_total_count((count_dead + count_alive));
        output
    }
    pub fn zero() -> Self {
        CompressedGroupConstraint(0)
    }
    pub fn set_player_flag(&mut self, player_id: usize, value: bool) {
        debug_assert!(player_id < 7);
        if value {
            self.0 |= 1 << player_id;
        } else {
            self.0 &= !(1 << player_id);
        }
    }
    pub fn get_player_flag(&self, player_id: usize) -> bool {
        debug_assert!(player_id < 7);
        (self.0 & (1 << player_id)) != 0
    }
    pub fn get_player_flags(&self) -> u16 {
        self.0 & Self::PLAYER_BITS
    }
    pub fn set_card(&mut self, card: Card) {
        debug_assert!(u8::from(card) < 8, "Card value out of bounds");
        // Clear existing card bits
        self.0 &= !(0b111 << Self::CARD_SHIFT);
        // Set new card bits
        self.0 |= (card as u16 & 0b111) << Self::CARD_SHIFT;
    }
    pub fn get_card(&self) -> Card {
        let num = ((self.0 & (0b111 << Self::CARD_SHIFT)) >> Self::CARD_SHIFT) as u8;
        Card::try_from(num).expect("Invalid value found")
    }
    /// Sets the dead count (0-3).
    pub fn set_dead_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Dead count out of bounds");
        // Clear existing dead count bits
        self.0 &= !(0b11 << Self::DEAD_COUNT_SHIFT);
        // Set new dead count bits
        self.0 |= (count as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
    }

    /// Retrieves the current dead count.
    pub fn get_dead_count(&self) -> u8 {
        ((self.0 & (0b11 << Self::DEAD_COUNT_SHIFT)) >> Self::DEAD_COUNT_SHIFT) as u8
    }

    /// Sets the alive count (0-3).
    pub fn set_alive_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Alive count out of bounds");
        // Clear existing alive count bits
        self.0 &= !(0b11 << Self::ALIVE_COUNT_SHIFT);
        // Set new alive count bits
        self.0 |= (count as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
    }
    
    /// Retrieves the current alive count.
    pub fn get_alive_count(&self) -> u8 {
        ((self.0 & (0b11 << Self::ALIVE_COUNT_SHIFT)) >> Self::ALIVE_COUNT_SHIFT) as u8
    }
    /// Sets the total count (0-3).
    pub fn set_total_count(&mut self, count: u8) {
        debug_assert!(count < 4, "Total count out of bounds");
        // Clear existing total count bits
        self.0 &= !(0b11 << Self::TOTAL_COUNT_SHIFT);
        // Set new total count bits
        self.0 |= (count as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
    }
    pub fn add_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        let new_count = dead_bits + (amount as u16);
        // TODO: Change this, this is meant to count total dead cards, up to 15, not just those of the card!
        debug_assert!(new_count < 4, "Dead count would exceed maximum");
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | (new_count << Self::DEAD_COUNT_SHIFT);
    }

    pub fn sub_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        debug_assert!(dead_bits >= amount as u16, "Dead count would go below zero dead_bits: {}, amount {}", dead_bits, amount);
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | ((dead_bits - amount as u16) << Self::DEAD_COUNT_SHIFT);
    }

    pub fn add_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        let new_count = alive_bits + (amount as u16);
        debug_assert!(new_count < 4, "Alive count would exceed maximum");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | (new_count << Self::ALIVE_COUNT_SHIFT);
    }

    pub fn sub_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        debug_assert!(alive_bits >= amount as u16, "Alive count would go below zero");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | ((alive_bits - amount as u16) << Self::ALIVE_COUNT_SHIFT);
    }

    pub fn add_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        let new_count = total_bits + (amount as u16);
        debug_assert!(new_count < 4, "Total count would exceed maximum");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | (new_count << Self::TOTAL_COUNT_SHIFT);
    }

    fn sub_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        debug_assert!(total_bits >= amount as u16, "Total count would go below zero");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | ((total_bits - amount as u16) << Self::TOTAL_COUNT_SHIFT);
    }
    /// Updates the total_count based on dead_count and alive_count.
    ///
    /// The total_count is set to the sum of dead_count and alive_count, clamped to a maximum of 3.
    pub fn update_total_count(&mut self) {
        let dead = self.get_dead_count();
        let alive = self.get_alive_count();
        let total = dead + alive;
        debug_assert!(total<4);
        // Clear existing total_count bits
        self.0 &= !Self::TOTAL_COUNT_MASK;

        // Set new total_count bits
        self.0 |= (total as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
    }
    /// Retrieves the total_count (1-3).
    ///
    /// # Example
    ///
    /// ```
    /// let total = constraint.get_total_count();
    /// ```
    pub fn get_total_count(&self) -> u8 {
        ((self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT) as u8
    }
    /// Clears all player flags.
    pub fn clear_players(&mut self) {
        self.0 &= !Self::PLAYER_BITS;
    }
    /// Retrieves all set player flags as a fixed-size array of boolean values.
    ///
    /// Each element in the returned array corresponds to a player.
    /// The array will have exactly 7 elements, where each index (0-6) represents a player.
    ///
    /// # Example
    ///
    /// ```
    /// let player_flags = constraint.get_set_players();
    /// ```
    pub fn get_set_players(&self) -> [bool; 7] {
        [
            self.get_player_flag(0),
            self.get_player_flag(1),
            self.get_player_flag(2),
            self.get_player_flag(3),
            self.get_player_flag(4),
            self.get_player_flag(5),
            self.get_player_flag(6),
        ]
    }
}
impl CompressedGroupConstraint {
    /// Constructor method that initialised based on a list of flags with each index representing player id
    pub fn new_list(participation_list: [bool; 7], card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u16 = 0;
        for (index, flag) in participation_list.iter().enumerate() {
            if *flag {
                value |= 1 << index;
            }
        }
        value |= (card as u16 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Constructor method that initialised based on a u16 0b00000000 for e.g. with each bit STARTING FROM THE RIGHT representing a player id flag
    /// From right to left, where 0 represents the last item
    /// Bits 0..=6 represent boolean flags indicating which players are involved
    /// Bits 7..=9 represent the Card number from 0..=4
    /// Bits 10..=11 represent the dead count 0..=2
    /// Bits 12..=13 represent the alive count 1..=3
    /// Bits 14..=15 represent total count = dead + alive 1..=3
    pub fn new_bit(participation_flags: u16, card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(participation_flags < 0b10000000, "Participation flag should be < 0b10000000 as there are only 7 players");
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u16 = participation_flags;
        value |= (card as u16 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u16 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u16 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u16 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Adds a player to the group represented by the group constraint
    /// Example: If the participation list originally included Players {1, 2, 5}, then adding 6 would make it {1, 2, 5, 6}.
    pub fn group_add(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        self.set_player_flag(player_id, true);
    }
    /// Replaces current participation list with a new one
    pub fn group_add_list(&mut self, list: [bool; 7]){
        self.0 &= !Self::PLAYER_BITS;
        for (index, flag) in list.iter().enumerate() {
            if *flag {
                self.0 |= 1 << index;
            }
        }
    }
    /// Removes a player from the group represented by the group constraint
    /// Example: If the participation list originally included Players {1, 2, 5}, then subtracting 5 would make it {1, 2}.
    pub fn group_subtract(&mut self, player_id: usize){
        debug_assert!(player_id < 7, "Invalid Player Id");
        let mask = !(0b1 << player_id);
        self.0 &= mask;
    }
    /// Adds number to dead count and increases total count too
    pub fn count_dead_add(&mut self, num: u8){
        self.add_dead_count(num);
        self.add_total_count(num);
    }
    /// Adds number to alive count and increases total count too
    pub fn count_alive_add(&mut self, num: u8){
        self.add_alive_count(num);
        self.add_total_count(num);
    }
    /// Subtracts number to dead count and decreases total count too
    pub fn count_dead_subtract(&mut self, num: u8){
        self.sub_dead_count(num);
        self.sub_total_count(num);
    }
    /// Subtracts number to alive count and decreases total count too
    pub fn count_alive_subtract(&mut self, num: u8){
        self.sub_alive_count(num);
        self.sub_total_count(num);
    }
    // TODO: Merge and refactor the wrappers
    // TODO: Review if its better to return a usize
    /// Returns count of alive + dead cards present in the group
    pub fn count(&self) -> u8 {
        self.get_total_count()
    }
    /// Returns count of dead cards present in the group
    pub fn count_dead(&self) -> u8 {
        self.get_dead_count()
    }
    /// Returns count of alive cards present in the group
    pub fn count_alive(&self) -> u8 {
        self.get_alive_count()
    }
    /// Returns the flag of whether a player is in the group being represented by the GroupConstraint
    pub fn indicator(&self, player_id : usize) -> bool {
        self.get_player_flag(player_id)
    }
    // TODO: Determine if this is the fastest way
    /// Returns true if participation list includes all players
    pub fn all_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b0000_0000_0111_1111
    }
    // TODO: Determine if this is the fastest way
    /// Returns true if participation list includes all players
    pub fn none_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b0000_0000_0000_0000
    }
    /// Returns a list of flags (the participation list) that indicates the set of players being specified to have a certain count of alive cards and dead cards
    pub fn get_list(&self) -> [bool; 7]{
        self.get_set_players()
    }
    pub fn get_flags(&self) -> u16 {
        self.0 & Self::PLAYER_BITS
    }
    /// Gets the Card thats stored
    pub fn card(&self) -> Card {
        self.get_card()
    } 
    /// Returns true if input group makes self redundant group constraint to store
    /// - The participation lists have to be equal
    /// - alive and dead counts of self have to be less than the input group
    pub fn is_subset_of(&self, group: &Self) -> bool {
        // Returns true if group makes self redundant
        // Its redundant if they have the same participation list and their counts are equal
        // Its also redundant if they have the same participation list and group has a higher count
        //      If a group has at least 2 Dukes, it also fulfils having at least 1 Duke.
        //      Therefore having at least 1 Duke is redundant

        if self.get_card() == group.get_card() &&
        // If participation lists are the same
        (self.get_flags() == group.get_flags()) &&
        // self.count() <= group.count() && 
        self.count_dead() <= group.count_dead() &&
        self.count_alive() <= group.count_alive() {
            return true
        }
        return false
    }
    /// Returns true if self's partipation list is subset of the input group's participation list
    /// Returns true if both participation lists are equal
    pub fn part_list_is_subset_of(&self, group: &Self) -> bool {
        // Checks if self participation list is a subset of group's participation list
        (group.0 & Self::PLAYER_BITS) == (self.0 & Self::PLAYER_BITS) | (group.0 & Self::PLAYER_BITS)
    }
    /// Returns true if self's partipation list is subset of the input group's participation list
    /// Returns true if both participation lists are equal
    pub fn part_list_is_subset_of_arr(&self, arr: &[bool; 7]) -> bool {
        // Checks if self participation list is a subset of group's participation list
        !((self.get_player_flag(0) & !arr[0]) |
        (self.get_player_flag(1) & !arr[1]) |
        (self.get_player_flag(2) & !arr[2]) |
        (self.get_player_flag(3) & !arr[3]) |
        (self.get_player_flag(4) & !arr[4]) |
        (self.get_player_flag(5) & !arr[5]) |
        (self.get_player_flag(6) & !arr[6]))
    }
    /// Returns true if the participation lists of self and group are mutually exclusive
    pub fn part_list_is_mut_excl(&self, group: Self) -> bool {
        // Checks if the groups are mutually exclusive
        ((self.0 & Self::PLAYER_BITS) & (group.0 & Self::PLAYER_BITS)) == 0
    }
    /// Returns true if participation list is subset of a group of only player and pile as true
    pub fn part_list_is_subset_of_player_and_pile(&self, player_id: usize) -> bool {
        // Ignores the 0 flags case, which should not exist
        debug_assert!((self.0 & Self::PLAYER_BITS) != 0, "0 flags case should not exist");
        Self::START_FLAGS[player_id] == (self.0 & Self::PLAYER_BITS) | Self::START_FLAGS[player_id]
    }
    // TODO: [TEST]
    // TODO: Refactor and make this redundant
    /// TODO: Consider refactoring this to use the bit representation
    /// TODO: Also considering why make a function this way... like u pass a mut ref and return it what even...
    pub fn list_union<'a>(list1: &'a mut [bool; 7], list2: &[bool; 7]) -> &'a [bool; 7] {
        list1.iter_mut().zip(list2).for_each(|(a, b)| *a |= b);
        list1
    }
        /// Test to determine if input group_in_question is considered redundant because of the information provided in group_in_reference
    /// Note: this is a test that decides the redundancy of group_in_question and not group_to_reference
    /// YOU WILL NOT NECESSARILY GET THE SAME OUTPUT IF YOU SWAP THEM
    /// This is different from the original
    pub fn is_redundant(&self, group_to_reference: &CompressedGroupConstraint) -> bool {
        if self == group_to_reference {
            return true
        }
        if self.get_card() == group_to_reference.get_card() {
            if self.get_flags() == group_to_reference.get_flags() && 
            // Same Participation list / player flags
            self.count_alive() < group_to_reference.count_alive() {
                // dead_count is skipped because self cannot possibly have more
                // Can use < here instead of <= as == is covered earlier
                // [SUBSET] group in question is redundant as its informational content is fully described by group_to_reference
                return true
            }
            // Don't recall why old version checks remaining counts
            if group_to_reference.part_list_is_subset_of(self) &&
            self.count_alive() == group_to_reference.count_alive() {
                // [SUBSET] group in questions is redundant as its informational content is fully described by group_to_reference
                return true
            }
        }
        false
    }
    /// Get union of 2 mutually exclusive CompressedGroupConstraint
    pub fn mutually_exclusive_union(group_i: CompressedGroupConstraint, group_j: CompressedGroupConstraint) -> CompressedGroupConstraint {
        debug_assert!(group_i.part_list_is_mut_excl(group_j), "Part List of groups must be mutually exclusive! Current groups are i:{:016b}, j: {:016b}", group_i.0, group_j.0);
        debug_assert!(group_i.card() == group_j.card(), "Cannot make a union of groups with different cards! i: {:?}, j: {:?}", group_i.card(), group_j.card());
        log::trace!("In mutually_exclusive_union");
        let mut new_group: CompressedGroupConstraint = CompressedGroupConstraint(group_i.0 | group_j.0);
        // TODO: Implement a Union
        new_group.set_card(group_i.card());
        let total_dead = group_i.count_dead() + group_j.count_dead();
        let total_alive = group_i.count_alive() + group_j.count_alive();
        new_group.set_dead_count(total_dead);
        new_group.set_alive_count(total_alive);
        new_group.set_total_count(total_alive + total_dead);
        new_group
    }
}