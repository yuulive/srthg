// Copyright 2019 Brendan Cox
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::agent::Agent;
use super::operations::{Score, ScoreProvider};
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Clone)]
pub struct Population <Gene> {
    agents: BTreeMap<Score, Agent<Gene>>,
    register: HashSet<u64>,
    unique_agents: bool,

}

impl <Gene> Population <Gene> {

    pub fn new_empty(unique: bool) -> Self {
        Self {
            agents: BTreeMap::new(),
            register: HashSet::new(),
            unique_agents: unique
        }
    }

    pub fn new<Data>(
        start_size: usize,
        number_of_genes: usize,
        unique: bool,
        data: &Data,
        score_provider: &mut ScoreProvider<Gene, Data>,
    ) -> Population<Gene> 
    where
    Standard: Distribution<Gene>,
    Gene: Hash + Clone
    {
        let mut population = Population::new_empty(unique);
        let mut rng = rand::thread_rng();
        for _ in 0..start_size {
            let agent = Agent::with_genes(number_of_genes);
            if population.will_accept(&agent) {
                let mut score = score_provider.get_score(&agent, &data, &mut rng);

                loop {
                    if score == 0 {
                        break;
                    }
                    if population.contains_score(score) {
                        score -= 1;
                    } else {
                        break;
                    }
                }

                population.insert(score, agent);
            }
        }

        population
    }

    pub fn set_agents(&mut self, agents: BTreeMap<Score, Agent<Gene>>) {
        for (score, agent) in agents {
            self.insert(score, agent);
        }
    }

    pub fn insert(&mut self, score: Score, agent: Agent<Gene>) {
        if self.unique_agents {
            if self.register.contains(&agent.get_hash()) {
                return;
            }
            self.register.insert(agent.get_hash());
        }
        self.agents.insert(score, agent);
    }

    pub fn remove(&mut self, score: Score) -> Option<Agent<Gene>> where Gene: Clone {
        let agent = self.agents.remove(&score);
        if self.unique_agents && agent.is_some() {
            self.register.remove(&agent.clone().unwrap().get_hash());
        }
        agent
    }

    pub fn get(&self, score: Score) -> Option<&Agent<Gene>> {
        self.agents.get(&score)
    }

    pub fn get_agents(&self) -> &BTreeMap<Score, Agent<Gene>> {
        &self.agents
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn cull_all_below(&mut self, score: Score) {
        self.agents = self.agents.split_off(&score);
        if self.unique_agents {
            self.register.clear();
            for (_, agent) in &self.agents {
                self.register.insert(agent.get_hash());
            }
        }
    }

    pub fn cull_all_above(&mut self, score: Score) {
        self.agents.split_off(&score);
        if self.unique_agents {
            self.register.clear();
            for (_, agent) in &self.agents {
                self.register.insert(agent.get_hash());
            }
        }
    }

    pub fn contains_score(&self, score: Score) -> bool {
        self.agents.contains_key(&score)
    }

    pub fn will_accept(&self, agent: &Agent<Gene>) -> bool {
        if self.unique_agents {
            return !self.register.contains(&agent.get_hash());
        }
        true
    }

    pub fn get_scores(&self) -> Vec<Score> {
        self.agents.keys().map(|k| *k).collect()
    }

    pub fn get_random_score(&self) -> Score {
        let mut rng = rand::thread_rng();
        self.get_scores()[rng.gen_range(0, self.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let population: Population<u8> = Population::new_empty(false);
        assert_eq!(0, population.len());
        assert_eq!(0, population.get_agents().len());
        assert_eq!(0, population.get_scores().len());
    }

    fn get_score_index(agent: &Agent<u8>, _data: &u8) -> Score {
        agent.get_genes()[0] as Score
    }

    #[test]
    fn new_with_false_unique() {
        let mut population = Population::new(5, 6, false, &0, &mut ScoreProvider::new(get_score_index, 25));
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
        for (_score, agent) in population.get_agents() {
            assert_eq!(6, agent.get_genes().len());
        }

        let random_score = population.get_random_score();
        let agent = population.get(random_score).unwrap().clone();
        assert!(population.will_accept(&agent));
        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        population.insert(new_score, agent);
        assert_eq!(6, population.len());
        assert_eq!(6, population.get_agents().len());
        assert_eq!(6, population.get_scores().len());
    }

    #[test]
    fn new_with_true_unique() {
        let mut population = Population::new(5, 6, true, &0, &mut ScoreProvider::new(get_score_index, 25));
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
        for (_score, agent) in population.get_agents() {
            assert_eq!(6, agent.get_genes().len());
        }

        let random_score = population.get_random_score();
        let agent = population.get(random_score).unwrap().clone();
        assert!(!population.will_accept(&agent));
        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        population.insert(new_score, agent.clone());
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());

        population.remove(random_score);
        assert_eq!(4, population.len());
        assert_eq!(4, population.get_agents().len());
        assert_eq!(4, population.get_scores().len());

        population.insert(new_score, agent);
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
    }

    #[test]
    fn cull_all_below() {
        let mut population = Population::new(5, 6, true, &0, &mut ScoreProvider::new(get_score_index, 25));
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());

        let lowest = population.get_scores()[0];
        let second_lowest = population.get_scores()[1];
        let middle = population.get_scores()[2];
        let second_highest = population.get_scores()[3];
        let highest = population.get_scores()[4];
        
        // Ensure ordering is as expected.
        assert!(highest > lowest);

        // Will be used for checking register of hashes was updated.
        let lowest_clone = population.get(lowest).unwrap().clone();
        let highest_clone = population.get(highest).unwrap().clone();

        population.cull_all_below(middle);
        assert_eq!(3, population.len());
        assert_eq!(3, population.get_agents().len());
        assert_eq!(3, population.get_scores().len());

        assert!(!population.contains_score(lowest));
        assert!(!population.contains_score(second_lowest));
        assert!(population.contains_score(middle));
        assert!(population.contains_score(second_highest));
        assert!(population.contains_score(highest));

        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        // The highest is still in there and so its clone should not be accepted.
        assert!(!population.will_accept(&highest_clone));
        population.insert(new_score, highest_clone);
        assert_eq!(3, population.len());
        assert_eq!(3, population.get_agents().len());
        assert_eq!(3, population.get_scores().len());

        // The lowest is no longer there and so its clone can be accepted.
        assert!(population.will_accept(&lowest_clone));
        population.insert(new_score, lowest_clone);
        assert_eq!(4, population.len());
        assert_eq!(4, population.get_agents().len());
        assert_eq!(4, population.get_scores().len());
    }
}
