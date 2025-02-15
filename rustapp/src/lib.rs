pub mod prob_manager {
    pub mod permutation_generator;
    pub mod prob_state_old;
    pub mod naive_prob;
    pub mod constraint;
    pub mod loader;
    mod coup_const;
    pub mod bit_prob;
    pub mod brute_prob;
    pub mod brute_prob_generic;
    pub mod compressed_group_constraint;
    pub mod collective_constraint;
    pub mod naive_sampler;
}
pub mod history_public;
pub mod string_utils;
pub mod pcmccfr;
pub mod cfr {
    pub mod explorer;
    mod best_response_policy;
    pub mod keys;
    mod mixed_strategy_policy;
    mod policy_handler;
    mod action_serialiser;
    mod value_function;
    mod inference_buffer;
    pub mod reach_prob;
}