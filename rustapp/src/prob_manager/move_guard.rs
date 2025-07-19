use crate::history_public::Card;

/// This aids in resetting inferred constraint after card swaps between players
pub struct MoveGuard;

impl MoveGuard {
    #[inline(always)]
    pub fn swap_run_reset(inferred_constraint: &mut Vec<Vec<Card>>, player_a: usize, player_b: usize, a_to_b: &[Card], b_to_a: &[Card], f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool) -> bool {
        Self::swap_run_swap_back(inferred_constraint, player_a, player_b, a_to_b, b_to_a, f)
    }
    #[inline(always)]
    pub fn swap_run_clone_back(inferred_constraint: &mut Vec<Vec<Card>>, player_a: usize, player_b: usize, a_to_b: &[Card], b_to_a: &[Card], f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool) -> bool {
        let backup_player_a = inferred_constraint[player_a].clone();
        let backup_player_b = inferred_constraint[player_b].clone();
        for c in a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_a].iter().rposition(|card| card == c) {
                let card = inferred_constraint[player_a].swap_remove(pos);
                inferred_constraint[player_b].push(card);
            }
        }
        for c in b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_b].iter().rposition(|card| card == c) {
                let card = inferred_constraint[player_b].swap_remove(pos);
                inferred_constraint[player_a].push(card);
            }
        }
        if !f(inferred_constraint) {
            inferred_constraint[player_a] = backup_player_a;
            inferred_constraint[player_b] = backup_player_b; 
            return false
        }
        true
    }
    #[inline(always)]
    pub fn swap_run_swap_back(inferred_constraint: &mut Vec<Vec<Card>>, player_a: usize, player_b: usize, a_to_b: &[Card], b_to_a: &[Card], f: impl FnOnce(&mut Vec<Vec<Card>>) -> bool) -> bool {
        let mut moved_from_a_to_b: Vec<Card> = Vec::with_capacity(a_to_b.len());
        let mut moved_from_b_to_a: Vec<Card> = Vec::with_capacity(b_to_a.len());
        for c in a_to_b.iter() {
            if let Some(pos) = inferred_constraint[player_a].iter().rposition(|card| card == c) {
                let card = inferred_constraint[player_a].swap_remove(pos);
                moved_from_a_to_b.push(card);
            }
        }
        for c in b_to_a.iter() {
            if let Some(pos) = inferred_constraint[player_b].iter().rposition(|card| card == c) {
                let card = inferred_constraint[player_b].swap_remove(pos);
                moved_from_b_to_a.push(card);
            }
        }
        inferred_constraint[player_b].extend(moved_from_a_to_b.iter());
        inferred_constraint[player_a].extend(moved_from_b_to_a.iter());
        if !f(inferred_constraint) {
            for c in moved_from_b_to_a.iter() {
                if let Some(pos) = inferred_constraint[player_a].iter().rposition(|card| card == c) {
                    inferred_constraint[player_a].swap_remove(pos);
                }
            }
            for c in moved_from_a_to_b.iter() {
                if let Some(pos) = inferred_constraint[player_b].iter().rposition(|card| card == c) {
                    inferred_constraint[player_b].swap_remove(pos);
                }
            }
            inferred_constraint[player_b].extend(moved_from_b_to_a.iter());
            inferred_constraint[player_a].extend(moved_from_a_to_b.iter());
            return false
        }
        true
    }
}