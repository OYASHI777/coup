use super::Collator;

pub struct Unique;

impl Collator for Unique {
    fn challenge(player: usize) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}