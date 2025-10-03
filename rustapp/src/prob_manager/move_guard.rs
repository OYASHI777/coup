use arrayvec::ArrayVec;

use crate::{history_public::Card, prob_manager::engine::constants::MAX_CARD_PERMS_ONE};

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
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool,
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
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool,
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
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        card: Card,
        f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool,
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
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        needed: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool,
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
}
