pub mod traits {
    pub mod prob_manager {
        pub mod card_state;
        pub mod coup_analysis;
    }
}
// TEMP REMOVED TO NOT SEE WARNINGS
// pub mod cfr {
//     pub mod explorer;
//     mod best_response_policy;
//     pub mod keys;
//     mod mixed_strategy_policy;
//     mod policy_handler;
//     mod action_serialiser;
//     mod value_function;
//     mod inference_buffer;
//     pub mod reach_prob;
// }
pub mod prob_manager {
    pub mod engine {
        pub mod models {
            pub mod assassinate;
            pub mod coup;
            pub mod end;
            pub mod engine_state;
            pub mod exchange;
            pub mod foreign_aid;
            pub mod game_state;
            pub mod steal;
            pub mod tax;
            pub mod turn_start;
        }
        pub mod models_prelude {
            pub use super::models::{
                assassinate::*,
                coup::*,
                end::*,
                exchange::*,
                foreign_aid::*,
                steal::*,
                tax::*,
                turn_start::*,
            };
        }
        pub mod constants;
        pub mod fsm_engine;
    }
    pub mod models {
        pub mod backtrack;
        pub mod card_state_u64;
    }
    pub mod tools {
        pub mod recursion_fn;
    }
    pub mod tracker {
        pub mod collater;
        pub mod informed_tracker;
        pub mod uninformed_tracker;
    }
    pub mod utils {
        pub mod permutation_generator;
    }
    // TO BE DEPRECATED
    // pub mod backtracking_prob;
    pub mod backtracking_prob_hybrid;
    // TO BE DEPRECATED
    // pub mod backtracking_collective_constraints;
    pub mod brute_prob_generic;
    // TO BE DEPRECATED
    // pub mod constraint;
    mod constants;
    pub mod move_guard;
    // TEMP REMOVED TO NOT SEE WARNINGS
    // pub mod naive_prob;
    // pub mod naive_sampler;
}
pub mod history_public;
pub mod string_utils;
pub mod pcmccfr;