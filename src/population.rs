use super::agent::Agent;
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};


pub struct Population <Gene: Clone> {
    agents: BTreeMap<isize, Agent<Gene>>,
    register: HashSet<u64>,
    unique_agents: bool
}

impl <Gene> Population <Gene>
where Standard: Distribution<Gene>, Gene: Clone + PartialEq + Hash {

    pub fn new_empty() -> Self {
        Self {
            agents: BTreeMap::new(),
            register: HashSet::new(),
            unique_agents: false
        }
    }

    pub fn set_agents(&mut self, agents: BTreeMap<isize, Agent<Gene>>) {
        for (score, agent) in agents {
            self.insert(score, agent);
        }
    }

    pub fn insert(&mut self, score: isize, agent: Agent<Gene>) {
        if self.unique_agents {
            if self.register.contains(&agent.get_hash()) {
                return;
            }
            self.register.insert(agent.get_hash());
        }
        self.agents.insert(score, agent);
    }

    pub fn remove(&mut self, score: isize) -> Option<Agent<Gene>> {
        let agent = self.agents.remove(&score);
        if self.unique_agents && agent.is_some() {
            self.register.remove(&agent.clone().unwrap().get_hash());
        }
        agent
    }

    pub fn get(&self, score: isize) -> Option<&Agent<Gene>> {
        self.agents.get(&score)
    }

    pub fn get_agents(&self) -> &BTreeMap<isize, Agent<Gene>> {
        &self.agents
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn cull_all_below(&mut self, score: isize) {
        self.agents = self.agents.split_off(&score);
        if self.unique_agents {
            self.register.clear();
            for (_, agent) in &self.agents {
                self.register.insert(agent.get_hash());
            }
        }
    }

    pub fn contains_score(&self, score: isize) -> bool {
        self.agents.contains_key(&score)
    }

    pub fn will_accept(&self, agent: &Agent<Gene>) -> bool {
        if self.unique_agents {
            return !self.register.contains(&agent.get_hash());
        }
        true
    }

    pub fn get_scores(&self) -> Vec<isize> {
        self.agents.keys().map(|k| *k).collect()
    }

    pub fn get_random_score(&self) -> isize {
        let mut rng = rand::thread_rng();
        self.get_scores()[rng.gen_range(0, self.len())]
    }
}