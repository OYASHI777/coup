use std::fmt::Debug;

use crate::history_public::Card;

/// A bitField that store impossibility boolean states for 3 card combinations
///
/// The main benefit of this is being able to use MASKS to check if a particular
/// card is completely impossible, without having to loop
#[derive(Copy, Clone)]
pub struct ImpossibleField3(u64);

impl ImpossibleField3 {
    pub const MASK_AMBASSADOR: u64 = Self::card_mask(Card::Ambassador);
    pub const MASK_ASSASSIN: u64 = Self::card_mask(Card::Assassin);
    pub const MASK_CAPTAIN: u64 = Self::card_mask(Card::Captain);
    pub const MASK_DUKE: u64 = Self::card_mask(Card::Duke);
    pub const MASK_CONTESSA: u64 = Self::card_mask(Card::Contessa);

    /// Initialises to all possible (None impossible)
    #[inline]
    pub fn zero() -> Self {
        Self(0)
    }

    /// Collision-free index for unordered triples (i, j, k) with self-pairs allowed.
    #[inline]
    pub const fn index(i: u8, j: u8, k: u8) -> u8 {
        debug_assert!(i < 5);
        debug_assert!(j < 5);
        debug_assert!(k < 5);
        let (a, b) = if i <= j { (i, j) } else { (j, i) };
        let (b, c) = if b <= k { (b, k) } else { (k, b) };
        let (a, b) = if a <= b { (a, b) } else { (b, a) };

        let b1 = a;
        let b2 = b + 1;
        let b3 = c + 2;

        Self::c3(b3) + Self::c2(b2) + b1
    }

    const fn c2(n: u8) -> u8 {
        n * (n - 1) / 2
    }
    const fn c3(n: u8) -> u8 {
        n * (n - 1) * (n - 2) / 4
    }

    /// Generates a mask with all indices related to the card as true
    pub const fn card_mask(card: Card) -> u64 {
        let card_i = card as u8;
        1 << Self::index(card_i, Card::Ambassador as u8, Card::Ambassador as u8)
            | 1 << Self::index(card_i, Card::Ambassador as u8, Card::Assassin as u8)
            | 1 << Self::index(card_i, Card::Ambassador as u8, Card::Captain as u8)
            | 1 << Self::index(card_i, Card::Ambassador as u8, Card::Duke as u8)
            | 1 << Self::index(card_i, Card::Ambassador as u8, Card::Contessa as u8)
            | 1 << Self::index(card_i, Card::Assassin as u8, Card::Assassin as u8)
            | 1 << Self::index(card_i, Card::Assassin as u8, Card::Captain as u8)
            | 1 << Self::index(card_i, Card::Assassin as u8, Card::Duke as u8)
            | 1 << Self::index(card_i, Card::Assassin as u8, Card::Contessa as u8)
            | 1 << Self::index(card_i, Card::Captain as u8, Card::Captain as u8)
            | 1 << Self::index(card_i, Card::Captain as u8, Card::Duke as u8)
            | 1 << Self::index(card_i, Card::Captain as u8, Card::Contessa as u8)
            | 1 << Self::index(card_i, Card::Duke as u8, Card::Duke as u8)
            | 1 << Self::index(card_i, Card::Duke as u8, Card::Contessa as u8)
            | 1 << Self::index(card_i, Card::Contessa as u8, Card::Contessa as u8)
    }

    /// Sets the impossibility state of a particular 3 card combination
    #[inline]
    pub fn set(&mut self, i: u8, j: u8, k: u8, impossibility: bool) {
        debug_assert!(i < 5);
        debug_assert!(j < 5);
        debug_assert!(k < 5);
        let idx = Self::index(i, j, k);
        let mask = 1u64 << idx;
        let bit = (impossibility as u64) << idx;
        self.0 = (self.0 & !mask) | bit;
    }

    /// Gets the impossibility state of a particular 3 card combination
    #[inline]
    pub fn get(&self, i: u8, j: u8, k: u8) -> bool {
        debug_assert!(i < 5);
        debug_assert!(j < 5);
        debug_assert!(k < 5);
        ((self.0 >> Self::index(i, j, k)) & 1) == 1
    }
}

impl Debug for ImpossibleField3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for i in 0..5 {
            for j in i..5 {
                for k in j..5 {
                    map.entry(
                        &format_args!(
                            "({:?}, {:?}, {:?})",
                            Card::try_from(i).unwrap(),
                            Card::try_from(j).unwrap(),
                            Card::try_from(k).unwrap()
                        ),
                        &self.get(i, j, k),
                    );
                }
            }
        }
        map.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip() {
        // Test every ordered triple (125) for robustness.
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..5 {
                    let mut f = ImpossibleField3::zero();

                    f.set(i, j, k, true);
                    assert!(f.get(i, j, k));

                    f.set(i, j, k, false);
                    assert!(!f.get(i, j, k));
                }
            }
        }
    }

    #[test]
    fn unordered() {
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..5 {
                    let mut f = ImpossibleField3::zero();

                    f.set(i, j, k, true);
                    assert!(f.get(i, j, k));
                    assert!(f.get(i, k, j));
                    assert!(f.get(j, i, k));
                    assert!(f.get(j, k, i));
                    assert!(f.get(k, i, j));
                    assert!(f.get(k, j, i));
                }
            }
        }
    }

    #[test]
    fn mask_set_correspondence() {
        for i in 0..5 {
            let mut f = ImpossibleField3::zero();
            for j in 0..5 {
                for k in 0..5 {
                    f.set(i, j, k, true);
                }
            }

            let mask = match Card::try_from(i).unwrap() {
                Card::Ambassador => ImpossibleField3::MASK_AMBASSADOR,
                Card::Assassin => ImpossibleField3::MASK_ASSASSIN,
                Card::Captain => ImpossibleField3::MASK_CAPTAIN,
                Card::Duke => ImpossibleField3::MASK_DUKE,
                Card::Contessa => ImpossibleField3::MASK_CONTESSA,
            };

            assert!(f.0 == mask);
        }
    }

    #[test]
    fn setting_one_does_not_affect_others() {
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..5 {
                    let mut f = ImpossibleField3::zero();
                    f.set(i, j, k, true);

                    for a in 0..5 {
                        for b in 0..5 {
                            for c in 0..5 {
                                let same = (a == i && b == j && c == k)
                                    || (a == i && b == k && c == j)
                                    || (a == j && b == i && c == k)
                                    || (a == j && b == k && c == i)
                                    || (a == k && b == i && c == j)
                                    || (a == k && b == j && c == i);
                                assert_eq!(f.get(a, b, c), same);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn no_collision() {
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..5 {
                    let mut ijk = vec![i, j, k];
                    ijk.sort_unstable();
                    for l in 0..5 {
                        for m in 0..5 {
                            for n in 0..5 {
                                let mut lmn = vec![l, m, n];
                                lmn.sort_unstable();
                                if ijk == lmn {
                                    continue;
                                }
                                assert!(
                                    ImpossibleField3::index(i, j, k)
                                        != ImpossibleField3::index(l, m, n)
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
