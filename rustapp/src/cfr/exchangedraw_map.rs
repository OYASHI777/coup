// Reach prob
// 6 players so
// 6 hashmap <infostate, bool>
// 6 vec for infostates with true values
// 6 vec for infostates false values
use super::keys::{Infostate, MSKey, INFOSTATES};
use super::keys::Infostate::*;
// use rayon::iter::IntoParallelIterator; 
use rayon::prelude::*;
use crossbeam::thread;
use std::collections::hash_map::Keys;
use std::time::Instant;
use ahash::AHashMap;
use super::mixed_strategy_policy::MSInterface;
use crate::history_public::{Card, ActionObservation};


#[derive(Clone)]
pub struct ReachProb {
    // TODO: Make AHashMap an Intmap<u8, bool> with nohashhasher, the interface takes Infostate but converts it under the hood as u8
    reach_probs_player0: AHashMap<Infostate, bool>,
    reach_probs_player1: AHashMap<Infostate, bool>,
    reach_probs_player2: AHashMap<Infostate, bool>,
    reach_probs_player3: AHashMap<Infostate, bool>,
    reach_probs_player4: AHashMap<Infostate, bool>,
    reach_probs_player5: AHashMap<Infostate, bool>,
    
    true_infostates_player0: Vec<Infostate>,
    true_infostates_player1: Vec<Infostate>,
    true_infostates_player2: Vec<Infostate>,
    true_infostates_player3: Vec<Infostate>,
    true_infostates_player4: Vec<Infostate>,
    true_infostates_player5: Vec<Infostate>,
    
    false_infostates_player0: Vec<Infostate>,
    false_infostates_player1: Vec<Infostate>,
    false_infostates_player2: Vec<Infostate>,
    false_infostates_player3: Vec<Infostate>,
    false_infostates_player4: Vec<Infostate>,
    false_infostates_player5: Vec<Infostate>,

    len: usize,
}

impl ReachProb {
    // Initialising to all true
    // cloning
    // TODO: Write restrictors for Discard and 
    pub fn new() -> Self{
        let mut starter: AHashMap<Infostate, bool> = AHashMap::with_capacity(15);
        for infostates in [AA, AB, AC, AD, AE, BB, BC, BD, BE, CC, CD, CE, DD, DE, EE].iter() {
            starter.insert(*infostates, true);
        }
        let total_infostates: usize = 15;
        let p0: [Infostate; 15] = [AB, AC, AD, AE, BC, BD, BE, CD, CE, DE, AA, EE, CC, DD, BB];
        let p1: [Infostate; 15] = [CD, BC, BD, AC, AD, CE, DE, BE, AE, AB, DD, CC, AA, BB, EE];
        let p2: [Infostate; 15] = [BE, AE, AD, DE, CE, CD, BC, BD, AB, AC, BB, AA, DD, CC, EE];
        let p3: [Infostate; 15] = [CE, CD, AC, AD, DE, AE, BC, BD, BE, AB, EE, AA, CC, BB, DD];
        let p4: [Infostate; 15] = [BD, CD, AC, AD, BE, CE, DE, AB, AE, BC, AA, CC, EE, DD, BB];
        let p5: [Infostate; 15] = [AE, DE, AB, CD, AC, BE, AD, CE, BD, BC, CC, BB, AA, DD, EE];
        ReachProb{
            reach_probs_player0: starter.clone(),
            reach_probs_player1: starter.clone(),
            reach_probs_player2: starter.clone(),
            reach_probs_player3: starter.clone(),
            reach_probs_player4: starter.clone(),
            reach_probs_player5: starter.clone(),
            true_infostates_player0: p0.to_vec(),
            true_infostates_player1: p1.to_vec(),
            true_infostates_player2: p2.to_vec(),
            true_infostates_player3: p3.to_vec(),
            true_infostates_player4: p4.to_vec(),
            true_infostates_player5: p5.to_vec(),
            false_infostates_player0: Vec::with_capacity(total_infostates),
            false_infostates_player1: Vec::with_capacity(total_infostates),
            false_infostates_player2: Vec::with_capacity(total_infostates),
            false_infostates_player3: Vec::with_capacity(total_infostates),
            false_infostates_player4: Vec::with_capacity(total_infostates),
            false_infostates_player5: Vec::with_capacity(total_infostates),
            len: 90,
        }
    }
    pub fn new_empty() -> Self {
        let starter: AHashMap<Infostate, bool> = AHashMap::with_capacity(15);
        let total_infostates: usize = 15;
        ReachProb{
            reach_probs_player0: starter.clone(),
            reach_probs_player1: starter.clone(),
            reach_probs_player2: starter.clone(),
            reach_probs_player3: starter.clone(),
            reach_probs_player4: starter.clone(),
            reach_probs_player5: starter.clone(),
            true_infostates_player0: Vec::with_capacity(total_infostates),
            true_infostates_player1: Vec::with_capacity(total_infostates),
            true_infostates_player2: Vec::with_capacity(total_infostates),
            true_infostates_player3: Vec::with_capacity(total_infostates),
            true_infostates_player4: Vec::with_capacity(total_infostates),
            true_infostates_player5: Vec::with_capacity(total_infostates),
            false_infostates_player0: Vec::with_capacity(total_infostates),
            false_infostates_player1: Vec::with_capacity(total_infostates),
            false_infostates_player2: Vec::with_capacity(total_infostates),
            false_infostates_player3: Vec::with_capacity(total_infostates),
            false_infostates_player4: Vec::with_capacity(total_infostates),
            false_infostates_player5: Vec::with_capacity(total_infostates),
            len: 0,
        }
    }
    pub fn get () {

    }
    pub fn get_mut() {

    }
    pub fn keys() {

    }
    pub fn remove(&mut self, player_id: u8, infostate: &Infostate) {
        // TODO: fill
        
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn log_state(&self) {
        log::trace!("HashMap State");
        log::trace!("P0: {:?}", self.reach_probs_player0);
        log::trace!("P1: {:?}", self.reach_probs_player1);
        log::trace!("P2: {:?}", self.reach_probs_player2);
        log::trace!("P3: {:?}", self.reach_probs_player3);
        log::trace!("P4: {:?}", self.reach_probs_player4);
        log::trace!("P5: {:?}", self.reach_probs_player5);
        log::trace!("True Infostates");
        log::trace!("P0: {:?}", self.true_infostates_player0);
        log::trace!("P1: {:?}", self.true_infostates_player1);
        log::trace!("P2: {:?}", self.true_infostates_player2);
        log::trace!("P3: {:?}", self.true_infostates_player3);
        log::trace!("P4: {:?}", self.true_infostates_player4);
        log::trace!("P5: {:?}", self.true_infostates_player5);
        log::trace!("False Infostates");
        log::trace!("P0: {:?}", self.false_infostates_player0);
        log::trace!("P1: {:?}", self.false_infostates_player1);
        log::trace!("P2: {:?}", self.false_infostates_player2);
        log::trace!("P3: {:?}", self.false_infostates_player3);
        log::trace!("P4: {:?}", self.false_infostates_player4);
        log::trace!("P5: {:?}", self.false_infostates_player5);
    }
    pub fn print_state(&self) {
        println!("HashMap State");
        println!("P0: {:?}", self.reach_probs_player0);
        println!("P1: {:?}", self.reach_probs_player1);
        println!("P2: {:?}", self.reach_probs_player2);
        println!("P3: {:?}", self.reach_probs_player3);
        println!("P4: {:?}", self.reach_probs_player4);
        println!("P5: {:?}", self.reach_probs_player5);
        println!("True Infostates");
        println!("P0: {:?}", self.true_infostates_player0);
        println!("P1: {:?}", self.true_infostates_player1);
        println!("P2: {:?}", self.true_infostates_player2);
        println!("P3: {:?}", self.true_infostates_player3);
        println!("P4: {:?}", self.true_infostates_player4);
        println!("P5: {:?}", self.true_infostates_player5);
        println!("False Infostates");
        println!("P0: {:?}", self.false_infostates_player0);
        println!("P1: {:?}", self.false_infostates_player1);
        println!("P2: {:?}", self.false_infostates_player2);
        println!("P3: {:?}", self.false_infostates_player3);
        println!("P4: {:?}", self.false_infostates_player4);
        println!("P5: {:?}", self.false_infostates_player5);
    }
    
}