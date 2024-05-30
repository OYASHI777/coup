// Reach prob
// 6 players so
// 6 hashmap <infostate, bool>
// 6 vec for infostates with true values
// 6 vec for infostates false values
use super::keys::INFOSTATES;
// use rayon::iter::IntoParallelIterator; 
use rayon::prelude::*;
use crossbeam::thread;
use std::collections::HashMap;

pub struct ReachProb {
    reach_probs_player0: HashMap<String, bool>,
    reach_probs_player1: HashMap<String, bool>,
    reach_probs_player2: HashMap<String, bool>,
    reach_probs_player3: HashMap<String, bool>,
    reach_probs_player4: HashMap<String, bool>,
    reach_probs_player5: HashMap<String, bool>,
    
    true_infostates_player0: Vec<String>,
    true_infostates_player1: Vec<String>,
    true_infostates_player2: Vec<String>,
    true_infostates_player3: Vec<String>,
    true_infostates_player4: Vec<String>,
    true_infostates_player5: Vec<String>,
    
    false_infostates_player0: Vec<String>,
    false_infostates_player1: Vec<String>,
    false_infostates_player2: Vec<String>,
    false_infostates_player3: Vec<String>,
    false_infostates_player4: Vec<String>,
    false_infostates_player5: Vec<String>,

}

impl ReachProb {
    // Initialising to all true
    // cloning
    // TODO: Write getters
    pub fn new() -> Self{
        let mut starter: HashMap<String, bool> = HashMap::with_capacity(15);
        for infostates in INFOSTATES.iter() {
            starter.insert(infostates.to_string(), true);
        }
        let total_infostates: usize = 15;
        let p0: [&str; 15] = ["AB", "AC", "AD", "AE", "BC", "BD", "BE", "CD", "CE", "DE", "AA", "EE", "CC", "DD", "BB"];
        let p1: [&str; 15] = ["CD", "BC", "BD", "AC", "AD", "CE", "DE", "BE", "AE", "AB", "DD", "CC", "AA", "BB", "EE"];
        let p2: [&str; 15] = ["BE", "AE", "AD", "DE", "CE", "CD", "BC", "BD", "AB", "AC", "BB", "AA", "DD", "CC", "EE"];
        let p3: [&str; 15] = ["CE", "CD", "AC", "AD", "DE", "AE", "BC", "BD", "BE", "AB", "EE", "AA", "CC", "BB", "DD"];
        let p4: [&str; 15] = ["BD", "CD", "AC", "AD", "BE", "CE", "DE", "AB", "AE", "BC", "AA", "CC", "EE", "DD", "BB"];
        let p5: [&str; 15] = ["AE", "DE", "AB", "CD", "AC", "BE", "AD", "CE", "BD", "BC", "CC", "BB", "AA", "DD", "EE"];
        ReachProb{
            reach_probs_player0: starter.clone(),
            reach_probs_player1: starter.clone(),
            reach_probs_player2: starter.clone(),
            reach_probs_player3: starter.clone(),
            reach_probs_player4: starter.clone(),
            reach_probs_player5: starter.clone(),
            true_infostates_player0: p0.iter().map(|&s| s.to_owned()).collect(),
            true_infostates_player1: p1.iter().map(|&s| s.to_owned()).collect(),
            true_infostates_player2: p2.iter().map(|&s| s.to_owned()).collect(),
            true_infostates_player3: p3.iter().map(|&s| s.to_owned()).collect(),
            true_infostates_player4: p4.iter().map(|&s| s.to_owned()).collect(),
            true_infostates_player5: p5.iter().map(|&s| s.to_owned()).collect(),
            false_infostates_player0: Vec::with_capacity(total_infostates),
            false_infostates_player1: Vec::with_capacity(total_infostates),
            false_infostates_player2: Vec::with_capacity(total_infostates),
            false_infostates_player3: Vec::with_capacity(total_infostates),
            false_infostates_player4: Vec::with_capacity(total_infostates),
            false_infostates_player5: Vec::with_capacity(total_infostates),
        }
    }
    pub fn set_status(&mut self, player_id: usize, infostate: &str, status: bool) {
        match player_id {
            0 => {
                if let Some(value) = self.reach_probs_player0.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player0.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player0.swap_remove(pos);
                                self.true_infostates_player0.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player0.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player0.swap_remove(pos);
                                self.false_infostates_player0.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            1 => {
                if let Some(value) = self.reach_probs_player1.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player1.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player1.swap_remove(pos);
                                self.true_infostates_player1.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player1.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player1.swap_remove(pos);
                                self.false_infostates_player1.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            2 => {
                if let Some(value) = self.reach_probs_player2.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player2.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player2.swap_remove(pos);
                                self.true_infostates_player2.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player2.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player2.swap_remove(pos);
                                self.false_infostates_player2.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            3 => {
                if let Some(value) = self.reach_probs_player3.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player3.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player3.swap_remove(pos);
                                self.true_infostates_player3.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player3.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player3.swap_remove(pos);
                                self.false_infostates_player3.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            4 => {
                if let Some(value) = self.reach_probs_player4.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player4.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player4.swap_remove(pos);
                                self.true_infostates_player4.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player4.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player4.swap_remove(pos);
                                self.false_infostates_player4.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            5 => {
                if let Some(value) = self.reach_probs_player5.get_mut(infostate){
                    if *value != status {
                        *value = status;
                        if status {
                            if let Some(pos) = self.false_infostates_player5.iter().position(|x| *x == infostate.to_string()){
                                self.false_infostates_player5.swap_remove(pos);
                                self.true_infostates_player5.push(infostate.to_string());
                            }
                        } else {
                            if let Some(pos) = self.true_infostates_player5.iter().position(|x| *x == infostate.to_string()){
                                self.true_infostates_player5.swap_remove(pos);
                                self.false_infostates_player5.push(infostate.to_string());
                            }
                        }
                    }
                }
            },
            _ => {
                panic!("Invalid Player ID, please provide between 0 to 5 both inclusive");
            }
        }
    }
    fn sort_true_infostates_by_length(&self) -> Vec<&Vec<String>> {
        let mut true_vectors = vec![
            &self.true_infostates_player0,
            &self.true_infostates_player1,
            &self.true_infostates_player2,
            &self.true_infostates_player3,
            &self.true_infostates_player4,
            &self.true_infostates_player5,
        ];
        
        true_vectors.sort_by(|a, b| b.len().cmp(&a.len()));
        true_vectors
    }

    fn sort_false_infostates_by_true_lengths(&self) -> Vec<&Vec<String>> {
        let true_lengths = vec![
            self.true_infostates_player0.len(),
            self.true_infostates_player1.len(),
            self.true_infostates_player2.len(),
            self.true_infostates_player3.len(),
            self.true_infostates_player4.len(),
            self.true_infostates_player5.len(),
        ];

        let mut false_vectors = vec![
            (&self.false_infostates_player0, true_lengths[0]),
            (&self.false_infostates_player1, true_lengths[1]),
            (&self.false_infostates_player2, true_lengths[2]),
            (&self.false_infostates_player3, true_lengths[3]),
            (&self.false_infostates_player4, true_lengths[4]),
            (&self.false_infostates_player5, true_lengths[5]),
        ];
        
        false_vectors.sort_by(|a, b| b.1.cmp(&a.1));
        false_vectors.into_iter().map(|(vec, _)| vec).collect()
    }

    pub fn should_pure_prune(&self) -> bool {
        let total_infostates: usize = INFOSTATES.len();
        let pointers_true: Vec<&Vec<String>> = self.sort_true_infostates_by_length();
        let pointers_false: Vec<&Vec<String>> = self.sort_false_infostates_by_true_lengths();
        let mut carrier_bool: bool;
        let mut counter_hm: HashMap<&str, usize> = HashMap::with_capacity(5);
        counter_hm.insert("A", 0);
        counter_hm.insert("B", 0);
        counter_hm.insert("C", 0);
        counter_hm.insert("D", 0);
        counter_hm.insert("E", 0);
        let mut i0: usize = 0;
        let mut i1: usize = 0;
        let mut i2: usize = 0;
        let mut i3: usize = 0;
        let mut i4: usize = 0;
        let mut i5: usize = 0;
        let mut break_bool: bool;
        while i0 < total_infostates {
            carrier_bool = false;
            break_bool = true;
            let infostate0: &str;
            if i0 < pointers_true[0].len(){
                // index true_infostates
                infostate0 = &pointers_true[0][i0];
                carrier_bool = true;
                break_bool = false;
            } else {
                // index false_infostates
                infostate0 = &pointers_false[0][i0 - pointers_true[0].len()]; 
            }
            if self.increment_continue(&infostate0, &mut counter_hm) {
                continue;
            }
            while i1 < total_infostates {
                let infostate1: &str;
                if i1 < pointers_true[1].len(){
                    // index true_infostates
                    infostate1 = &pointers_true[1][i1];
                    carrier_bool = true;
                    break_bool = false;
                } else {
                    // index false_infostates
                    infostate1 = &pointers_false[1][i1 - pointers_true[1].len()]; 
                }
                if self.increment_continue(&infostate1, &mut counter_hm) {
                    continue;
                }
                while i2 < total_infostates {
                    let infostate2: &str;
                    if i2 < pointers_true[2].len(){
                        // index true_infostates
                        infostate2 = &pointers_true[2][i2];
                        carrier_bool = true;
                        break_bool = false;
                    } else {
                        // index false_infostates
                        infostate2 = &pointers_false[2][i2 - pointers_true[2].len()]; 
                    }
                    if self.increment_continue(&infostate2, &mut counter_hm) {
                        continue;
                    }
                    while i3 < total_infostates {
                        let infostate3: &str;
                        if i3 < pointers_true[3].len(){
                            // index true_infostates
                            infostate3 = &pointers_true[3][i3];
                            carrier_bool = true;
                            break_bool = false;
                        } else {
                            // index false_infostates
                            infostate3 = &pointers_false[3][i3 - pointers_true[3].len()]; 
                        }
                        if self.increment_continue(&infostate3, &mut counter_hm) {
                            continue;
                        }
                        while i4 < total_infostates {
                            let infostate4: &str;
                            if i4 < pointers_true[4].len(){
                                // index true_infostates
                                infostate4 = &pointers_true[4][i4];
                                carrier_bool = true;
                                break_bool = false;
                            } else {
                                // index false_infostates
                                infostate4 = &pointers_false[4][i4 - pointers_true[4].len()]; 
                            }
                            if self.increment_continue(&infostate4, &mut counter_hm) {
                                continue;
                            }
                            while i5 < total_infostates {
                                let infostate5: &str;
                                if i5 < pointers_true[5].len(){
                                    // index true_infostates
                                    infostate5 = &pointers_true[5][i5];
                                    carrier_bool = true;
                                    break_bool = false;
                                } else {
                                    // index false_infostates
                                    infostate5 = &pointers_false[5][i5 - pointers_true[5].len()]; 
                                }
                                if self.increment_continue(&infostate5, &mut counter_hm) {
                                    continue;
                                }
                                if break_bool {
                                    // Breaks if all infostates traversed are 0
                                    // As all subsequent will also all be 0
                                    return true;
                                }
                                if carrier_bool {
                                    // If there exists a legal gamestate where at least 1 infostate is true
                                    // Do not prune and return false
                                    return false;
                                }
                                self.decrement(&infostate5, &mut counter_hm);
                                i5 += 1;
                            }
                            self.decrement(&infostate4, &mut counter_hm);
                            i4 += 1;
                        }
                        self.decrement(&infostate3, &mut counter_hm);
                        i3 += 1;
                    }
                    self.decrement(&infostate2, &mut counter_hm);
                    i2 += 1;
                }
                self.decrement(&infostate1, &mut counter_hm);
                i1 += 1;
            }
            self.decrement(&infostate0, &mut counter_hm);
            i0 += 1;
        }
        true
    }
    pub fn should_pure_prune_par(&self) -> bool {
        let total_infostates: usize = INFOSTATES.len();
        let pointers_true: Vec<&Vec<String>> = self.sort_true_infostates_by_length();
        let pointers_false: Vec<&Vec<String>> = self.sort_false_infostates_by_true_lengths();
        let mut carrier_bool: bool;
        let mut counter_hm_base: HashMap<&str, usize> = HashMap::with_capacity(5);
        counter_hm_base.insert("A", 0);
        counter_hm_base.insert("B", 0);
        counter_hm_base.insert("C", 0);
        counter_hm_base.insert("D", 0);
        counter_hm_base.insert("E", 0);

        let prune_result = thread::scope(|s|{
            (0..total_infostates).into_par_iter().any(|i0| {
                let mut i1: usize = 0;
                let mut i2: usize = 0;
                let mut i3: usize = 0;
                let mut i4: usize = 0;
                let mut i5: usize = 0;
                let mut carrier_bool: bool = false;
                let mut break_bool: bool = true;
                let mut counter_hm: HashMap<&str, usize> = counter_hm_base.clone();
                let infostate0: &str;
                if i0 < pointers_true[0].len(){
                    // index true_infostates
                    infostate0 = &pointers_true[0][i0];
                    carrier_bool = true;
                    break_bool = false;

                } else {
                    // index false_infostates
                    infostate0 = &pointers_false[0][i0 - pointers_true[0].len()]; 
                }
                self.increment_continue(&infostate0, &mut counter_hm);
                while i1 < total_infostates {
                    let infostate1: &str;
                    if i1 < pointers_true[1].len(){
                        // index true_infostates
                        infostate1 = &pointers_true[1][i1];
                        carrier_bool = true;
                        break_bool = false;
                    } else {
                        // index false_infostates
                        infostate1 = &pointers_false[1][i1 - pointers_true[1].len()]; 
                    }
                    if self.increment_continue(&infostate1, &mut counter_hm) {
                        continue;
                    }
                    while i2 < total_infostates {
                        let infostate2: &str;
                        if i2 < pointers_true[2].len(){
                            // index true_infostates
                            infostate2 = &pointers_true[2][i2];
                            carrier_bool = true;
                            break_bool = false;
                        } else {
                            // index false_infostates
                            infostate2 = &pointers_false[2][i2 - pointers_true[2].len()]; 
                        }
                        if self.increment_continue(&infostate2, &mut counter_hm) {
                            continue;
                        }
                        while i3 < total_infostates {
                            let infostate3: &str;
                            if i3 < pointers_true[3].len(){
                                // index true_infostates
                                infostate3 = &pointers_true[3][i3];
                                carrier_bool = true;
                                break_bool = false;
                            } else {
                                // index false_infostates
                                infostate3 = &pointers_false[3][i3 - pointers_true[3].len()]; 
                            }
                            if self.increment_continue(&infostate3, &mut counter_hm) {
                                continue;
                            }
                            while i4 < total_infostates {
                                let infostate4: &str;
                                if i4 < pointers_true[4].len(){
                                    // index true_infostates
                                    infostate4 = &pointers_true[4][i4];
                                    carrier_bool = true;
                                    break_bool = false;
                                } else {
                                    // index false_infostates
                                    infostate4 = &pointers_false[4][i4 - pointers_true[4].len()]; 
                                }
                                if self.increment_continue(&infostate4, &mut counter_hm) {
                                    continue;
                                }
                                while i5 < total_infostates {
                                    let infostate5: &str;
                                    if i5 < pointers_true[5].len(){
                                        // index true_infostates
                                        infostate5 = &pointers_true[5][i5];
                                        carrier_bool = true;
                                        break_bool = false;
                                    } else {
                                        // index false_infostates
                                        infostate5 = &pointers_false[5][i5 - pointers_true[5].len()]; 
                                    }
                                    if self.increment_continue(&infostate5, &mut counter_hm) {
                                        continue;
                                    }
                                    if break_bool {
                                        // Breaks if all infostates traversed are 0
                                        // As all subsequent will also all be 0
                                        return true;
                                    }
                                    if carrier_bool {
                                        // If there exists a legal gamestate where at least 1 infostate is true
                                        // Do not prune and return false
                                        return false;
                                    }
                                    self.decrement(&infostate5, &mut counter_hm);
                                    i5 += 1;
                                }
                                self.decrement(&infostate4, &mut counter_hm);
                                i4 += 1;
                            }
                            self.decrement(&infostate3, &mut counter_hm);
                            i3 += 1;
                        }
                        self.decrement(&infostate2, &mut counter_hm);
                        i2 += 1;
                    }
                    self.decrement(&infostate1, &mut counter_hm);
                    i1 += 1;
                }
                self.decrement(&infostate0, &mut counter_hm);
                return false;
            })
        }); 
        prune_result.unwrap_or(false)
    }
    fn increment_continue(&self, str_ref: &str, counter_hm: &mut HashMap<&str, usize> ) -> bool {
        // Takes a hashmap and increments according to str_ref value
        // Returns true if should continue because an impossible gamestate is reached!
        // If an impossible str_ref value is given, true is returned and nothing is incremented
        if str_ref == "AA" {
            if counter_hm["A"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 2;
                }
            }
        } else if str_ref == "AB" {
            if counter_hm["A"] > 2 || counter_hm["B"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
            }
        } else if str_ref == "AC" {
            if counter_hm["A"] > 2 || counter_hm["C"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
            }
        } else if str_ref == "AD" {
            if counter_hm["A"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "AE" {
            if counter_hm["A"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("A"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "BB" { 
            if counter_hm["B"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 2;
                }
            }
        } else if str_ref == "BC" { 
            if counter_hm["B"] > 2 || counter_hm["C"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
            }
        } else if str_ref == "BD" { 
            if counter_hm["B"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "BE" { 
            if counter_hm["B"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("B"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "CC" {
            if counter_hm["C"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 2;
                }
            }
        } else if str_ref == "CD" {
            if counter_hm["C"] > 2 || counter_hm["D"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
            }
        } else if str_ref == "CE" {
            if counter_hm["C"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("C"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "DD" {
            if counter_hm["D"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 2;
                }
            }
        } else if str_ref == "DE" {
            if counter_hm["D"] > 2 || counter_hm["E"] > 2{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("D"){
                    *value += 1;
                }
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 1;
                }
            }
        } else if str_ref == "EE" {
            if counter_hm["E"] > 1{
                return true;
            } else {
                if let Some(value) = counter_hm.get_mut("E"){
                    *value += 2;
                }
            }
        }
        false
    }
    fn decrement(&self, str_ref: &str, counter_hm: &mut HashMap<&str, usize> ) {
        // Takes a HashMap and decrements according to str_ref value
        if str_ref == "AA" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 2;
            }
        } else if str_ref == "AB" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
        } else if str_ref == "AC" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
        } else if str_ref == "AD" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "AE" {
            if let Some(value) = counter_hm.get_mut("A"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "BB" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 2;
            }
        } else if str_ref == "BC" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
        } else if str_ref == "BD" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "BE" { 
            if let Some(value) = counter_hm.get_mut("B"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "CC" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 2;
            }
        } else if str_ref == "CD" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
        } else if str_ref == "CE" {
            if let Some(value) = counter_hm.get_mut("C"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "DD" {
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 2;
            }
        } else if str_ref == "DE" {
            if let Some(value) = counter_hm.get_mut("D"){
                *value -= 1;
            }
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 1;
            }
        } else if str_ref == "EE" {
            if let Some(value) = counter_hm.get_mut("E"){
                *value -= 2;
            }
        }
    }
}