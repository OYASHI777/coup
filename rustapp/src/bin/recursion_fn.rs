use env_logger::{Builder, Env, Target};
use itertools::Itertools;
use log::LevelFilter;
use rustapp::history_public::Card;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
const LOG_FILE_NAME: &str = "recursion_fn.log";
fn main() {
    test_variant_recurse();
}
pub fn test_variant_recurse() {
    // RecursionTest::test_variant_recurse(LOG_FILE_NAME);
    RecursionTest::test_exchange_draw_public(LOG_FILE_NAME);
}
pub struct RecursionTest;

impl RecursionTest {
    pub fn return_variants_reveal_redraw(
        reveal: Card,
        redraw: Card,
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::new();
        let mut temp = inferred_constraints.to_vec();
        if let Some(pos) = temp[player_loop].iter().position(|c| *c == redraw) {
            temp[player_loop].swap_remove(pos);
        }
        temp[6].push(redraw);
        if let Some(pos) = temp[6].iter().position(|c| *c == reveal) {
            temp[6].swap_remove(pos);
        }
        temp[player_loop].push(reveal);
        if temp[6].len() < 4 && temp[player_loop].len() < 3 {
            variants.push(temp);
        }
        variants
    }
    pub fn return_variants_reveal_redraw_none(
        reveal: Card,
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[player_loop].len() + inferred_constraints[6].len() == 5
            && !inferred_constraints[player_loop].contains(&reveal)
            && !inferred_constraints[6].contains(&reveal)
        {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        // TODO: OPTIMIZE Probably don't need this source thing
        // This needs to have player indexes before pile indexes
        // we check what player redrew first, which moves item from player to pile
        // then we check what could have moved from pile to player
        let source_cards: Vec<(usize, Card)> = inferred_constraints[player_loop]
            .iter()
            .copied()
            .map(|c| (player_loop, c))
            .chain(inferred_constraints[6].iter().copied().map(|c| (6, c)))
            .collect();
        // TODO: Consider moving this out of this function
        // Self::build_variants_reveal_redraw_none(reveal, &source_cards, 0, player_loop, 6, 0, 0, &inferred_constraints, &mut variants);
        Self::build_variants_reveal_redraw_none_opt(
            reveal,
            &source_cards,
            0,
            player_loop,
            6,
            0,
            0,
            inferred_constraints,
            &mut variants,
        );
        variants
    }
    /// Builds possible previous inferred_constraint states
    /// All cards have a source
    /// card == reveal in player hand can come from
    /// - player without moving
    /// - player after revealing then redrawing the same
    /// - pile after revealing then redrawing different
    /// - player after revealing then redrawing different
    ///
    /// Not trying that many cases here as some of them just lead to more specific sets that will already be covered
    /// e.g.
    /// src: [[Ambassador], [], [], [], [], [], [Assassin, Ambassador]], [Ambassador, Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
    /// dest: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
    /// the second is a more specific case of the first item in src
    #[allow(clippy::too_many_arguments)]
    fn build_variants_reveal_redraw_none_opt(
        reveal: Card,
        cards: &[(usize, Card)],
        idx: usize,
        player_loop: usize,
        pile_index: usize,
        player_to_pile_count: u8, // dst -> src
        pile_to_player_count: u8, // dst -> src
        current: &[Vec<Card>],
        variants: &mut Vec<Vec<Vec<Card>>>,
    ) {
        // build an intermediate state
        // src: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin, Ambassador]],
        // dest: [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // This is ok, but Im thinking u technically don't need this cos it will be searched by
        // TODO: OPTIMIZE and remove subsets
        // [[Ambassador], [], [], [], [], [], [Ambassador, Assassin]]
        // src: []
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Assassin, Captain, Captain]]
        // P0 RR AMB None
        // src: [[Ambassador], [], [], [], [], [], [Assassin, Assassin, Assassin]]
        // dest: [[Ambassador, Assassin], [], [], [], [], [], [Assassin, Assassin]]
        // src should have 2 Ambassador as 1 stayed and one was revealed
        // if you redrew a card, and the pile is 3 cards after that means the other card had to have been relinquished...
        // P0 RR AMB None
        // Would be impossible here as there is no room for AMB in player or pile in dest,
        // dest: [[Captain, Duke], [], [], [], [], [], [Captain, Captain, Contessa]]
        // src: [[[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]], [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain, Ambassador]]]
        // dest: [[Ambassador, Ambassador], [], [], [], [], [], [Captain, Captain]]
        if idx == cards.len() {
            // TODO: OPTIMIZE the push
            let mut source_constraints = current.to_vec();
            // Player redrew same card
            // Not exactly right as player_to_pile could be a non ambassador
            // Add if player_to_pile is 0
            if !source_constraints[player_loop].contains(&reveal) {
                // TESTING OPTIMIZE
                source_constraints[player_loop].push(reveal);
            }
            if source_constraints[player_loop].len() < 3
                && source_constraints[pile_index].len() < 4
                && source_constraints
                    .iter()
                    .map(|v| v.iter().filter(|c| **c == reveal).count() as u8)
                    .sum::<u8>()
                    < 4
            {
                variants.push(source_constraints);
            }
            return;
        }

        // CASE: player Card did not come from pile after reveal
        // CASE: pile Card did not come from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        Self::build_variants_reveal_redraw_none_opt(
            reveal,
            cards,
            idx + 1,
            player_loop,
            pile_index,
            player_to_pile_count,
            pile_to_player_count,
            current,
            variants,
        );
        // Destructure this card's source and value
        let (dst, card) = cards[idx];
        // OPTIMIZE: probably could just run this as the ONLY case for reveal redraw as the other case will be a more specific case of this
        // CASE: Card originally left player hand and returned to player and is the same unique card
        // T                :  [C0]    []
        // T after reveal   :  []    [C0]
        // T + 1            :  [C0]    []
        if dst == player_loop && card == reveal {
            // No need to add reveal anymore
            Self::build_variants_reveal_redraw_none_opt(
                reveal,
                cards,
                cards.len(),
                player_loop,
                pile_index,
                1,
                1,
                current,
                variants,
            );
            return;
        }

        let is_player_card = dst == player_loop;
        let could_have_swapped = if is_player_card {
            player_to_pile_count < 1
            // Testing OPTIMIZE: exclude all other reveal redraw same cards
            // && card != reveal
        } else {
            pile_to_player_count < 1 && card == reveal
        };
        // CASE: player Card came from pile after reveal
        // CASE: pile Card came from player after reveal
        // ADDITIONALLY: if player reveal and redrew the same card, it is considered seperate cards
        if could_have_swapped {
            let (src, new_player_to_pile_count, new_pile_to_player_count) = if is_player_card {
                (pile_index, player_to_pile_count + 1, pile_to_player_count)
            } else {
                (player_loop, player_to_pile_count, pile_to_player_count + 1)
            };

            let mut new_constraints = current.to_vec();
            if let Some(pos) = new_constraints[dst].iter().position(|&c| c == card) {
                new_constraints[dst].swap_remove(pos);
                new_constraints[src].push(card);
                Self::build_variants_reveal_redraw_none_opt(
                    reveal,
                    cards,
                    idx + 1,
                    player_loop,
                    pile_index,
                    new_player_to_pile_count,
                    new_pile_to_player_count,
                    &new_constraints,
                    variants,
                );
            }
        }
    }
    // TODO: modify inferred_constraints and recurse when no longer testing this
    pub fn return_variants_reveal_redraw_none_opt(
        reveal: Card,
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[player_loop].len() + inferred_constraints[6].len() == 5
            && !inferred_constraints[player_loop].contains(&reveal)
            && !inferred_constraints[6].contains(&reveal)
        {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        let mut player_hand = inferred_constraints[player_loop].clone();
        let mut pile_hand = inferred_constraints[6].clone();
        if player_hand.is_empty() {
            // let mut bool_move_from_pile_to_player = false;
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.to_vec();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();
            if temp[player_loop].len() < 3
                && temp
                    .iter()
                    .map(|v| v.iter().filter(|c| **c == reveal).count() as u8)
                    .sum::<u8>()
                    < 4
            {
                // TODO: Recurse here in other version
                variants.push(temp.to_vec());
            }
            return variants;
            // TODO: remove if recursing
        }
        let mut iter_cards = inferred_constraints[player_loop].clone();
        iter_cards.sort_unstable();
        iter_cards.dedup();
        // Doesnt handle empty case
        for card_player in iter_cards.iter() {
            // Card Source was not from Pile
            let mut bool_move_from_pile_to_player = false;
            if *card_player != reveal || inferred_constraints[6].contains(&reveal) {
                if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                    pile_hand.swap_remove(pos);
                    bool_move_from_pile_to_player = true;
                }
                player_hand.push(reveal);
                let mut temp = inferred_constraints.to_vec();
                temp[player_loop] = player_hand.clone();
                temp[6] = pile_hand.clone();

                if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                    player_hand.swap_remove(pos);
                }
                if bool_move_from_pile_to_player {
                    pile_hand.push(reveal);
                }
                // Probably need push only if certain conditions met
                if temp[player_loop].len() < 3
                    && temp[6].len() < 4
                    && temp
                        .iter()
                        .map(|v| v.iter().filter(|c| **c == reveal).count() as u8)
                        .sum::<u8>()
                        < 4
                {
                    // TODO: Recurse here in other version
                    variants.push(temp.to_vec());
                }
            }
            // TEMP
            // player_hand.sort();
            // pile_hand.sort();
            // let mut checker = inferred_constraints.clone();
            // checker[player_loop].sort();
            // checker[6].sort();
            // if player_hand != checker[player_loop] {
            //     log::warn!("failed 1 to pop player hand properly");
            //     log::warn!("player_hand now: {:?}", player_hand);
            // }
            // if pile_hand != checker[6] {
            //     log::warn!("failed 1 to pop pile hand properly");
            //     log::warn!("pile_hand now: {:?}", pile_hand);
            // }
            // Card Source was from Pile
            player_hand = inferred_constraints[player_loop].clone();
            pile_hand = inferred_constraints[6].clone();
            bool_move_from_pile_to_player = false;
            if let Some(pos) = player_hand.iter().position(|c| *c == *card_player) {
                player_hand.swap_remove(pos);
            }
            pile_hand.push(*card_player);
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
                bool_move_from_pile_to_player = true;
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.to_vec();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();

            // Probably need push only if certain conditions met
            if temp[player_loop].len() < 3
                && temp[6].len() < 4
                && temp
                    .iter()
                    .map(|v| v.iter().filter(|c| **c == reveal).count() as u8)
                    .sum::<u8>()
                    < 4
            {
                // TODO: Recurse here in other version
                variants.push(temp.to_vec());
            }

            if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                player_hand.swap_remove(pos);
            }
            if bool_move_from_pile_to_player {
                pile_hand.push(reveal);
            }
            if let Some(pos) = pile_hand.iter().rposition(|c| c == card_player) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(*card_player);
            // TEMP
            // player_hand.sort();
            // pile_hand.sort();
            // let mut checker = inferred_constraints.clone();
            // checker[player_loop].sort();
            // checker[6].sort();
            // if player_hand != checker[player_loop] {
            //     log::warn!("failed 2 to pop player hand properly");
            // }
            // if pile_hand != checker[6] {
            //     log::warn!("failed 2 to pop pile hand properly");
            // }
        }
        variants
    }
    // TODO: modify inferred_constraints and recurse when no longer testing this
    pub fn return_variants_exchange_opt(
        player_lives: u8,
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        let mut iter_cards_player = inferred_constraints[player_loop].clone();
        iter_cards_player.sort_unstable();
        iter_cards_player.dedup();
        let mut iter_cards_pile = inferred_constraints[6].clone();
        iter_cards_pile.sort_unstable();
        iter_cards_pile.dedup();
        let mut player_count = [0u8; 5];
        let mut pile_count = [0u8; 5];
        inferred_constraints[player_loop]
            .iter()
            .for_each(|c| player_count[*c as usize] += 1);
        inferred_constraints[6]
            .iter()
            .for_each(|c| pile_count[*c as usize] += 1);
        // let mut redraw_count = [0u8; 5];
        // let mut relinquish_count = [0u8; 5];
        // redraw.iter().for_each(|c| redraw_count[*c as usize] += 1);
        // relinquish.iter().for_each(|c| relinquish_count[*c as usize] += 1);

        // Can maybe consider all possible unique moves characterized by player_to_pile and pile_to_player
        // redraw_count and relinquish_count define the degree of freedom for both of those
        // the possible choices that can be player_to_pile depend on whats in player_hand
        // 2 AMB -> up to 2 AMB from player_to_pile
        // 1 AMB -> 1 AMB from player_to_pile
        // So I guess can loop through all possible version of player_to_pile and pile_to_player
        // player_count
        // 0 moves, 1 moves, 2 moves
        // NOTE AMB player to pile and AMB pile to player cancel out so no intersection of player and pile ish
        // 0 player_to_pile move, 0 pile_to_player move
        variants.push(inferred_constraints.to_vec());
        // 1 player_to_pile move, 0 pile_to_player move
        if inferred_constraints[6].len() < 3 && !inferred_constraints[player_loop].is_empty() {
            for card_player in iter_cards_player.iter() {
                // move to pile
                let mut player_hand = inferred_constraints[player_loop].clone();
                let mut pile_hand = inferred_constraints[6].clone();
                if let Some(pos) = player_hand.iter().rposition(|c| *c == *card_player) {
                    player_hand.swap_remove(pos);
                    pile_hand.push(*card_player);
                    let mut temp = inferred_constraints.to_vec();
                    temp[player_loop] = player_hand.clone();
                    temp[6] = pile_hand.clone();
                    variants.push(temp.to_vec());
                }
            }
        }
        // 0 player_to_pile move, 1 pile_to_player move
        if inferred_constraints[player_loop].len() < 2 && !inferred_constraints[6].is_empty() {
            for card_pile in iter_cards_pile.iter() {
                // move to player
                let mut player_hand = inferred_constraints[player_loop].clone();
                let mut pile_hand = inferred_constraints[6].clone();
                if let Some(pos) = pile_hand.iter().rposition(|c| *c == *card_pile) {
                    pile_hand.swap_remove(pos);
                    player_hand.push(*card_pile);
                    let mut temp = inferred_constraints.to_vec();
                    temp[player_loop] = player_hand.clone();
                    temp[6] = pile_hand.clone();
                    variants.push(temp.to_vec());
                }
            }
        }
        // 1 player_to_pile move, 1 pile_to_player move
        if !inferred_constraints[player_loop].is_empty() && !inferred_constraints[6].is_empty() {
            for card_player in iter_cards_player.iter() {
                for card_pile in iter_cards_pile.iter() {
                    if card_player == card_pile {
                        continue;
                    }
                    let mut player_hand = inferred_constraints[player_loop].clone();
                    let mut pile_hand = inferred_constraints[6].clone();
                    if let Some(pos) = pile_hand.iter().rposition(|c| *c == *card_pile) {
                        pile_hand.swap_remove(pos);
                    }
                    if let Some(pos) = player_hand.iter().rposition(|c| *c == *card_player) {
                        player_hand.swap_remove(pos);
                    }
                    pile_hand.push(*card_player);
                    player_hand.push(*card_pile);
                    let mut temp = inferred_constraints.to_vec();
                    temp[player_loop] = player_hand.clone();
                    temp[6] = pile_hand.clone();
                    variants.push(temp.to_vec());
                }
            }
        }
        if player_lives > 1 {
            // 2 player_to_pile move, 0 pile_to_player move
            if inferred_constraints[player_loop].len() == 2 && inferred_constraints[6].len() < 2 {
                let card_0 = inferred_constraints[player_loop][0];
                let card_1 = inferred_constraints[player_loop][1];
                let mut player_hand = inferred_constraints[player_loop].clone();
                let mut pile_hand = inferred_constraints[6].clone();
                player_hand.clear();
                pile_hand.push(card_0);
                pile_hand.push(card_1);
                let mut temp = inferred_constraints.to_vec();
                temp[player_loop] = player_hand.clone();
                temp[6] = pile_hand.clone();
                variants.push(temp.to_vec());
            }
            // 0 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].is_empty() && inferred_constraints[6].len() > 1 {
                for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                    for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len() {
                        if index_pile_to_player_0 == index_pile_to_player_1
                            && pile_count[iter_cards_pile[index_pile_to_player_0] as usize] < 2
                        {
                            continue; // Ensure enough cards to move
                        }
                        let mut player_hand = inferred_constraints[player_loop].clone();
                        let mut pile_hand = inferred_constraints[6].clone();
                        if let Some(pos) = pile_hand
                            .iter()
                            .rposition(|c| *c == iter_cards_pile[index_pile_to_player_0])
                        {
                            pile_hand.swap_remove(pos);
                        }
                        if let Some(pos) = pile_hand
                            .iter()
                            .rposition(|c| *c == iter_cards_pile[index_pile_to_player_1])
                        {
                            pile_hand.swap_remove(pos);
                        }
                        player_hand.push(iter_cards_pile[index_pile_to_player_0]);
                        player_hand.push(iter_cards_pile[index_pile_to_player_1]);
                        let mut temp = inferred_constraints.to_vec();
                        temp[player_loop] = player_hand.clone();
                        temp[6] = pile_hand.clone();
                        variants.push(temp.to_vec());
                    }
                }
            }
            // 2 player_to_pile move, 1 pile_to_player move
            if !inferred_constraints[6].is_empty()
                && inferred_constraints[6].len() < 3
                && inferred_constraints[player_loop].len() > 1
            {
                for card_pile in iter_cards_pile.iter() {
                    for index_player_to_pile_0 in 0..iter_cards_player.len() {
                        // TODO: Shift index_player_to_pile == case shift here
                        if iter_cards_player[index_player_to_pile_0] == *card_pile {
                            continue; // Avoid duplicates
                        }
                        for index_player_to_pile_1 in
                            index_player_to_pile_0..iter_cards_player.len()
                        {
                            // Check DF
                            if iter_cards_player[index_player_to_pile_1] == *card_pile {
                                continue; // Avoid duplicates
                            }
                            if index_player_to_pile_0 == index_player_to_pile_1
                                && player_count[iter_cards_player[index_player_to_pile_0] as usize]
                                    < 2
                            {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue; // Ensure enough cards to move
                            }
                            let mut player_hand = inferred_constraints[player_loop].clone();
                            let mut pile_hand = inferred_constraints[6].clone();
                            if let Some(pos) = player_hand
                                .iter()
                                .rposition(|c| *c == iter_cards_player[index_player_to_pile_0])
                            {
                                player_hand.swap_remove(pos);
                            }
                            if let Some(pos) = player_hand
                                .iter()
                                .rposition(|c| *c == iter_cards_player[index_player_to_pile_1])
                            {
                                player_hand.swap_remove(pos);
                            }
                            if let Some(pos) = pile_hand.iter().rposition(|c| *c == *card_pile) {
                                pile_hand.swap_remove(pos);
                            }
                            pile_hand.push(iter_cards_player[index_player_to_pile_0]);
                            pile_hand.push(iter_cards_player[index_player_to_pile_1]);
                            player_hand.push(*card_pile);
                            let mut temp = inferred_constraints.to_vec();
                            temp[player_loop] = player_hand.clone();
                            temp[6] = pile_hand.clone();
                            variants.push(temp.to_vec());
                        }
                    }
                }
            }
            // 1 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() == 1 && inferred_constraints[6].len() > 1 {
                for card_player in iter_cards_player.iter() {
                    for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                        if iter_cards_pile[index_pile_to_player_0] == *card_player {
                            continue; // Avoid Duplicates
                        }
                        for index_pile_to_player_1 in index_pile_to_player_0..iter_cards_pile.len()
                        {
                            // Check DF
                            if iter_cards_pile[index_pile_to_player_1] == *card_player {
                                continue; // Avoid Duplicates
                            }
                            if index_pile_to_player_0 == index_pile_to_player_1
                                && (pile_count[iter_cards_pile[index_pile_to_player_0] as usize]
                                    < 2)
                            {
                                // Checks that player has enough cards to move out
                                // TODO: OPTIMIZE Can shift this out of for loop actually
                                continue; // Ensure enough cards to move
                            }
                            let mut player_hand = inferred_constraints[player_loop].clone();
                            let mut pile_hand = inferred_constraints[6].clone();
                            if let Some(pos) = pile_hand
                                .iter()
                                .rposition(|c| *c == iter_cards_pile[index_pile_to_player_0])
                            {
                                pile_hand.swap_remove(pos);
                            }
                            if let Some(pos) = pile_hand
                                .iter()
                                .rposition(|c| *c == iter_cards_pile[index_pile_to_player_1])
                            {
                                pile_hand.swap_remove(pos);
                            }
                            if let Some(pos) = player_hand.iter().rposition(|c| *c == *card_player)
                            {
                                player_hand.swap_remove(pos);
                            }
                            player_hand.push(iter_cards_pile[index_pile_to_player_0]);
                            player_hand.push(iter_cards_pile[index_pile_to_player_1]);
                            pile_hand.push(*card_player);
                            let mut temp = inferred_constraints.to_vec();
                            temp[player_loop] = player_hand.clone();
                            temp[6] = pile_hand.clone();
                            variants.push(temp.to_vec());
                        }
                    }
                }
            }
            // 2 player_to_pile move, 2 pile_to_player move
            if inferred_constraints[player_loop].len() > 1 && inferred_constraints[6].len() > 1 {
                for index_player_to_pile_0 in 0..iter_cards_player.len() {
                    for index_player_to_pile_1 in index_player_to_pile_0..iter_cards_player.len() {
                        if index_player_to_pile_0 == index_player_to_pile_1
                            && player_count[iter_cards_player[index_player_to_pile_0] as usize] < 2
                        {
                            // Checks that player has enough cards to move out
                            // TODO: OPTIMIZE Can shift this out of for loop actually
                            continue; // Ensure enough cards to move
                        }
                        // Check DF
                        for index_pile_to_player_0 in 0..iter_cards_pile.len() {
                            if iter_cards_pile[index_pile_to_player_0]
                                == iter_cards_player[index_player_to_pile_0]
                                || iter_cards_pile[index_pile_to_player_0]
                                    == iter_cards_player[index_player_to_pile_1]
                            {
                                continue; // Avoid Duplicates
                            }
                            for index_pile_to_player_1 in
                                index_pile_to_player_0..iter_cards_pile.len()
                            {
                                if iter_cards_pile[index_pile_to_player_1]
                                    == iter_cards_player[index_player_to_pile_0]
                                    || iter_cards_pile[index_pile_to_player_1]
                                        == iter_cards_player[index_player_to_pile_1]
                                {
                                    continue; // Avoid Duplicates
                                }
                                if index_pile_to_player_0 == index_pile_to_player_1
                                    && (pile_count
                                        [iter_cards_pile[index_pile_to_player_0] as usize]
                                        < 2)
                                {
                                    // Checks that player has enough cards to move out
                                    // TODO: OPTIMIZE Can shift this out of for loop actually
                                    continue; // Ensure enough cards to move
                                }
                                let mut player_hand = inferred_constraints[player_loop].clone();
                                let mut pile_hand = inferred_constraints[6].clone();
                                if let Some(pos) = pile_hand
                                    .iter()
                                    .rposition(|c| *c == iter_cards_pile[index_pile_to_player_0])
                                {
                                    pile_hand.swap_remove(pos);
                                }
                                if let Some(pos) = pile_hand
                                    .iter()
                                    .rposition(|c| *c == iter_cards_pile[index_pile_to_player_1])
                                {
                                    pile_hand.swap_remove(pos);
                                }
                                if let Some(pos) = player_hand
                                    .iter()
                                    .rposition(|c| *c == iter_cards_player[index_player_to_pile_0])
                                {
                                    player_hand.swap_remove(pos);
                                }
                                if let Some(pos) = player_hand
                                    .iter()
                                    .rposition(|c| *c == iter_cards_player[index_player_to_pile_1])
                                {
                                    player_hand.swap_remove(pos);
                                }
                                player_hand.push(iter_cards_pile[index_pile_to_player_0]);
                                player_hand.push(iter_cards_pile[index_pile_to_player_1]);
                                pile_hand.push(iter_cards_player[index_player_to_pile_0]);
                                pile_hand.push(iter_cards_player[index_player_to_pile_1]);
                                let mut temp = inferred_constraints.to_vec();
                                temp[player_loop] = player_hand.clone();
                                temp[6] = pile_hand.clone();
                                variants.push(temp.to_vec());
                            }
                        }
                    }
                }
            }
        }
        variants
    }
    pub fn return_variants_exchange_private_3(
        player_loop: usize,
        draw: &[Card],
        relinquish: &[Card],
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        // TODO: [REFACTOR] maybe only need draw and relinquish?
        // TODO: [REFACTOR] I think this case might handle all cases?
        // TODO: [REFACTOR] I think don't need hand?
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(1);
        let mut temp = inferred_constraints.to_owned();
        // same card stays in pile
        // ensure
        if let Some(pos) = temp[6].iter().position(|c| *c == relinquish[0]) {
            temp[6].swap_remove(pos);
        }
        temp[player_loop].push(relinquish[0]);
        if let Some(pos) = temp[6].iter().position(|c| *c == relinquish[1]) {
            temp[6].swap_remove(pos);
        }
        temp[player_loop].push(relinquish[1]);
        if let Some(pos) = temp[player_loop].iter().position(|c| *c == draw[0]) {
            temp[player_loop].swap_remove(pos);
        }
        temp[6].push(draw[0]);
        if let Some(pos) = temp[player_loop].iter().position(|c| *c == draw[1]) {
            temp[player_loop].swap_remove(pos);
        }
        temp[6].push(draw[1]);
        // Remove this to check if able to add illegal moves! for simulation
        if temp[player_loop].len() < 3 && temp[6].len() < 4 {
            // Consider that Vec for players should have capacity 4 for this to work!
            variants.push(temp.to_vec());
        } else {
            // response = false;
        }
        variants
    }
    pub fn return_variants_reveal_relinquish_opt(
        reveal: Card,
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::with_capacity(12);
        if inferred_constraints[6].len() == 3 && !inferred_constraints[6].contains(&reveal) {
            // This state cannot be arrive after the reveal_redraw
            return Vec::with_capacity(0);
        }
        let mut player_hand = inferred_constraints[player_loop].clone();
        let mut pile_hand = inferred_constraints[6].clone();
        if player_hand.is_empty() {
            // let mut bool_move_from_pile_to_player = false;
            if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                pile_hand.swap_remove(pos);
            }
            player_hand.push(reveal);
            let mut temp = inferred_constraints.to_vec();
            temp[player_loop] = player_hand.clone();
            temp[6] = pile_hand.clone();
            if temp[player_loop].len() < 3
                && temp
                    .iter()
                    .map(|v| v.iter().filter(|c| **c == reveal).count() as u8)
                    .sum::<u8>()
                    < 4
            {
                // TODO: Recurse here in other version
                variants.push(temp.to_vec());
            }
            return variants;
            // TODO: remove if recursing
        }
        let mut iter_cards = inferred_constraints[player_loop].clone();
        iter_cards.sort_unstable();
        iter_cards.dedup();
        for card_player in iter_cards.iter() {
            // Card Source was not from Pile
            if player_hand.len() < 2 {
                let mut bool_move_from_pile_to_player = false;
                if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                    pile_hand.swap_remove(pos);
                    bool_move_from_pile_to_player = true;
                }
                player_hand.push(reveal);

                let mut temp = inferred_constraints.to_vec();
                temp[player_loop] = player_hand.clone();
                temp[6] = pile_hand.clone();

                // TODO: Recurse here in other version
                variants.push(temp.to_vec());

                if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                    player_hand.swap_remove(pos);
                }
                if bool_move_from_pile_to_player {
                    pile_hand.push(reveal);
                }
            }

            // Card Source was from Pile
            if *card_player != reveal {
                let mut bool_move_from_pile_to_player = false;
                let mut bool_move_from_player_to_pile = false;
                if let Some(pos) = player_hand.iter().position(|c| *c == *card_player) {
                    player_hand.swap_remove(pos);
                    bool_move_from_player_to_pile = true;
                }
                pile_hand.push(*card_player);
                if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
                    pile_hand.swap_remove(pos);
                    bool_move_from_pile_to_player = true;
                }
                player_hand.push(reveal);
                let mut temp = inferred_constraints.to_vec();
                temp[player_loop] = player_hand.clone();
                temp[6] = pile_hand.clone();
                // TODO: Recurse here in other version
                variants.push(temp.to_vec());

                if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
                    player_hand.swap_remove(pos);
                }
                if bool_move_from_pile_to_player {
                    pile_hand.push(reveal);
                }
                if bool_move_from_player_to_pile {
                    player_hand.push(*card_player);
                }
            }
            // Card Source was from Pile
            // if *card_player != reveal {
            //     let mut bool_move_from_pile_to_player = false;
            //     if let Some(pos) = player_hand.iter().position(|c| *c == *card_player) {
            //         player_hand.swap_remove(pos);
            //     }
            //     pile_hand.push(*card_player);
            //     if let Some(pos) = pile_hand.iter().rposition(|c| *c == reveal) {
            //         pile_hand.swap_remove(pos);
            //         bool_move_from_pile_to_player = true;
            //     }
            //     player_hand.push(reveal);
            //     let mut temp = inferred_constraints.to_vec();
            //     temp[player_loop] = player_hand.clone();
            //     temp[6] = pile_hand.clone();

            //     // TODO: Recurse here in other version
            //     variants.push(temp);

            //     if let Some(pos) = player_hand.iter().rposition(|c| *c == reveal) {
            //         player_hand.swap_remove(pos);
            //     }
            //     if bool_move_from_pile_to_player {
            //         pile_hand.push(reveal);
            //     }
            //     if let Some(pos) = pile_hand.iter().rposition(|c| c == card_player) {
            //         pile_hand.swap_remove(pos);
            //     }
            //     player_hand.push(*card_player);
            //     if inferred_constraints[player_loop].len() == 2 && inferred_constraints[player_loop][0] == inferred_constraints[player_loop][1]{
            //         break;
            //     }
            // }
        }
        variants
    }
    pub fn return_variants_exchange_draw_public(
        player_loop: usize,
        inferred_constraints: &[Vec<Card>],
    ) -> Vec<Vec<Vec<Card>>> {
        let mut variants: Vec<Vec<Vec<Card>>> = Vec::new();

        let player_hand_len = inferred_constraints[player_loop].len();
        if inferred_constraints[6].len() > 1 || player_hand_len + inferred_constraints[6].len() > 5
        {
            // pile cannot have more than 1 card!
            return variants;
        }

        // Get unique cards in player hand
        let mut iter_cards_player = inferred_constraints[player_loop].clone();
        iter_cards_player.sort_unstable();
        iter_cards_player.dedup();

        // Count cards in player hand
        let mut player_count = [0u8; 5];
        inferred_constraints[player_loop]
            .iter()
            .for_each(|c| player_count[*c as usize] += 1);

        // Case 0: Move 0 cards (2 unstated cards)
        if player_hand_len <= 2 && inferred_constraints[6].len() <= 3 - 0 {
            variants.push(inferred_constraints.to_vec());
        }

        // Case 1: Move 1 known card (1 known card + 1 unstated card)
        if player_hand_len <= 3 && inferred_constraints[6].len() <= 3 - 1 {
            for card in iter_cards_player.iter() {
                let mut player_hand = inferred_constraints[player_loop].clone();
                let mut pile_hand = inferred_constraints[6].clone();

                if let Some(pos) = player_hand.iter().position(|c| *c == *card) {
                    player_hand.swap_remove(pos);
                    pile_hand.push(*card);

                    if player_hand.len() <= 2 && pile_hand.len() <= 3 {
                        let mut temp = inferred_constraints.to_vec();
                        temp[player_loop] = player_hand;
                        temp[6] = pile_hand;
                        variants.push(temp);
                    }
                }
            }
        }

        // Case 2: Move 2 known cards
        if player_hand_len >= 2 && player_hand_len <= 4 && inferred_constraints[6].len() <= 3 - 2 {
            for idx0 in 0..iter_cards_player.len() {
                for idx1 in idx0..iter_cards_player.len() {
                    let card0 = iter_cards_player[idx0];
                    let card1 = iter_cards_player[idx1];

                    // Check if we have enough cards to move
                    if idx0 == idx1 && player_count[card0 as usize] < 2 {
                        continue;
                    }

                    let mut player_hand = inferred_constraints[player_loop].clone();
                    let mut pile_hand = inferred_constraints[6].clone();

                    // Remove first card
                    if let Some(pos) = player_hand.iter().position(|c| *c == card0) {
                        player_hand.swap_remove(pos);
                    }

                    // Remove second card
                    if let Some(pos) = player_hand.iter().position(|c| *c == card1) {
                        player_hand.swap_remove(pos);
                    }

                    // Add to pile
                    pile_hand.push(card0);
                    pile_hand.push(card1);

                    // These conditions are guaranteed
                    // if player_hand.len() <= 2 && pile_hand.len() <= 3 {
                    let mut temp = inferred_constraints.to_vec();
                    temp[player_loop] = player_hand;
                    temp[6] = pile_hand;
                    variants.push(temp);
                    // }
                }
            }
        }

        variants
    }
    pub fn gen_variants(card_types: &[Card], max_cards: usize) -> Vec<Vec<Card>> {
        let mut variants = Vec::new();
        // for each possible hand size 0..=max_cards
        for size in 0..=max_cards {
            // Generate all multisets of exactly `size` cards
            for combo in card_types.iter().combinations_with_replacement(size) {
                // sort and collect into Vec<Card>
                let mut hand = combo.into_iter().cloned().collect::<Vec<_>>();
                // ensure deterministic ordering
                hand.sort_by_key(|c| *c as u8);
                variants.push(hand);
            }
        }
        variants
    }
    pub fn clear_log(log_file_name: &str) -> std::io::Result<()> {
        // Open file with truncate flag to clear contents
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(log_file_name)?;
        Ok(())
    }
    // TODO: use a utility logger
    pub fn logger(log_file_name: &str, level: LevelFilter) {
        // Clear log file before initializing logger

        let _ = Self::clear_log(log_file_name);
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_name)
            .expect("Failed to open log file");

        Builder::from_env(Env::default().default_filter_or("info"))
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, level)
            .target(Target::Pipe(Box::new(log_file)))
            .init();
    }
    pub fn test_exchange_draw_public(log_file_name: &str) {
        Self::logger(log_file_name, LevelFilter::Info);
        let mut test_inferred_constraints: HashSet<Vec<Vec<Card>>> = HashSet::new();

        // Generate all possible combinations for player_loop (0-4 cards) and player 6 (0-3 cards)
        for player_hand in Self::gen_variants(
            &[
                Card::Ambassador,
                Card::Assassin,
                Card::Captain,
                Card::Duke,
                Card::Contessa,
            ],
            4,
        ) {
            'outer: for pile_hand in Self::gen_variants(
                &[
                    Card::Ambassador,
                    Card::Assassin,
                    Card::Captain,
                    Card::Duke,
                    Card::Contessa,
                ],
                3,
            ) {
                // Check that no card type exceeds 3 total across player and pile
                for card in [
                    Card::Ambassador,
                    Card::Assassin,
                    Card::Captain,
                    Card::Duke,
                    Card::Contessa,
                ] {
                    if player_hand.iter().filter(|c| **c == card).count()
                        + pile_hand.iter().filter(|c| **c == card).count()
                        > 3
                        || player_hand.len() + pile_hand.len() > 5
                        || pile_hand.len() > 1
                    {
                        continue 'outer;
                    }
                }
                test_inferred_constraints.insert(vec![
                    player_hand.clone(),
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    pile_hand.clone(),
                ]);
            }
        }

        log::info!(
            "Testing {} unique inferred_constraints configurations",
            test_inferred_constraints.len()
        );

        for item in test_inferred_constraints.iter() {
            let variants = Self::return_variants_exchange_draw_public(0, item);
            log::info!("dest: {:?}", item);
            log::info!("variants count: {}", variants.len());

            // Check for duplicate variants
            let mut seen_variants: HashSet<Vec<Vec<Card>>> = HashSet::new();
            for variant in variants.iter() {
                if !seen_variants.insert(variant.clone()) {
                    log::warn!("  FAILED: duplicate variant found: {:?}", variant);
                }
            }

            for variant in variants.iter() {
                log::info!("  src: {:?}", variant);
                // Verify constraints
                if variant[0].len() > 2 {
                    log::warn!("  FAILED: player_loop has > 4 cards in source");
                }
                if variant[6].len() > 3 {
                    log::warn!("  FAILED: pile has > 3 cards in source");
                }
                // Verify that after the move, player ends with at most 2 cards
                // (This is checking the reverse: source -> dest should result in dest having ≤2)
                if item[0].len() > 4 {
                    log::warn!("  FAILED: dest player_loop has > 4 cards");
                }
                if item[6].len() > 1 {
                    log::warn!("  FAILED: dest pile has > 1 cards");
                }
            }
        }
    }
    pub fn test_variant_recurse(log_file_name: &str) {
        Self::logger(log_file_name, LevelFilter::Info);
        let mut test_inferred_constraints: HashSet<Vec<Vec<Card>>> = HashSet::new();
        for player_hand in Self::gen_variants(&[Card::Ambassador, Card::Assassin], 2) {
            'outer: for pile_hand in
                Self::gen_variants(&[Card::Ambassador, Card::Assassin, Card::Captain], 3)
            {
                for card in [
                    Card::Ambassador,
                    Card::Assassin,
                    Card::Captain,
                    Card::Duke,
                    Card::Contessa,
                ] {
                    if player_hand.iter().filter(|c| **c == card).count()
                        + pile_hand.iter().filter(|c| **c == card).count()
                        > 3
                    {
                        continue 'outer;
                    }
                    test_inferred_constraints.insert(vec![
                        player_hand.clone(),
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        pile_hand.clone(),
                    ]);
                }
            }
        }
        // let mut symmetric_keys: HashSet<[u8; 10]> = HashSet::with_capacity(200);
        'outer: for item in test_inferred_constraints.iter() {
            for card_num in 0..5 {
                if item
                    .iter()
                    .map(|v| v.iter().filter(|c| **c as usize == card_num).count() as u8)
                    .sum::<u8>()
                    > 3
                {
                    continue 'outer;
                }
            }
            // let reveal = Self::return_variants_reveal_redraw_none_opt(Card::Ambassador, 0, item);
            // log::info!("src rr none: {:?}", reveal);
            let redraw =
                Self::return_variants_reveal_redraw(Card::Ambassador, Card::Assassin, 0, item);
            log::info!("src rr draw: {:?}", redraw);
            let redraw =
                Self::return_variants_reveal_redraw(Card::Ambassador, Card::Ambassador, 0, item);
            log::info!("src rr same: {:?}", redraw);
            // let relin = Self::return_variants_reveal_relinquish_opt(Card::Ambassador, 0, item);
            // log::info!("src rr rel: {:?}", relin);
            // let exchange_1 = Self::return_variants_exchange_opt(1, 0, item);
            // log::info!("src ex one: {:?}", exchange_1);
            // let exchange_2 = Self::return_variants_exchange_opt(2, 0, item);
            // log::info!("src ex two: {:?}", exchange_2);
            // WRite a test for all cases, generate some start state, go fwd, then go bwd, then check if bwd generated item includes start state

            // === START EXCHANGE CHOICE ===
            // let mut hand_count: [u8; 5] = [0; 5];
            // item[0].iter().for_each(|c| hand_count[*c as usize] += 1);
            // let mut pile_count: [u8; 5] = [0; 5];
            // item[6].iter().for_each(|c| pile_count[*c as usize] += 1);
            // let mut symmetric_key: [u8; 10] = [0; 10];
            // let mut temp = hand_count.clone();
            // temp.sort_unstable();
            // temp.iter().enumerate().for_each(|(i, c)| symmetric_key[i] = *c);
            // let mut temp = pile_count.clone();
            // temp.sort_unstable();
            // temp.iter().enumerate().for_each(|(i, c)| symmetric_key[i+5] = *c);
            // if symmetric_keys.contains(&symmetric_key) {
            //     // Avoiding symmetric cases
            //     continue 'outer;
            // }
            // symmetric_keys.insert(symmetric_key);
            // let mut hand_and_pile_count = hand_count.clone();
            // hand_and_pile_count.iter_mut().zip(pile_count.iter()).for_each(|(hp, p)| *hp += *p);
            // for i in 0..5 {
            //     for j in i..5 {
            //         for k in 0..5 {
            //             'inner: for l in k..5 {
            //                 let mut temp_count: [u8; 5] = hand_count.clone();
            //                 temp_count[i as usize] += 1;
            //                 temp_count[j as usize] += 1;
            //                 let mut relin_count: [u8; 5] = [0; 5];
            //                 relin_count[k as usize] += 1;
            //                 relin_count[l as usize] += 1;
            //                 let mut draw_count: [u8; 5] = [0; 5];
            //                 draw_count[i as usize] += 1;
            //                 draw_count[j as usize] += 1;
            //                 // Get union of all 3
            //                 let mut union_count_0: u8 = 0;
            //                 let mut union_count_1: u8 = 0;
            //                 for m in 0..5 {
            //                     // relinquish must be in outcome hand + draw
            //                     // not exact
            //                     // draw + hand + relin at most 4
            //                     union_count_0 += hand_count[m].max(draw_count[m]).max(relin_count[m]);
            //                     // draw + hand + relin + pile at most 5
            //                     let max_union_count = hand_and_pile_count[m].max(relin_count[m]).max(pile_count[m]);
            //                     // max cards at most 3
            //                     if max_union_count > 3 {
            //                         continue 'inner;
            //                     }
            //                     union_count_1 += max_union_count;
            //                 }
            //                 if union_count_0 > 4 || union_count_1 > 5{
            //                     continue 'inner;
            //                 }
            //                 // Enough space to have received the relinquish
            //                 let mut pile_space: u8 = 0;
            //                 pile_count.iter().zip(relin_count.iter()).for_each(|(p, r)| pile_space += *p.max(r));
            //                 if pile_space > 3 {
            //                     continue 'inner;
            //                 }
            //                 // Total amount in (draw union relinquish) and not in (hand + pile) cannot exceed (5 - hand.len() - pile.len())
            //                 let mut draw_union_relin_count: [u8; 5] = [0; 5];
            //                 let mut hand_plus_pile_count: [u8; 5] = [0; 5];
            //                 let mut degrees_of_freedom_used: u8 = 0;
            //                 draw_count.iter().zip(relin_count.iter()).enumerate().for_each(|(c, (d, r))| draw_union_relin_count[c] += *d.max(r));
            //                 hand_plus_pile_count.iter_mut().enumerate().for_each(|(i, count)| *count = hand_count[i] + pile_count[i]);
            //                 hand_plus_pile_count.iter().zip(draw_union_relin_count.iter()).for_each(|(h, d)| {
            //                     if *d > *h {
            //                         degrees_of_freedom_used += *d - *h;
            //                     }
            //                 });
            //                 if degrees_of_freedom_used + hand_plus_pile_count.iter().sum::<u8>() > 5{
            //                     continue 'inner;
            //                 }
            //                 // destination hand must be
            //                 let draw = vec![Card::try_from(i).unwrap(), Card::try_from(j).unwrap()];
            //                 let relinquish = vec![Card::try_from(k).unwrap(), Card::try_from(l).unwrap()];
            //                 let exchange_private = Self::return_variants_exchange_private_3(0, &draw, &relinquish,item);
            //                 log::info!("src ex_p draw: {:?}, relin: {:?} : {:?}", draw, relinquish, exchange_private);
            //                 // Length Check
            //                 for v in exchange_private.iter() {
            //                     if v[0].len() > 2 || v[6].len() > 3 {
            //                         log::warn!("FAILED ORIG LEN CHECK");
            //                         continue 'inner;
            //                     }
            //                 }
            //                 // Test if when all draw != relin, old hand == relinquish
            //                 // Test can be done via reversing it to verify if given solution is legit
            //             }
            //         }
            //     }
            // }
            // === END EXCHANGE CHOICE ===
            // if reveal.len() < relin.len() {
            //     log::warn!("reveal failed constraint check");
            // }
            // if reveal.len() < redraw.len() {
            //     log::warn!("reveal redraw failed constraint check");
            // }
            // if relin.len() < redraw.len() {
            //     log::warn!("redraw relin failed constraint check");
            // }
            log::info!("dest: {:?}", item);
        }
        // for item in test_inferred_constraints.iter() {
        //     let cc = PathDependentCollectiveConstraint::return_variants_reveal_redraw_none_opt(Card::Ambassador, 0, item);
        //     log::info!("src: {:?}", cc);
        //     log::info!("dest: {:?}", item);
        // }
        // for item in test_inferred_constaints.iter() {
        //     let cc = PathDependentCollectiveConstraint::return_variants_reveal_redraw(Card::Ambassador, Card::Captain, 0, item);
        //     log::info!("src: {:?}", cc);
        //     log::info!("dest: {:?}", item);
        // }
    }
}
