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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Clone)]
pub struct Agent <Gene> {
    genes: Vec<Gene>,
    hash: u64
}

impl <Gene> Agent<Gene> {
    pub fn new(number_of_genes: usize) -> Self where Standard: Distribution<Gene>, Gene: Hash {
        let mut genes = Vec::with_capacity(number_of_genes);
        for _ in 0..number_of_genes {
            genes.push(rand::random());
        }

        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        let hash = s.finish();

        Self {
            genes: genes,
            hash: hash
        }
    }

    pub fn get_genes(&self) -> &Vec<Gene> {
        return &self.genes;
    }

    pub fn crossover_some_genes(&mut self, other: &Self) where Gene: Clone + Hash {
        let mut rng = rand::thread_rng();
        
        let self_len = self.genes.len();
        let other_len = other.genes.len();

        let mut gene_count = self_len;
        if self_len > other_len {
            gene_count = other_len;
        }

        let crossover_point = rng.gen_range(0, gene_count);

        let mut self_crossover_point = crossover_point;
        let mut other_crossover_point = crossover_point;

        if self_len > other_len {
            self_crossover_point += self_len - other_len;
        } else if other_len > self_len {
            other_crossover_point += other_len - self_len;
        }

        self.genes.truncate(self_crossover_point);
        let mut other_genes = other.get_genes().clone();
        other_genes.drain(..other_crossover_point);
        self.genes.append(&mut other_genes);

        let mut s = DefaultHasher::new();
        self.genes.hash(&mut s);
        self.hash = s.finish();
    }

    pub fn mutate(&mut self) where Standard: Distribution<Gene>, Gene: Hash {
        let mut rng = rand::thread_rng();

        let gene_count = self.genes.len();

        for _ in 0..5 {
           self.genes.remove(rng.gen_range(0, gene_count));
           self.genes.insert(rng.gen_range(0, gene_count - 1), rand::random());
        }

        let mut s = DefaultHasher::new();
        self.genes.hash(&mut s);
        self.hash = s.finish();
    }

    pub fn has_same_genes(&self, other: &Self) -> bool {
        self.hash == other.hash
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }
}

pub fn mate <Gene> (parent1: &Agent<Gene>, parent2: &Agent<Gene>) -> Agent<Gene> 
where Gene: Clone + Hash {
    let mut child = parent1.clone();

    child.crossover_some_genes(parent2);

    return child;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_no_genes() {
        let agent: Agent<u8> = Agent::new(0);
        let empty_vec: Vec<u8> = Vec::new();
        assert_eq!(&empty_vec, agent.get_genes());

        // Hash is still generated when there are no genes.
        let mut s = DefaultHasher::new();
        empty_vec.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn new_with_genes() {
        let agent: Agent<u8> = Agent::new(2);

        let genes = agent.get_genes();
        assert_eq!(2, genes.len());

        // Ensure hash is already available.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn mutate() {
        let mut agent: Agent<u8> = Agent::new(2);

        agent.mutate();

        // Length should still be as specified in new().
        let genes = agent.get_genes();
        assert_eq!(2, genes.len());

        // Ensure hash is correct.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn crossover_some_genes_same_length_other() {
        let mut agent: Agent<u8> = Agent::new(6);
        let other: Agent<u8> = Agent::new(6);

        agent.crossover_some_genes(&other);

        // Length should still be as specified in new().
        let genes = agent.get_genes();
        assert_eq!(6, genes.len());

        // Ensure hash is correct.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn crossover_some_genes_shorter_other() {
        let mut agent: Agent<u8> = Agent::new(6);
        let other: Agent<u8> = Agent::new(5);

        agent.crossover_some_genes(&other);

        // Length should still be as specified in new().
        let genes = agent.get_genes();
        assert_eq!(6, genes.len());

        // Ensure hash is correct.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn crossover_some_genes_longer_other() {
        let mut agent: Agent<u8> = Agent::new(6);
        let other: Agent<u8> = Agent::new(7);

        agent.crossover_some_genes(&other);

        // Length should still be as specified in new().
        let genes = agent.get_genes();
        assert_eq!(6, genes.len());

        // Ensure hash is correct.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), agent.get_hash());
    }

    #[test]
    fn mate_parents() {
        let parent_one: Agent<u8> = Agent::new(6);
        let parent_two: Agent<u8> = Agent::new(5);

        let child = mate(&parent_one, &parent_two);

        // Length should be as for parent_one.
        let genes = child.get_genes();
        assert_eq!(6, genes.len());

        // Ensure hash is correct.
        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        assert_eq!(s.finish(), child.get_hash());
    }
}