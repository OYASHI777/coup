use crate::history_public::Card;

/// A bitField that store impossibility boolean states for 2 card combinations
///
/// The main benefit of this is being able to use MASKS to check if a particular
/// card is completely impossible, without having to loop
#[derive(Copy, Clone, Debug)]
pub struct ImpossibleField2(u32);
//TODO: Print a better debug representation
impl ImpossibleField2 {
    pub const MASK_AMBASSADOR: u32 = Self::card_mask(Card::Ambassador);
    pub const MASK_ASSASSIN: u32 = Self::card_mask(Card::Assassin);
    pub const MASK_CAPTAIN: u32 = Self::card_mask(Card::Captain);
    pub const MASK_DUKE: u32 = Self::card_mask(Card::Duke);
    pub const MASK_CONTESSA: u32 = Self::card_mask(Card::Contessa);

    /// Initialises to all possible (None impossible)
    pub fn zero() -> Self {
        Self(0)
    }

    /// Collision-free index for unordered pairs (i, j) with self-pairs allowed.
    pub const fn index(i: u8, j: u8) -> u8 {
        let ai = i + 1;
        let aj = j + 1;
        let p = ai * aj;

        // Case where (2, 2) which collides with (1, 4)
        // We assign 7 which is unused
        if ai == 2 && aj == 2 {
            return 7;
        }
        p
    }

    /// Generates a mask with all indices related to the card as true
    pub const fn card_mask(card: Card) -> u32 {
        let card_i = card as u8;
        1 << ImpossibleField2::index(card_i, Card::Ambassador as u8)
            | 1 << ImpossibleField2::index(card_i, Card::Assassin as u8)
            | 1 << ImpossibleField2::index(card_i, Card::Captain as u8)
            | 1 << ImpossibleField2::index(card_i, Card::Duke as u8)
            | 1 << ImpossibleField2::index(card_i, Card::Contessa as u8)
    }

    /// Sets the impossibility state of a particular 2 card combination
    pub fn set(&mut self, i: u8, j: u8, impossibility: bool) {
        let index = Self::index(i, j);
        let mask = 1 << index;
        let bit = (impossibility as u32) << index;
        self.0 = (self.0 & !mask) | bit;
    }
    
    /// Gets the impossibility state of a particular 2 card combination
    pub fn get(&self, i: u8, j: u8) -> bool {
        (self.0 >> Self::index(i, j)) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip() {
        for i in 0..5 {
            for j in 0..5 {
                let mut f = ImpossibleField2::zero();

                f.set(i, j, true);
                assert!(f.get(i, j));

                f.set(i, j, false);
                assert!(!f.get(i, j));
            }
        }
    }

    #[test]
    fn unordered() {
        for i in 0..5 {
            for j in 0..5 {
                let mut f = ImpossibleField2::zero();

                f.set(i, j, true);
                assert!(f.get(i, j));
                assert!(f.get(j, i));
            }
        }
    }

    #[test]
    fn mask_set_correspondence() {
        for i in 0..5 {
            let mut f = ImpossibleField2::zero();
            for j in 0..5 {
                f.set(i, j, true);
            }

            let mask = match Card::try_from(i).unwrap() {
                Card::Ambassador => ImpossibleField2::MASK_AMBASSADOR,
                Card::Assassin => ImpossibleField2::MASK_ASSASSIN,
                Card::Captain => ImpossibleField2::MASK_CAPTAIN,
                Card::Duke => ImpossibleField2::MASK_DUKE,
                Card::Contessa => ImpossibleField2::MASK_CONTESSA,
            };

            assert!(f.0 == mask);
        }
    }

    #[test]
    fn no_collision() {
        for i in 0..5 {
            for j in 0..5 {
                let mut ij = vec![i, j];
                ij.sort_unstable();
                for k in 0..5 {
                    for l in 0..5 {
                        let mut kl = vec![k, l];
                        kl.sort_unstable();
                        if ij == kl {
                            continue;
                        }
                        assert!(ImpossibleField2::index(i, j) != ImpossibleField2::index(k, l))
                    }
                }
            }
        }
    }
}
