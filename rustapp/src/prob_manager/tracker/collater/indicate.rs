use super::Collator;

pub struct Indicate;

impl Collator for Indicate {
    fn challenge(player: usize) -> Vec<crate::history_public::ActionObservation> {
        todo!()
    }
}