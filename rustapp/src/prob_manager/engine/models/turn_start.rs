use crate::prob_manager::engine::fsm_engine::Node;
pub struct TurnStart;

impl Node for TurnStart {
    fn dispatch(&self) -> bool {
        false
    }
}