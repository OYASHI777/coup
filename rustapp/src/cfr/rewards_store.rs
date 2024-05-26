pub struct RewardsStore {
    rewards: HashMap<MSKey, HashMap<BRKey, f32>>,
}

impl RewardsStore {
    pub fn new() -> Self {
        RewardsStore {
            rewards: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: MSKey, value: HashMap<BRKey, f32>) {
        self.rewards.insert(key, value);
    }
    pub fn reset(&mut self) {
        self.rewards.clear();
    }
    pub fn remove(&mut self, key: &MSKey) {
        if self.rewards.contains_key(key) {
            self.rewards.remove(key);
        }
    }
    pub fn get(&mut self, key: &MSKey) -> Option<&HashMap<BRKey, f32>>{
        self.rewards.get(key)
    }
}