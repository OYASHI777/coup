use crate::history_public::Card;

/// A bitField that store impossibility boolean states for 3 card combinations
///
/// The main benefit of this is being able to use MASKS to check if a particular
/// card is completely impossible, without having to loop
#[derive(Copy, Clone, Debug)]
pub struct ImpossibleField3(u64);

impl ImpossibleField3 {
    pub const MASK_AMBASSADOR: u64 = Self::card_mask(Card::Ambassador);
    pub const MASK_ASSASSIN: u64 = Self::card_mask(Card::Assassin);
    pub const MASK_CAPTAIN: u64 = Self::card_mask(Card::Captain);
    pub const MASK_DUKE: u64 = Self::card_mask(Card::Duke);
    pub const MASK_CONTESSA: u64 = Self::card_mask(Card::Contessa);

    #[inline(always)]
    pub fn zero() -> Self {
        Self(0)
    }

    /// Collision-free index for unordered triples (i, j, k) with self-pairs allowed.
    #[inline(always)]
    pub const fn index(i: Card, j: Card, k: Card) -> u32 {
        let ai = i as u32;
        let aj = j as u32;
        let ak = k as u32;

        let (a, b) = if ai <= aj { (ai, aj) } else { (aj, ai) };
        let (b, c) = if b <= ak { (b, ak) } else { (ak, b) };
        let (a, b) = if a <= b { (a, b) } else { (b, a) };

        let b1 = a;
        let b2 = b + 1;
        let b3 = c + 2;

        Self::c3(b3) + Self::c2(b2) + b1
    }

    #[inline(always)]
    const fn c2(n: u32) -> u32 {
        n * (n - 1) / 2
    }
    #[inline(always)]
    const fn c3(n: u32) -> u32 {
        n * (n - 1) * (n - 2) / 4 
    }

    /// Generates a mask with all indices related to the card as true
    pub const fn card_mask(card: Card) -> u64 {
        1 << Self::index(card, Card::Ambassador, Card::Ambassador)
            | 1 << Self::index(card, Card::Ambassador, Card::Assassin)
            | 1 << Self::index(card, Card::Ambassador, Card::Captain)
            | 1 << Self::index(card, Card::Ambassador, Card::Duke)
            | 1 << Self::index(card, Card::Ambassador, Card::Contessa)
            | 1 << Self::index(card, Card::Assassin, Card::Assassin)
            | 1 << Self::index(card, Card::Assassin, Card::Captain)
            | 1 << Self::index(card, Card::Assassin, Card::Duke)
            | 1 << Self::index(card, Card::Assassin, Card::Contessa)
            | 1 << Self::index(card, Card::Captain, Card::Captain)
            | 1 << Self::index(card, Card::Captain, Card::Duke)
            | 1 << Self::index(card, Card::Captain, Card::Contessa)
            | 1 << Self::index(card, Card::Duke, Card::Duke)
            | 1 << Self::index(card, Card::Duke, Card::Contessa)
            | 1 << Self::index(card, Card::Contessa, Card::Contessa)
    }

    /// Sets the impossibility state of a particular 3 card combination
    #[inline(always)]
    pub fn set(&mut self, i: Card, j: Card, k: Card, impossibility: bool) {
        let idx = Self::index(i, j, k);
        let mask = 1u64 << idx;
        let bit = (impossibility as u64) << idx;
        self.0 = (self.0 & !mask) | bit;
    }

    /// Gets the impossibility state of a particular 3 card combination
    #[inline(always)]
    pub fn get(&self, i: Card, j: Card, k: Card) -> bool {
        ((self.0 >> Self::index(i, j, k)) & 1) == 1
    }

    /// (Optional helper) Return the raw storage (useful for tests/bit ops).
    #[inline(always)]
    pub fn bits(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARDS: [Card; 5] = [
        Card::Ambassador,
        Card::Assassin,
        Card::Captain,
        Card::Duke,
        Card::Contessa,
    ];

    #[test]
    fn set_get_roundtrip() {
        // Test every ordered triple (125) for robustness.
        for &i in &CARDS {
            for &j in &CARDS {
                for &k in &CARDS {
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
        for &i in &CARDS {
            for &j in &CARDS {
                for &k in &CARDS {
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
        for &i in &CARDS {
            let mut f = ImpossibleField3::zero();
            for &j in &CARDS {
                for &k in &CARDS {
                    f.set(i, j, k, true);
                }
            }

            let mask = match i {
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
        for &i in &CARDS {
            for &j in &CARDS {
                for &k in &CARDS {
                    let mut f = ImpossibleField3::zero();
                    f.set(i, j, k, true);

                    for &a in &CARDS {
                        for &b in &CARDS {
                            for &c in &CARDS {
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
        for i in &CARDS {
            for j in &CARDS {
                for k in &CARDS {
                    let mut ijk = vec![i, j, k];
                    ijk.sort_unstable();
                    for l in &CARDS {
                        for m in &CARDS {
                            for n in &CARDS {
                                let mut lmn = vec![l, m, n];
                                lmn.sort_unstable();
                                if ijk == lmn {
                                    continue;
                                }
                                assert!(
                                    ImpossibleField3::index(*i, *j, *k)
                                        != ImpossibleField3::index(*l, *m, *n)
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
