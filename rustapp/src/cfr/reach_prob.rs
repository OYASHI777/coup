// Reach prob
// 6 players so
// 6 hashmap <infostate, bool>
// 6 vec for infostates with true values
// 6 vec for infostates false values

struct ReachProb {
    reach_probs_player1: HashMap<String, bool>,
    reach_probs_player2: HashMap<String, bool>,
    reach_probs_player3: HashMap<String, bool>,
    reach_probs_player4: HashMap<String, bool>,
    reach_probs_player5: HashMap<String, bool>,
    reach_probs_player6: HashMap<String, bool>,
    
    true_infostates_player1: Vec<String>,
    true_infostates_player2: Vec<String>,
    true_infostates_player3: Vec<String>,
    true_infostates_player4: Vec<String>,
    true_infostates_player5: Vec<String>,
    true_infostates_player6: Vec<String>,
    
    false_infostates_player1: Vec<String>,
    false_infostates_player2: Vec<String>,
    false_infostates_player3: Vec<String>,
    false_infostates_player4: Vec<String>,
    false_infostates_player5: Vec<String>,
    false_infostates_player6: Vec<String>,
}

impl ReachProb {
    // Initialising to all true
    // cloning
    fn sort_true_infostates_by_length(&self) -> Vec<&Vec<String>> {
        let mut true_vectors = vec![
            &self.true_infostates_player1,
            &self.true_infostates_player2,
            &self.true_infostates_player3,
            &self.true_infostates_player4,
            &self.true_infostates_player5,
            &self.true_infostates_player6,
        ];
        
        true_vectors.sort_by(|a, b| b.len().cmp(&a.len()));
        true_vectors
    }

    fn sort_false_infostates_by_true_lengths(&self) -> Vec<&Vec<String>> {
        let true_lengths = vec![
            self.true_infostates_player1.len(),
            self.true_infostates_player2.len(),
            self.true_infostates_player3.len(),
            self.true_infostates_player4.len(),
            self.true_infostates_player5.len(),
            self.true_infostates_player6.len(),
        ];

        let mut false_vectors = vec![
            (&self.false_infostates_player1, true_lengths[0]),
            (&self.false_infostates_player2, true_lengths[1]),
            (&self.false_infostates_player3, true_lengths[2]),
            (&self.false_infostates_player4, true_lengths[3]),
            (&self.false_infostates_player5, true_lengths[4]),
            (&self.false_infostates_player6, true_lengths[5]),
        ];
        
        false_vectors.sort_by(|a, b| b.1.cmp(&a.1));
        false_vectors.into_iter().map(|(vec, _)| vec).collect()
    }
    // prune function but going through the true vector first
    // and if all 0 return true
}