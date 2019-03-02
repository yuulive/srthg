use super::agent::Agent;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
    prelude::ThreadRng
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ScoreError {
    details: String
}

impl Display for ScoreError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ScoreError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub type FitnessFunction<Gene, Data> = fn(&Agent<Gene>, &Data) -> Result<Score, ScoreError>;

pub type Score = u64;

pub trait ScoreProvider <Gene, Data> {
    fn evaluate_scores(&mut self, agents: Vec<Agent<Gene>>, data: &Data) -> Vec<Agent<Gene>>;
    fn get_score(&mut self, agent: &Agent<Gene>, data: &Data, rng: &mut ThreadRng) -> Result<Score, ScoreError>;
}

#[derive(Clone)]
pub struct GeneralScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    scoring_function: FitnessFunction<Gene, Data>,
    offset: Score,
    score_cache: HashMap<u64, Score>
}

impl <Gene, Data> GeneralScoreProvider <Gene, Data>
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

    pub fn offset_cached_score(&self, hash: &u64, offset: Score) -> Result<Score, ScoreError> {
        let score = self.score_cache[&hash] + offset;
        if score <= self.offset {
            return Ok(0);
        } else {
            return Ok(score - self.offset);
        }
    }
}

impl <Gene, Data> ScoreProvider<Gene, Data> for GeneralScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    fn evaluate_scores(&mut self, agents: Vec<Agent<Gene>>, data: &Data) -> Vec<Agent<Gene>> {
        let mut cached = Vec::new();
        
        for agent in agents {
            let hash = agent.get_hash();
            if self.score_cache.contains_key(&hash) {
                cached.push(agent);
            } else {
                let result = (self.scoring_function)(&agent, data);
                if result.is_ok() {
                    self.score_cache.insert(hash, result.unwrap());
                    cached.push(agent);
                }
                // else we simply skip the agent.
            }
        }

        cached
    }

    fn get_score(&mut self, agent: &Agent<Gene>, data: &Data, rng: &mut ThreadRng) -> Result<Score, ScoreError> {
        let hash = agent.get_hash();
        let offset = rng.gen_range(0, self.offset * 2);

        if self.score_cache.contains_key(&hash) {
            return self.offset_cached_score(&hash, offset);
        }

        let score = (self.scoring_function)(agent, data).unwrap();
        self.score_cache.insert(hash, score);

        return self.offset_cached_score(&hash, offset);
    }
}

