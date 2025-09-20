use std::fmt::Debug;

use crate::{history_public::Card, prob_manager::engine::constants::MAX_CARD_PERMS_ONE};

/// A bitField that store impossibility boolean states for 1 card combinations
///
/// The main benefit of this is being able to use 8 bits instead of 40 bits
#[derive(Copy, Clone)]
pub struct ImpossibleField1(u8);

impl ImpossibleField1 {
    pub const MASK_AMBASSADOR: u8 = Self::card_mask(Card::Ambassador);
    pub const MASK_ASSASSIN: u8 = Self::card_mask(Card::Assassin);
    pub const MASK_CAPTAIN: u8 = Self::card_mask(Card::Captain);
    pub const MASK_DUKE: u8 = Self::card_mask(Card::Duke);
    pub const MASK_CONTESSA: u8 = Self::card_mask(Card::Contessa);

    /// Initializes to all possible (none impossible).
    #[inline]
    pub fn zero() -> Self {
        Self(0)
    }

    /// Collision-free index
    #[inline]
    pub const fn index(i: u8) -> u8 {
        debug_assert!(i < MAX_CARD_PERMS_ONE as u8);
        i
    }

    /// Mask for a specific `Card`.
    pub const fn card_mask(card: Card) -> u8 {
        1 << Self::index(card as u8)
    }

    /// Sets the impossibility state for a single card
    #[inline]
    pub fn set(&mut self, i: u8, impossibility: bool) {
        debug_assert!(i < MAX_CARD_PERMS_ONE as u8);
        let index = Self::index(i);
        let mask = 1 << index;
        let bit = (impossibility as u8) << index;
        self.0 = (self.0 & !mask) | bit;
    }

    /// Gets the impossibility state for a single card
    #[inline]
    pub fn get(&self, i: u8) -> bool {
        debug_assert!(i < MAX_CARD_PERMS_ONE as u8);
        (self.0 >> Self::index(i)) & 1 == 1
    }
}

impl Debug for ImpossibleField1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for i in 0..MAX_CARD_PERMS_ONE as u8 {
            map.entry(
                &format_args!("{:?}", Card::try_from(i).unwrap()),
                &self.get(i),
            );
        }
        map.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip() {
        for i in 0..MAX_CARD_PERMS_ONE as u8 {
            let mut f = ImpossibleField1::zero();

            f.set(i, true);
            assert!(f.get(i));

            f.set(i, false);
            assert!(!f.get(i));
        }
    }

    #[test]
    fn mask_set_correspondence() {
        for i in 0..MAX_CARD_PERMS_ONE as u8 {
            let mut f = ImpossibleField1::zero();
            f.set(i, true);

            let mask = match Card::try_from(i).unwrap() {
                Card::Ambassador => ImpossibleField1::MASK_AMBASSADOR,
                Card::Assassin => ImpossibleField1::MASK_ASSASSIN,
                Card::Captain => ImpossibleField1::MASK_CAPTAIN,
                Card::Duke => ImpossibleField1::MASK_DUKE,
                Card::Contessa => ImpossibleField1::MASK_CONTESSA,
            };

            assert!(f.0 == mask);
        }
    }

    #[test]
    fn no_collision() {
        for i in 0..MAX_CARD_PERMS_ONE as u8 {
            for j in 0..MAX_CARD_PERMS_ONE as u8 {
                if i == j {
                    continue;
                }
                assert!(ImpossibleField1::index(i) != ImpossibleField1::index(j));
            }
        }
    }
}
