
use crate::history_public::Card;
use core::fmt;
use std::{fmt::Display, fmt::Debug};

// TODO: public constraint as a u32 3 bits per card x 6 players? Or probably better to just have a Vec that reserves space and reduces need for conversion
// TODO: joint constraint as u64 3bits per card x 6 players?
// TODO: but this is helpful for storeing the game state and sending it to a neural network...

/// [OLD] u16
/// From right to left, where 0 represents the last item
/// Bits 0..=6 represent boolean flags indicating which players are involved
/// Bits 7..=9 represent the Card number from 0..=4
/// Bits 10..=11 represent the dead count 0..=2
/// Bits 12..=13 represent the alive count 1..=3
/// Bits 14..=15 represent total count = dead + alive 1..=3
/// [NEW] u32
/// From right to left, where 0 represents the last item
/// Bits 0..=6 represent boolean flags indicating which players are involved
/// Bits 7..=13 represent boolean flags indicating whether only 1 of a player's card is involved b0 => 2 cards involved, b1 => 1 card involved
///     - e.g. if a player reveal_redraw, only 1 card of the player is part of the mix, but all are for the pile as no one knows what card comes from the pile
/// Bits 14..=16 represent the Card number from 0..=4
/// Bits 17..=18 represent the dead count 0..=2
/// Bits 19..=20 represent the alive count 1..=3
/// Bits 21..=22 represent total count = dead + alive 1..=3
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CompressedGroupConstraint(u32);

impl Display for CompressedGroupConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Card: {:?}, Flags: [{} {} {} {} {} {} {}], Single Card Flags: [{} {} {} {} {} {} {}], {} dead {} alive {} total}}", 
            self.card(), 
            self.get_player_flag(0) as u8,
            self.get_player_flag(1) as u8,
            self.get_player_flag(2) as u8,
            self.get_player_flag(3) as u8,
            self.get_player_flag(4) as u8,
            self.get_player_flag(5) as u8,
            self.get_player_flag(6) as u8,
            self.get_single_card_flag(0) as u8,
            self.get_single_card_flag(1) as u8,
            self.get_single_card_flag(2) as u8,
            self.get_single_card_flag(3) as u8,
            self.get_single_card_flag(4) as u8,
            self.get_single_card_flag(5) as u8,
            self.get_single_card_flag(6) as u8,
            self.count_dead(), 
            self.count_alive(), 
            self.count(),
        )
    }
}

impl Debug for CompressedGroupConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Card: {:?}, Flags: [{} {} {} {} {} {} {}], Single Card Flags: [{} {} {} {} {} {} {}], {} dead {} alive {} total}}", 
            self.card(), 
            self.get_player_flag(0) as u8,
            self.get_player_flag(1) as u8,
            self.get_player_flag(2) as u8,
            self.get_player_flag(3) as u8,
            self.get_player_flag(4) as u8,
            self.get_player_flag(5) as u8,
            self.get_player_flag(6) as u8,
            self.get_single_card_flag(0) as u8,
            self.get_single_card_flag(1) as u8,
            self.get_single_card_flag(2) as u8,
            self.get_single_card_flag(3) as u8,
            self.get_single_card_flag(4) as u8,
            self.get_single_card_flag(5) as u8,
            self.get_single_card_flag(6) as u8,
            self.count_dead(), 
            self.count_alive(), 
            self.count(),
        )
    }
}

impl CompressedGroupConstraint {
    const START_FLAGS: [u32; 6] = [
        0b00000000_00000000_00000000_01000001, 
        0b00000000_00000000_00000000_01000010, 
        0b00000000_00000000_00000000_01000100, 
        0b00000000_00000000_00000000_01001000, 
        0b00000000_00000000_00000000_01010000, 
        0b00000000_00000000_00000000_01100000,];
    const PLAYER_BITS: u32 = 0b00000000_00000000_00000000_01111111; // Bits 0-6
    const SINGLE_CARD_FLAGS_SHIFT: usize = 7;
    const SINGLE_CARD_MASK: u32 = 0b00000000_00000000_00111111_10000000; // Bits 7-13
    // const CARD_SHIFT: u8 = 7;
    const CARD_SHIFT: u8 = 14;
    const CARD_MASK: u32 = 0b00000000_00000001_11000000_00000000; // Bits 14-16
    // const DEAD_COUNT_SHIFT: u8 = 10;
    const DEAD_COUNT_SHIFT: u8 = 17;
    const DEAD_COUNT_MASK: u32 = 0b00000000_00000110_00000000_00000000; // Bits 17-18
    // const ALIVE_COUNT_SHIFT: u8 = 12;
    const ALIVE_COUNT_SHIFT: u8 = 19;
    const ALIVE_COUNT_MASK: u32 = 0b00000000_00011000_00000000_00000000; // Bits 19-20
    // const TOTAL_COUNT_SHIFT: u8 = 14;
    const TOTAL_COUNT_SHIFT: u8 = 21;
    const TOTAL_COUNT_MASK: u32 = 0b00000000_01100000_00000000_00000000; // Bits 21-22
    const MAX_PLAYERS: u8 = 7;
    /// Constructor method that initialised based on a list of flags with each index representing player id
    /// Lets you set single_card_lists to 1 for that particular player
    pub fn new_with_pile(player_id: usize, card: Card, count_dead: u8, count_alive: u8) -> Self {
        debug_assert!(player_id < 6);
        let mut output = CompressedGroupConstraint(Self::START_FLAGS[player_id]);
        output.set_single_card_flag(player_id, true);
        output.set_card(card);
        output.set_alive_count(count_alive);
        output.set_dead_count(count_dead);
        output.set_total_count(count_dead + count_alive);
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
    pub fn get_player_flags(&self) -> u32 {
        self.0 & Self::PLAYER_BITS
    }
    pub fn set_card(&mut self, card: Card) {
        debug_assert!(u8::from(card) < 8, "Card value out of bounds");
        // Clear existing card bits
        self.0 &= !(0b111 << Self::CARD_SHIFT);
        // Set new card bits
        self.0 |= (card as u32 & 0b111) << Self::CARD_SHIFT;
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
        self.0 |= (count as u32 & 0b11) << Self::DEAD_COUNT_SHIFT;
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
        self.0 |= (count as u32 & 0b11) << Self::ALIVE_COUNT_SHIFT;
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
        self.0 |= (count as u32 & 0b11) << Self::TOTAL_COUNT_SHIFT;
    }
    pub fn add_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        let new_count = dead_bits + (amount as u32);
        debug_assert!(new_count < 4, "Dead count would exceed maximum");
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | (new_count << Self::DEAD_COUNT_SHIFT);
    }

    pub fn sub_dead_count(&mut self, amount: u8) {
        let dead_bits = (self.0 & Self::DEAD_COUNT_MASK) >> Self::DEAD_COUNT_SHIFT;
        debug_assert!(dead_bits >= amount as u32, "Dead count would go below zero dead_bits: {}, amount {}", dead_bits, amount);
        self.0 = (self.0 & !Self::DEAD_COUNT_MASK) | ((dead_bits - amount as u32) << Self::DEAD_COUNT_SHIFT);
    }

    pub fn add_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        let new_count = alive_bits + (amount as u32);
        debug_assert!(new_count < 4, "Alive count would exceed maximum");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | (new_count << Self::ALIVE_COUNT_SHIFT);
    }

    pub fn sub_alive_count(&mut self, amount: u8) {
        let alive_bits = (self.0 & Self::ALIVE_COUNT_MASK) >> Self::ALIVE_COUNT_SHIFT;
        debug_assert!(alive_bits >= amount as u32, "Alive count would go below zero");
        self.0 = (self.0 & !Self::ALIVE_COUNT_MASK) | ((alive_bits - amount as u32) << Self::ALIVE_COUNT_SHIFT);
    }

    pub fn add_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        let new_count = total_bits + (amount as u32);
        debug_assert!(new_count < 4, "Total count would exceed maximum");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | (new_count << Self::TOTAL_COUNT_SHIFT);
    }

    fn sub_total_count(&mut self, amount: u8) {
        let total_bits = (self.0 & Self::TOTAL_COUNT_MASK) >> Self::TOTAL_COUNT_SHIFT;
        debug_assert!(total_bits >= amount as u32, "Total count would go below zero");
        self.0 = (self.0 & !Self::TOTAL_COUNT_MASK) | ((total_bits - amount as u32) << Self::TOTAL_COUNT_SHIFT);
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
        self.0 |= (total as u32 & 0b11) << Self::TOTAL_COUNT_SHIFT;
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
        self.0 &= !(Self::PLAYER_BITS | Self::SINGLE_CARD_MASK);
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
    /// Sets the flag for single cards
    /// false => includes both of the players' cards
    /// true => includes 1 of the players' cards
    pub fn set_single_card_flag(&mut self, player_id: usize, value: bool) {
        debug_assert!(player_id < 7, "Player ID out of bounds for single card flags");
        let bit = Self::SINGLE_CARD_FLAGS_SHIFT + player_id;
        if value {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }
    /// Gets the flag for a particular player
    /// false => includes both of the players' cards
    /// true => includes 1 of the players' cards
    pub fn get_single_card_flag(&self, player_id: usize) -> bool {
        debug_assert!(player_id < 7, "Player ID out of bounds for single card flags");
        let bit = Self::SINGLE_CARD_FLAGS_SHIFT + player_id;
        (self.0 & (1 << bit)) != 0
    }
    /// Retrieves all set single card player as a fixed-size array of boolean values.
    ///
    /// Each element in the returned array corresponds to a player.
    /// The array will have exactly 7 elements, where each index (0-6) represents a player.
    pub fn get_single_card_flags(&self, player_id: usize) -> [bool; 7] {
        [
            self.get_single_card_flag(0),
            self.get_single_card_flag(1),
            self.get_single_card_flag(2),
            self.get_single_card_flag(3),
            self.get_single_card_flag(4),
            self.get_single_card_flag(5),
            self.get_single_card_flag(6),
        ]
    }
    /// Returns true if all are single card flags are the same
    pub fn single_card_flags_equal(&self, other_group: Self) -> bool {
        self.0 & Self::SINGLE_CARD_MASK == other_group.0 & Self::SINGLE_CARD_MASK
    }
    /// Returns true if self group is a subset of other_group
    /// NOTE:
    /// - Subset here does not refer to a SET subset
    /// - It refers to an informational subset
    /// - [0 0 0 0 0 0 0] is subset of [0 1 0 0 0 0 0] because having a single card flag is more restrictive, and tells us more
    /// - [0 1 0 0 0 0 0] is a subset of [0 1 0 0 0 0 0] as this is not a "strict" subset
    /// - [0 1 0 0 0 0 0] is a subset of [0 1 0 1 0 0 0]
    /// - [0 1 0 1 0 0 0] is not a subset of [0 1 0 1 0 0 0]
    /// - [0 1 0 1 0 0 0] is not a subset of [0 0 0 1 0 1 0] and vice versa
    pub fn single_card_flags_is_subset_of(&self, other_group: Self) -> bool {
        (self.0 & Self::SINGLE_CARD_MASK) == (self.0 & other_group.0 & Self::SINGLE_CARD_MASK)
    }
}
impl CompressedGroupConstraint {
    /// Constructor method that initialised based on a list of flags with each index representing player id
    /// By default all single_card_lists are 0 i.e. all cards for a player are double
    pub fn new_list(participation_list: [bool; 7], card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u32 = 0;
        for (index, flag) in participation_list.iter().enumerate() {
            if *flag {
                value |= 1 << index;
            }
        }
        value |= (card as u32 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u32 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u32 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u32 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Constructor method that initialised based on a list of flags with each index representing player id
    /// Lets you set single_card_lists to 1 for that particular player
    pub fn new_list_redraw(participation_list: [bool; 7], player_id: usize, card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u32 = 0;
        for (index, flag) in participation_list.iter().enumerate() {
            if *flag {
                value |= 1 << index;
            }
        }
        value |= 0b1 << (Self::SINGLE_CARD_FLAGS_SHIFT + player_id);
        value |= (card as u32 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u32 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u32 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u32 & 0b11) << Self::TOTAL_COUNT_SHIFT;
        CompressedGroupConstraint(value)
    }
    /// Constructor method that initialised based on a u16 0b00000000 for e.g. with each bit STARTING FROM THE RIGHT representing a player id flag
    /// From right to left, where 0 represents the last item
    /// Bits 0..=6 represent boolean flags indicating which players are involved
    /// Bits 7..=13 represent boolean flags indicating whether only 1 of a player's card is involved b0 => 2 cards involved, b1 => 1 card involved
    /// Bits 14..=16 represent the Card number from 0..=4
    /// Bits 17..=18 represent the dead count 0..=2
    /// Bits 19..=20 represent the alive count 1..=3
    /// Bits 21..=22 represent total count = dead + alive 1..=3
    pub fn new_bit(participation_flags: u32, card: Card, count_dead: usize, count_alive: usize) -> Self {
        debug_assert!(participation_flags < 0b10000000, "Participation flag should be < 0b10000000 as there are only 7 players");
        debug_assert!(count_dead < 4, "Dead count must be less than 4");
        debug_assert!(count_alive < 4, "Alive count must be less than 4");
        debug_assert!(count_dead + count_alive < 4, "Total count must be less than 4");
        let mut value: u32 = participation_flags;
        value |= (card as u32 & 0b111) << Self::CARD_SHIFT;
        value |= (count_dead as u32 & 0b11) << Self::DEAD_COUNT_SHIFT;
        value |= (count_alive as u32 & 0b11) << Self::ALIVE_COUNT_SHIFT;
        value |= ((count_dead + count_alive) as u32 & 0b11) << Self::TOTAL_COUNT_SHIFT;
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
    /// Returns true if participation list includes all players
    pub fn all_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b00000000_00000000_00000000_01111111
    }
    /// Returns true if participation list includes all players
    pub fn none_in(&self) -> bool {
        (self.0 & Self::PLAYER_BITS) == 0b00000000_00000000_00000000_00000000
    }
    /// Returns a list of flags (the participation list) that indicates the set of players being specified to have a certain count of alive cards and dead cards
    pub fn get_list(&self) -> [bool; 7]{
        self.get_set_players()
    }
    pub fn get_flags(&self) -> u32 {
        self.0 & Self::PLAYER_BITS
    }
    /// Gets the Card thats stored
    pub fn card(&self) -> Card {
        self.get_card()
    } 
    /// Returns true if self's partipation list is subset of the input group's participation list
    /// Returns true if both participation lists are equal
    pub fn part_list_is_subset_of(&self, group: &Self) -> bool {
        // Checks if self participation list is a subset of group's participation list
        // TODO: Cant u just use
        // self.0 & Self::PLAYER_BITS == self.0 & group.0 & Self::PLAYER_BITS
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
    /// Get union of 2 mutually exclusive CompressedGroupConstraint
    pub fn mutually_exclusive_union(group_i: CompressedGroupConstraint, group_j: CompressedGroupConstraint) -> CompressedGroupConstraint {
        debug_assert!(group_i.part_list_is_mut_excl(group_j), "Part List of groups must be mutually exclusive! Current groups are i:{:016b}, j: {:016b}", group_i.0, group_j.0);
        debug_assert!(group_i.card() == group_j.card(), "Cannot make a union of groups with different cards! i: {:?}, j: {:?}", group_i.card(), group_j.card());
        log::trace!("In mutually_exclusive_union");
        let mut new_group: CompressedGroupConstraint = CompressedGroupConstraint(group_i.0 | group_j.0);
        new_group.set_card(group_i.card());
        let total_dead = group_i.count_dead() + group_j.count_dead();
        let total_alive = group_i.count_alive() + group_j.count_alive();
        new_group.set_dead_count(total_dead);
        new_group.set_alive_count(total_alive);
        new_group.set_total_count(total_alive + total_dead);
        new_group
    }
}

#[cfg(test)]
mod group_test {
    use super::*;
    use crate::history_public::Card;

    #[test]
    fn test_new_with_pile() {
        // This constructor sets player_id, card, count_dead, count_alive
        let mut constraint = CompressedGroupConstraint::new_with_pile(2, Card::Captain, 1, 2);
        assert_eq!(constraint.get_dead_count(), 1);
        assert_eq!(constraint.get_alive_count(), 2);
        assert_eq!(constraint.count(), 3); // total = 1 + 2
        assert_eq!(constraint.card(), Card::Captain);
        // Player 2 & the "pile" flags must be set
        // So we expect bits for player 2 (1 << 2) and also the bit for the pile in START_FLAGS (which is the top 1<<6).
        // Check that get_player_flag(2) is true.
        assert!(constraint.get_player_flag(2));
        assert!(constraint.get_single_card_flag(2));

        // Let's modify counts
        constraint.set_dead_count(2);
        assert_eq!(constraint.get_dead_count(), 2);
        constraint.set_alive_count(1);
        assert_eq!(constraint.get_alive_count(), 1);
        // Now total_count should be updated to 3, let's confirm
        constraint.update_total_count();
        assert_eq!(constraint.get_total_count(), 3);

        // Make sure card can be changed
        constraint.set_card(Card::Duke);
        assert_eq!(constraint.card(), Card::Duke);
    }

    #[test]
    fn test_zero() {
        // zero() means no flags are set, no players, zero counts.
        let constraint = CompressedGroupConstraint::zero();
        // All flags = false
        for i in 0..7 {
            assert!(!constraint.get_player_flag(i));
        }
        for i in 0..7 {
            assert!(!constraint.get_single_card_flag(i));
        }
        // All counts = 0
        assert_eq!(constraint.count_dead(), 0);
        assert_eq!(constraint.count_alive(), 0);
        assert_eq!(constraint.count(), 0);
        // By default, card = 0 => Card(0)
        // Depending on your Card definition, this might be invalid or the first variant, so check carefully
        assert_eq!(constraint.card(), Card::try_from(0).unwrap());
    }

    #[test]
    fn test_set_player_flag() {
        let mut constraint = CompressedGroupConstraint::zero();
        // Set a few player flags
        constraint.set_player_flag(0, true);
        constraint.set_player_flag(3, true);
        constraint.set_player_flag(6, true);

        assert!(constraint.get_player_flag(0));
        assert!(constraint.get_player_flag(3));
        assert!(constraint.get_player_flag(6));

        // Turn off player 3
        constraint.set_player_flag(3, false);
        assert!(!constraint.get_player_flag(3));
        constraint.set_player_flag(5, false);
        assert!(!constraint.get_player_flag(5));
    }

    #[test]
    fn test_clear_players() {
        let mut constraint = CompressedGroupConstraint::zero();
        for i in 0..7 {
            constraint.set_player_flag(i, true);
        }
        for i in 0..7 {
            constraint.set_single_card_flag(i, true);
        }
        constraint.clear_players();
        for i in 0..7 {
            assert!(!constraint.get_player_flag(i));
        }
        for i in 0..7 {
            assert!(!constraint.get_single_card_flag(i));
        }
    }

    #[test]
    fn test_single_card_flags() {
        let mut constraint = CompressedGroupConstraint::zero();
        // By default, single_card_flags are all false
        for i in 0..7 {
            assert!(!constraint.get_single_card_flag(i));
        }
        // set_single_card_flag
        constraint.set_single_card_flag(2, true);
        assert!(constraint.get_single_card_flag(2));
        constraint.set_single_card_flag(2, false);
        assert!(!constraint.get_single_card_flag(2));
        constraint.set_single_card_flag(5, true);
        assert!(constraint.get_single_card_flag(2));
        constraint.set_single_card_flag(5, false);
        assert!(!constraint.get_single_card_flag(2));
    }

    #[test]
    fn test_counts() {
        let mut constraint = CompressedGroupConstraint::zero();
        constraint.set_dead_count(1);
        constraint.set_alive_count(2);
        constraint.update_total_count();
        assert_eq!(constraint.get_dead_count(), 1);
        assert_eq!(constraint.get_alive_count(), 2);
        assert_eq!(constraint.count(), 3);
        
        let mut constraint = CompressedGroupConstraint::zero();
        // Adding
        constraint.count_dead_add(1); // dead = 2
        constraint.count_alive_add(1); // alive = 3
        assert_eq!(constraint.count_dead(), 1);
        assert_eq!(constraint.count_alive(), 1);
        assert_eq!(constraint.count(), 2, "But watch out if your total bits only store up to 3!");

        // The code debug_assert! might fail if the total > 3 is not allowed.
        // If your actual usage is capping at 3, the debug_assert would panic in test or debug mode.
        // For demonstration, let's reduce back to valid <=3 scenario:
        let mut constraint2 = CompressedGroupConstraint::zero();
        constraint2.set_dead_count(2);
        constraint2.set_alive_count(1);
        constraint2.set_total_count(3);
        assert_eq!(constraint2.count_dead(), 2);
        assert_eq!(constraint2.count_alive(), 1);
        assert_eq!(constraint2.count(), 3);

        constraint2.count_dead_subtract(1);
        constraint2.count_alive_subtract(1);
        assert_eq!(constraint2.count_dead(), 1);
        assert_eq!(constraint2.count_alive(), 0);
        constraint2.update_total_count();
        assert_eq!(constraint2.count(), 1);
    }


    #[test]
    fn test_new_bit() {
        // Example: if we pass participation_flags = 0b00000101 => players 0 & 2 are set
        let constraint = CompressedGroupConstraint::new_bit(0b00000101, Card::Duke, 1, 2);
        assert!(constraint.get_player_flag(0));
        assert!(!constraint.get_player_flag(1));
        assert!(constraint.get_player_flag(2));
        assert_eq!(constraint.card(), Card::Duke);
        assert_eq!(constraint.count_dead(), 1);
        assert_eq!(constraint.count_alive(), 2);
        assert_eq!(constraint.count(), 3);
    }

    #[test]
    fn test_group_add_and_subtract() {
        let mut constraint = CompressedGroupConstraint::zero();
        constraint.group_add(1);
        assert!(constraint.get_player_flag(1));

        constraint.group_subtract(1);
        assert!(!constraint.get_player_flag(1));

        constraint.group_add(3);
        constraint.group_add(5);
        assert!(constraint.get_player_flag(3));
        assert!(constraint.get_player_flag(5));

        constraint.group_add_list([true, false, true, false, true, false, true]);
        // Now flags for player 0,2,4,6 must be set, everything else must match
        assert!(constraint.get_player_flag(0));
        assert!(!constraint.get_player_flag(1));
        assert!(constraint.get_player_flag(2));
        assert!(!constraint.get_player_flag(3));
        assert!(constraint.get_player_flag(4));
        assert!(!constraint.get_player_flag(5));
        assert!(constraint.get_player_flag(6));
    }

    #[test]
    fn test_part_list_is_subset_of() {
        let group1 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        let group2 = CompressedGroupConstraint::new_list([true, true, true, false, false, false, false], Card::Duke, 1, 0);
        assert!(group1.part_list_is_subset_of(&group2));
        assert!(!group2.part_list_is_subset_of(&group1));

        let group1 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        let group2 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        assert!(group1.part_list_is_subset_of(&group2));
        assert!(group2.part_list_is_subset_of(&group1));

        let group1 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        let group2 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Captain, 2, 1);
        assert!(group1.part_list_is_subset_of(&group2));
        assert!(group2.part_list_is_subset_of(&group1));

        let group1 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        let group2 = CompressedGroupConstraint::new_list([true, true, true, false, false, false, false], Card::Captain, 2, 1);
        assert!(group1.part_list_is_subset_of(&group2));
        assert!(!group2.part_list_is_subset_of(&group1));
    }

    #[test]
    fn test_part_list_is_subset_of_arr() {
        let group1 = CompressedGroupConstraint::new_list([true, false, true, false, false, false, false], Card::Duke, 1, 0);
        let bigger_arr = [true, true, true, false, false, false, false];
        assert!(group1.part_list_is_subset_of_arr(&bigger_arr));

        let smaller_arr = [false, false, false, false, false, false, false];
        assert!(!group1.part_list_is_subset_of_arr(&smaller_arr));
    }

    #[test]
    fn test_part_list_is_mut_excl() {
        let group1 = CompressedGroupConstraint::new_list([true, false, false, false, false, false, false], Card::Duke, 0, 1);
        let group2 = CompressedGroupConstraint::new_list([false, true, true, false, false, false, false], Card::Duke, 0, 2);
        // group1 => only player 0, group2 => players 1,2 => no overlap => mutually exclusive
        assert!(group1.part_list_is_mut_excl(group2));

        let group3 = CompressedGroupConstraint::new_list([true, true, false, false, false, false, false], Card::Duke, 0, 1);
        assert!(!group1.part_list_is_mut_excl(group3));
    }

    #[test]
    fn test_mutually_exclusive_union_new_with_pile() {
        // Use new_with_pile() for both constraints
        // i => player_id=0, Duke, 1 dead, 0 alive
        //     implies single_card_flag(0)=true, plus the pile bit set from START_FLAGS[0]
        let i = CompressedGroupConstraint::new_with_pile(0, Card::Duke, 1, 1);
    
        let mut j = CompressedGroupConstraint::zero();
        j.set_player_flag(1, true);
        j.set_alive_count(1);
        j.set_total_count(1);
        j.set_card(Card::Duke);
    
        assert!(i.part_list_is_mut_excl(j), "Expected i and j to be disjoint in player bits");
        assert!(i.get_single_card_flag(0), "Expected i to have single_card_flag(0) = true");
        assert!(!i.get_single_card_flag(6), "Expected i to have single_card_flag(6) = false");
    
        let union_group = CompressedGroupConstraint::mutually_exclusive_union(i, j);
    
        // 4. Verify the union's participation flags
        //    We expect players 0 & 1 to be set, plus the pile bit from each (bit 6).
        assert!(union_group.get_player_flag(0), "Player 0 should be set in the union");
        assert!(union_group.get_player_flag(1), "Player 1 should be set in the union");
        assert!(union_group.get_player_flag(6), "Pile bit (player 6) should be set in the union");

        assert!(union_group.get_single_card_flag(0), "Expected i to have single_card_flag(0) = true");
        assert!(!union_group.get_single_card_flag(6), "Expected i to have single_card_flag(6) = false");
        assert_eq!(union_group.card(), Card::Duke, "Union should keep the same card (Duke)");
        assert_eq!(union_group.count_dead(), 1, "Expected 3 dead in the union");
        assert_eq!(union_group.count_alive(), 2, "Expected 0 alive in the union");
        assert_eq!(union_group.count(), 3, "Total count should be 3");
    
        // 7. Single card flags after union
        // Because they do not overlap in players, we expect single_card_flag(0)=true, single_card_flag(1)=true
        // This is effectively a bitwise OR as well.
        assert!(union_group.get_single_card_flag(0), "single_card_flag(0) should remain true");
    
        // No other single_card_flags should be set
        for other_pid in 2..6 {
            assert!(
                !union_group.get_single_card_flag(other_pid),
                "Expected single_card_flag({}) to remain false",
                other_pid
            );
        }
    }
    

    #[test]
    fn test_part_list_is_subset_of_player_and_pile() {
        // Suppose player_id=3 => we check against START_FLAGS[3]
        // START_FLAGS[3] is 0b0000_0000_0000_0000_0100_1000 (the reference for 'pile' + player 3 bit).
        let group = CompressedGroupConstraint::START_FLAGS[3];
        // The group we test with must only contain the same bits as START_FLAGS[3].
        let c = CompressedGroupConstraint(group);
        assert!(c.part_list_is_subset_of_player_and_pile(3));

        // If we add an extra player => it should fail
        let mut c2 = CompressedGroupConstraint(group);
        c2.group_add(0);
        assert!(!c2.part_list_is_subset_of_player_and_pile(3));
    }
}
