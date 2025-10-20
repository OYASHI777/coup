use arrayvec::ArrayVec;

use crate::{
    history_public::Card,
    prob_manager::engine::constants::{INDEX_PILE, MAX_CARD_PERMS_ONE},
};

pub const MAX_REQUIRED_PLAYER_BUFFER_SIZE: usize = 5; // swap_ordered can make it go up to 5
/// This aids in resetting inferred constraint after card swaps between players
pub struct MoveGuard;

impl MoveGuard {
    /// Here it is intended that the cards are
    ///     - removed from BOTH
    ///     - then added to BOTH
    /// NOT
    ///     - removed from b
    ///     - added to a
    ///     - then removed from a
    ///     - then added to b
    #[inline(always)]
    pub fn swap(
        inferred_constraint: &mut [Vec<Card>],
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        let mut moved_from_a_to_b: ArrayVec<Card, 2> = ArrayVec::new();
        let mut moved_from_b_to_a: ArrayVec<Card, 2> = ArrayVec::new();
        for c in a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_a]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_a].swap_remove(pos);
                // SAFETY: This is fine as we know it never exceeds the cap
                unsafe {
                    moved_from_a_to_b.push_unchecked(card);
                }
            }
        }
        for c in b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_b]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_b].swap_remove(pos);
                // SAFETY: This is fine as we know it never exceeds the cap
                unsafe {
                    moved_from_b_to_a.push_unchecked(card);
                }
            }
        }
        inferred_constraint[player_b].extend(moved_from_a_to_b.iter());
        inferred_constraint[player_a].extend(moved_from_b_to_a.iter());
        if f(inferred_constraint) {
            return true;
        }
        for c in moved_from_b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_a]
                .iter()
                .rposition(|card| card == c)
            {
                inferred_constraint[player_a].swap_remove(pos);
            }
        }
        for c in moved_from_a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_b]
                .iter()
                .rposition(|card| card == c)
            {
                inferred_constraint[player_b].swap_remove(pos);
            }
        }
        inferred_constraint[player_b].extend(moved_from_b_to_a.iter());
        inferred_constraint[player_a].extend(moved_from_a_to_b.iter());
        false
    }
    /// Here it is intended that the cards are
    ///     - removed from b
    ///     - added to a
    ///     - then removed from a
    ///     - then added to b
    /// NOT
    ///     - removed from BOTH
    ///     - then added to BOTH
    #[inline(always)]
    pub fn swap_ordered(
        inferred_constraint: &mut [Vec<Card>],
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        let mut removed_from_b: ArrayVec<Card, 2> = ArrayVec::new();
        let mut removed_from_a: ArrayVec<Card, 2> = ArrayVec::new();
        for &c in a_to_b {
            if let Some(pos) = inferred_constraint[player_b].iter().rposition(|x| *x == c) {
                inferred_constraint[player_b].swap_remove(pos);
                // SAFETY: This is fine as we know it never exceeds the cap
                unsafe {
                    removed_from_b.push_unchecked(c);
                }
            }
            inferred_constraint[player_a].push(c);
        }
        for &c in b_to_a {
            if let Some(pos) = inferred_constraint[player_a].iter().rposition(|x| *x == c) {
                inferred_constraint[player_a].swap_remove(pos);
                // SAFETY: This is fine as we know it never exceeds the cap
                unsafe {
                    removed_from_a.push_unchecked(c);
                }
            }
            inferred_constraint[player_b].push(c);
        }
        if f(inferred_constraint) {
            return true;
        }
        for &c in b_to_a.iter().rev() {
            if let Some(pos) = inferred_constraint[player_b].iter().rposition(|x| *x == c) {
                inferred_constraint[player_b].swap_remove(pos);
            }
        }
        for &c in removed_from_a.iter().rev() {
            inferred_constraint[player_a].push(c);
        }
        for &c in a_to_b.iter().rev() {
            if let Some(pos) = inferred_constraint[player_a].iter().rposition(|x| *x == c) {
                inferred_constraint[player_a].swap_remove(pos);
            }
        }
        for &c in removed_from_b.iter().rev() {
            inferred_constraint[player_b].push(c);
        }
        false
    }
    #[inline(always)]
    pub fn discard(
        inferred_constraint: &mut [Vec<Card>],
        player: usize,
        card: Card,
        f: impl FnOnce(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        // Documentation comment => This is only required if resurrection is possible
        // let mut removed: ArrayVec<Card, 1> = ArrayVec::new();
        // if let Some(pos) = public_constraint[player].iter().rposition(|c| *c == card) {
        //     public_constraint[player].swap_remove(pos);
        //     // SAFETY: This is fine as we know it never exceeds the cap
        //     unsafe {
        //         removed.push_unchecked(card);
        //     }
        // }
        inferred_constraint[player].push(card);
        if f(inferred_constraint) {
            return true;
        }
        if let Some(pos) = inferred_constraint[player].iter().rposition(|c| *c == card) {
            inferred_constraint[player].swap_remove(pos);
        }
        // Documentation comment => This is only required if resurrection is possible
        // for &c in removed.iter() {
        //     public_constraint[player].push(c);
        // }
        false
    }
    /// Ensures a player's hand contains at least the cards in needed
    #[inline(always)]
    pub fn with_needed_cards_present(
        inferred_constraint: &mut [Vec<Card>],
        player: usize,
        needed: &[Card],
        f: impl FnOnce(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        let mut have: [u8; MAX_CARD_PERMS_ONE] = [0; MAX_CARD_PERMS_ONE];
        for &x in inferred_constraint[player].iter() {
            have[x as usize] += 1;
        }

        let mut need: [u8; MAX_CARD_PERMS_ONE] = [0; MAX_CARD_PERMS_ONE];
        for &c in needed {
            need[c as usize] += 1;
        }

        let mut added: [u8; MAX_CARD_PERMS_ONE] = [0; MAX_CARD_PERMS_ONE];
        for i in 0..MAX_CARD_PERMS_ONE {
            let deficit = need[i].saturating_sub(have[i]);
            inferred_constraint[player].extend(std::iter::repeat_n(
                Card::try_from(i as u8).unwrap(),
                deficit as usize,
            ));
            added[i] = deficit;
        }

        if f(inferred_constraint) {
            return true;
        }

        for (i, count) in added.iter().enumerate() {
            let card = Card::try_from(i as u8).unwrap();
            for _ in 0..*count as usize {
                if let Some(pos) = inferred_constraint[player].iter().rposition(|x| *x == card) {
                    inferred_constraint[player].swap_remove(pos);
                }
            }
        }
        false
    }
    /// Exchange draw public: Try all possible combinations of moving 2 cards from player to pile
    /// This tests: 2 known cards, 1 known + 1 unstated, or 2 unstated cards
    #[inline(always)]
    pub fn exchange_draw_public(
        inferred_constraint: &mut [Vec<Card>],
        player_loop: usize,
        pile_index: usize,
        f: impl Fn(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        let player_hand_len = inferred_constraint[player_loop].len();
        if inferred_constraint[INDEX_PILE].len() > 1 {
            return false;
        }
        // Get unique cards in player hand
        let mut iter_cards_player: ArrayVec<Card, 4> = ArrayVec::new();
        for &card in inferred_constraint[player_loop].iter() {
            if !iter_cards_player.contains(&card) {
                // SAFETY: Player has at most 4 cards, so at most 4 unique cards
                unsafe {
                    iter_cards_player.push_unchecked(card);
                }
            }
        }
        iter_cards_player.sort_unstable();

        // Count cards in player hand
        let mut player_count = [0u8; 5];
        for &card in inferred_constraint[player_loop].iter() {
            player_count[card as usize] += 1;
        }

        // Case 0: Move 0 cards (2 unstated cards)
        if player_hand_len <= 2 && inferred_constraint[pile_index].len() <= 3
            && f(inferred_constraint) {
                return true;
            }

        // Case 1: Move 1 known card (1 known card + 1 unstated card)
        if player_hand_len <= 3 && inferred_constraint[pile_index].len() <= 2 {
            for &card in iter_cards_player.iter() {
                if let Some(pos) = inferred_constraint[player_loop]
                    .iter()
                    .position(|c| *c == card)
                {
                    let removed_card = inferred_constraint[player_loop].swap_remove(pos);
                    inferred_constraint[pile_index].push(removed_card);

                    if inferred_constraint[player_loop].len() <= 2
                        && inferred_constraint[pile_index].len() <= 3
                        && f(inferred_constraint)
                    {
                        // Restore before returning
                        if let Some(pos) = inferred_constraint[pile_index]
                            .iter()
                            .rposition(|c| *c == card)
                        {
                            inferred_constraint[pile_index].swap_remove(pos);
                        }
                        inferred_constraint[player_loop].push(removed_card);
                        return true;
                    }

                    // Restore
                    if let Some(pos) = inferred_constraint[pile_index]
                        .iter()
                        .rposition(|c| *c == card)
                    {
                        inferred_constraint[pile_index].swap_remove(pos);
                    }
                    inferred_constraint[player_loop].push(removed_card);
                }
            }
        }

        // Case 2: Move 2 known cards
        if (2..=4).contains(&player_hand_len)
            && inferred_constraint[pile_index].len() <= 1
        {
            for idx0 in 0..iter_cards_player.len() {
                for idx1 in idx0..iter_cards_player.len() {
                    let card0 = iter_cards_player[idx0];
                    let card1 = iter_cards_player[idx1];

                    // Check if we have enough cards to move
                    if idx0 == idx1 && player_count[card0 as usize] < 2 {
                        continue;
                    }

                    let mut removed_cards: ArrayVec<Card, 2> = ArrayVec::new();

                    // Remove first card
                    if let Some(pos) = inferred_constraint[player_loop]
                        .iter()
                        .position(|c| *c == card0)
                    {
                        let card = inferred_constraint[player_loop].swap_remove(pos);
                        // SAFETY: We're adding at most 2 cards
                        unsafe {
                            removed_cards.push_unchecked(card);
                        }
                    }

                    // Remove second card
                    if let Some(pos) = inferred_constraint[player_loop]
                        .iter()
                        .position(|c| *c == card1)
                    {
                        let card = inferred_constraint[player_loop].swap_remove(pos);
                        // SAFETY: We're adding at most 2 cards
                        unsafe {
                            removed_cards.push_unchecked(card);
                        }
                    }

                    // Add to pile
                    inferred_constraint[pile_index].extend(removed_cards.iter());

                    if f(inferred_constraint) {
                        return true;
                    }

                    // Restore
                    for &card in removed_cards.iter().rev() {
                        if let Some(pos) = inferred_constraint[pile_index]
                            .iter()
                            .rposition(|c| *c == card)
                        {
                            inferred_constraint[pile_index].swap_remove(pos);
                        }
                    }
                    inferred_constraint[player_loop].extend(removed_cards.iter());
                }
            }
        }

        false
    }
    /// Exchange draw private: Move specific 2 cards from player to pile
    /// Cards may or may not be in player's hand (if not, they're treated as unstated)
    #[inline(always)]
    pub fn exchange_draw_private(
        inferred_constraint: &mut [Vec<Card>],
        player_loop: usize,
        pile_index: usize,
        draw: &[Card],
        f: impl FnOnce(&mut [Vec<Card>]) -> bool,
    ) -> bool {
        if draw.len() != 2
            || inferred_constraint[INDEX_PILE].len() > 1
            || inferred_constraint[player_loop].len() > 4
        {
            return false;
        }
        let max_cards_added_but_not_removed = 4 - inferred_constraint[player_loop].len();
        let card0 = draw[0];
        let card1 = draw[1];

        let mut removed_cards: ArrayVec<Card, 2> = ArrayVec::new();

        // Try to remove card0 from player hand
        if let Some(pos) = inferred_constraint[player_loop]
            .iter()
            .position(|c| *c == card0)
        {
            let card = inferred_constraint[player_loop].swap_remove(pos);
            // SAFETY: We're adding at most 2 cards
            unsafe {
                removed_cards.push_unchecked(card);
            }
        }

        // Try to remove card1 from player hand
        if let Some(pos) = inferred_constraint[player_loop]
            .iter()
            .position(|c| *c == card1)
        {
            let card = inferred_constraint[player_loop].swap_remove(pos);
            // SAFETY: We're adding at most 2 cards
            unsafe {
                removed_cards.push_unchecked(card);
            }
        }

        // Add both cards to pile
        inferred_constraint[pile_index].push(card0);
        inferred_constraint[pile_index].push(card1);

        // Check validity: player must have had space for cards they didn't have
        let cards_added_but_not_removed = 2 - removed_cards.len();
        let original_size = inferred_constraint[player_loop].len() + cards_added_but_not_removed;

        // Player must have had enough space for the cards
        let valid = inferred_constraint[player_loop].len() <= 2
            && inferred_constraint[pile_index].len() <= 3
            && cards_added_but_not_removed <= max_cards_added_but_not_removed
            && original_size <= 4;

        if valid && f(inferred_constraint) {
            return true;
        }

        // Restore
        if let Some(pos) = inferred_constraint[pile_index]
            .iter()
            .rposition(|c| *c == card1)
        {
            inferred_constraint[pile_index].swap_remove(pos);
        }
        if let Some(pos) = inferred_constraint[pile_index]
            .iter()
            .rposition(|c| *c == card0)
        {
            inferred_constraint[pile_index].swap_remove(pos);
        }
        inferred_constraint[player_loop].extend(removed_cards.iter());

        false
    }
}
