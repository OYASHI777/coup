use crate::{
    history_public::Card,
    prob_manager::engine::constants::{MAX_CARD_PERMS_ONE, MAX_PLAYER_HAND_SIZE},
};

/// This aids in resetting inferred constraint after card swaps between players
pub struct MoveGuard;

impl MoveGuard {
    #[inline(always)]
    pub fn swap(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        Self::swap_run_swap_back(
            public_constraint,
            inferred_constraint,
            player_a,
            player_b,
            a_to_b,
            b_to_a,
            f,
        )
    }
    /// Here it is intended that the cards are
    ///     - removed from BOTH
    ///     - then added to BOTH
    /// NOT
    ///     - removed from b
    ///     - added to a
    ///     - then removed from a
    ///     - then added to b
    #[inline(always)]
    pub fn swap_run_clone_back(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        let backup_player_a = inferred_constraint[player_a].clone();
        let backup_player_b = inferred_constraint[player_b].clone();
        for c in a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_a]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_a].swap_remove(pos);
                inferred_constraint[player_b].push(card);
            }
        }
        for c in b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_b]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_b].swap_remove(pos);
                inferred_constraint[player_a].push(card);
            }
        }
        if public_constraint[player_a].len() + inferred_constraint[player_a].len()
            <= MAX_PLAYER_HAND_SIZE[player_a]
            && public_constraint[player_b].len() + inferred_constraint[player_b].len()
                <= MAX_PLAYER_HAND_SIZE[player_b]
            && f(public_constraint, inferred_constraint)
        {
            return true;
        }
        inferred_constraint[player_a] = backup_player_a;
        inferred_constraint[player_b] = backup_player_b;
        false
    }
    /// Here it is intended that the cards are
    ///     - removed from BOTH
    ///     - then added to BOTH
    /// NOT
    ///     - removed from b
    ///     - added to a
    ///     - then removed from a
    ///     - then added to b
    #[inline(always)]
    pub fn swap_run_swap_back(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        let mut moved_from_a_to_b: Vec<Card> = Vec::with_capacity(a_to_b.len());
        let mut moved_from_b_to_a: Vec<Card> = Vec::with_capacity(b_to_a.len());
        for c in a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_a]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_a].swap_remove(pos);
                moved_from_a_to_b.push(card);
            }
        }
        for c in b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_b]
                .iter()
                .rposition(|card| card == c)
            {
                let card = inferred_constraint[player_b].swap_remove(pos);
                moved_from_b_to_a.push(card);
            }
        }
        inferred_constraint[player_b].extend(moved_from_a_to_b.iter());
        inferred_constraint[player_a].extend(moved_from_b_to_a.iter());
        if public_constraint[player_a].len() + inferred_constraint[player_a].len()
            <= MAX_PLAYER_HAND_SIZE[player_a]
            && public_constraint[player_b].len() + inferred_constraint[player_b].len()
                <= MAX_PLAYER_HAND_SIZE[player_b]
            && f(public_constraint, inferred_constraint)
        {
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
    pub fn ordered_swap(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player_a: usize,
        player_b: usize,
        a_to_b: &[Card],
        b_to_a: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        let mut removed_from_b: Vec<Card> = Vec::with_capacity(b_to_a.len());
        let mut removed_from_a: Vec<Card> = Vec::with_capacity(a_to_b.len());
        for &c in a_to_b {
            if let Some(pos) = inferred_constraint[player_b].iter().rposition(|x| *x == c) {
                inferred_constraint[player_b].swap_remove(pos);
                removed_from_b.push(c);
            }
            inferred_constraint[player_a].push(c);
        }
        for &c in b_to_a {
            if let Some(pos) = inferred_constraint[player_a].iter().rposition(|x| *x == c) {
                inferred_constraint[player_a].swap_remove(pos);
                removed_from_a.push(c);
            }
            inferred_constraint[player_b].push(c);
        }
        if public_constraint[player_a].len() + inferred_constraint[player_a].len()
            <= MAX_PLAYER_HAND_SIZE[player_a]
            && public_constraint[player_b].len() + inferred_constraint[player_b].len()
                <= MAX_PLAYER_HAND_SIZE[player_b]
            && f(public_constraint, inferred_constraint)
        {
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
    pub fn reveal_none_pull_only_run_reset(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        pile: usize,   // usually 6
        reveal: Card,
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        let mut removed_reveal_from_pile = false;
        if let Some(pos) = inferred_constraint[pile].iter().rposition(|c| *c == reveal) {
            inferred_constraint[pile].swap_remove(pos);
            removed_reveal_from_pile = true;
        }
        inferred_constraint[player].push(reveal); // always push

        let ok = f(public_constraint, inferred_constraint);

        if let Some(pos) = inferred_constraint[player].iter().rposition(|c| *c == reveal) {
            inferred_constraint[player].swap_remove(pos);
        }
        if removed_reveal_from_pile {
            inferred_constraint[pile].push(reveal);
        }

        ok
    }
    #[inline(always)]
    pub fn reveal_none_swap_then_pull_run_reset(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        pile: usize,   // usually 6
        reveal: Card,
        swap_card: Card,
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        // forward: player -> pile (swap_card)
        let mut removed_swap_from_player = false;
        if let Some(pos) = inferred_constraint[player]
            .iter()
            .position(|c| *c == swap_card) // match your original .position()
        {
            inferred_constraint[player].swap_remove(pos);
            removed_swap_from_player = true;
        }
        inferred_constraint[pile].push(swap_card); // always push

        // forward: pile -> player (reveal)
        let mut removed_reveal_from_pile = false;
        if let Some(pos) = inferred_constraint[pile].iter().rposition(|c| *c == reveal) {
            inferred_constraint[pile].swap_remove(pos);
            removed_reveal_from_pile = true;
        }
        inferred_constraint[player].push(reveal); // always push

        // user function
        let ok = f(public_constraint, inferred_constraint);

        // rollback (reverse order)
        if let Some(pos) = inferred_constraint[player].iter().rposition(|c| *c == reveal) {
            inferred_constraint[player].swap_remove(pos);
        }
        if removed_reveal_from_pile {
            inferred_constraint[pile].push(reveal);
        }

        if let Some(pos) = inferred_constraint[pile].iter().rposition(|c| *c == swap_card) {
            inferred_constraint[pile].swap_remove(pos);
        }
        if removed_swap_from_player {
            inferred_constraint[player].push(swap_card);
        }

        ok
    }
    #[inline(always)]
    pub fn discard(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        card: Card,
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
    ) -> bool {
        let mut removed_discard = false;
        if let Some(pos) = public_constraint[player].iter().rposition(|c| *c == card) {
            public_constraint[player].swap_remove(pos);
            removed_discard = true;
        }
        inferred_constraint[player].push(card);
        if f(public_constraint, inferred_constraint) {
            return true;
        }
        if let Some(pos) = inferred_constraint[player].iter().rposition(|c| *c == card) {
            inferred_constraint[player].swap_remove(pos);
        }
        if removed_discard {
            public_constraint[player].push(card);
        }
        false
    }
    /// Ensures a player's hand contains at least the cards in needed
    #[inline(always)]
    pub fn with_needed_cards_present(
        public_constraint: &mut Vec<Vec<Card>>,
        inferred_constraint: &mut Vec<Vec<Card>>,
        player: usize,
        needed: &[Card],
        f: impl FnOnce(&mut Vec<Vec<Card>>, &mut Vec<Vec<Card>>) -> bool,
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
            let deficit = if need[i] > have[i] {
                need[i] - have[i]
            } else {
                0
            };
            inferred_constraint[player]
                .extend(std::iter::repeat(Card::try_from(i as u8).unwrap()).take(deficit as usize));
            added[i] = deficit;
        }

        if f(public_constraint, inferred_constraint) {
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
