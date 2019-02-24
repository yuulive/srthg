use super::agent::Agent;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
    prelude::ThreadRng
};
use std::collections::HashMap;


pub type FitnessFunction<Gene, Data> = fn(&Agent<Gene>, &Data) -> Score;

pub type Score = u64;

#[derive(Clone)]
pub struct ScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    scoring_function: FitnessFunction<Gene, Data>,
    offset: Score,
    score_cache: HashMap<u64, Score>
}

impl <Gene, Data> ScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    pub fn new(scoring_function: FitnessFunction<Gene, Data>, offset: Score) -> Self {
        Self {
            scoring_function: scoring_function,
            offset: offset,
            score_cache: HashMap::new()
        }
    }

    pub fn get_score(&mut self, agent: &Agent<Gene>, data: &Data, rng: &mut ThreadRng) -> Score {
        let hash = agent.get_hash();

        let offset = rng.gen_range(0, self.offset * 2);

        if self.score_cache.contains_key(&hash) {
            let score = self.score_cache[&hash] + offset - self.offset;
            if score <= self.offset {
                return 0;
            } else {
                return score - self.offset;
            }
        }

        let score = (self.scoring_function)(agent, data);
        self.score_cache.insert(hash, score);

        let score = score + offset;

        if score <= self.offset {
            return 0;
        } else {
            return score - self.offset;
        }
    }
}