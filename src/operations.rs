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

use super::agent::{Agent, crossover};
use super::population::Population;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
    prelude::ThreadRng
};
use std::marker::{Send, PhantomData};
use std::collections::{BTreeMap, HashMap};

pub type ScoreFunction<Gene, Data> = fn(&Agent<Gene>, &Data) -> Score;

#[derive(Clone, Copy)]
pub enum OperationType {
    Mutate,
    Crossover,
    Cull
}

#[derive(Clone, Copy)]
pub enum SelectionType {
    RandomAny,
    HighestScore,
    LowestScore
}

/// Allows definition of parameters for selecting some agents from a population.
#[derive(Clone, Copy)]
pub struct Selection {
    selection_type: SelectionType,
    proportion: f64,
    preferred_minimum: usize
}

impl Selection {
    pub fn with_values(selection_type: SelectionType, proportion: f64, preferred_minimum: usize) -> Self {
        Self {
            selection_type: selection_type,
            proportion: proportion,
            preferred_minimum: preferred_minimum
        }
    }

    pub fn new(selection_type: SelectionType, proportion: f64) -> Self {
        Self {
            selection_type: selection_type,
            proportion: proportion,
            preferred_minimum: 1
        }
    }

    pub fn selection_type(&self) -> SelectionType {
        self.selection_type
    }

    pub fn proportion(&self) -> f64 {
        self.proportion
    }

    pub fn preferred_minimum(&self) -> usize {
        self.preferred_minimum
    }

    pub fn agents <'a, Gene> (&self, population: &'a Population<Gene>) -> BTreeMap<Score, &'a Agent<Gene>>
    where
    Gene: Clone
    {
        match self.selection_type {
            SelectionType::RandomAny => get_random_subset(population.get_agents(), self.proportion, self.preferred_minimum),
            SelectionType::HighestScore => get_highest_scored_agents(population.get_agents(), self.proportion, self.preferred_minimum),
            SelectionType::LowestScore => get_lowest_scored_agents(population.get_agents(), self.proportion, self.preferred_minimum)
        }
    }

    pub fn count <Gene> (&self, population: &Population<Gene>) -> usize {
        rate_to_number(population.len(), self.proportion, self.preferred_minimum)
    }
}

/// Modifies a selection of a population.
#[derive(Clone)]
pub struct Operation <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    selection: Selection,
    operation_type: OperationType,
    gene: PhantomData<Gene>,
    data: PhantomData<Data>
}

impl <Gene, Data> Operation <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    pub fn with_values(
        selection: Selection,
        operation_type: OperationType
        ) -> Self {
        Self {
            selection: selection,
            operation_type: operation_type,
            gene: PhantomData,
            data: PhantomData
        }
    }

    pub fn new(
        operation_type: OperationType,
        selection: Selection
    ) -> Self {
        Self {
            selection: selection,
            operation_type: operation_type,
            gene: PhantomData,
            data: PhantomData
        }
    }

    pub fn run (&self, population: Population<Gene>, data: &Data, score_provider: &mut ScoreProvider<Gene, Data>) -> Population<Gene>
    {
        match self.operation_type {
            OperationType::Mutate => mutate_agents(population, self.selection, data, score_provider),
            OperationType::Crossover => crossover_agents(population, self.selection, data, score_provider),
            OperationType::Cull => cull_agents(population, self.selection)
        }
    }
}

pub type Score = u64;

#[derive(Clone)]
pub struct ScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    scoring_function: ScoreFunction<Gene, Data>,
    offset: Score,
    score_cache: HashMap<u64, Score>
}

impl <Gene, Data> ScoreProvider <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    pub fn new(scoring_function: ScoreFunction<Gene, Data>, offset: Score) -> Self {
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

fn mutate_agents<Gene, Data>(
    mut population: Population<Gene>,
    selection: Selection,
    data: &Data,
    score_provider: &mut ScoreProvider<Gene, Data>
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    let children = get_mutated_agents(selection.agents(&population));
    let mut rng = rand::thread_rng();
    for agent in children {
        let score_index = score_provider.get_score(&agent, data, &mut rng);
        population.insert(score_index, agent);
    }

    population
}

fn crossover_agents<Gene, Data>(
    mut population: Population<Gene>,
    selection: Selection,
    data: &Data,
    score_provider: &mut ScoreProvider<Gene, Data>
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    let pairs = create_random_pairs(
        selection.agents(&population)
    );

    let children = create_children_from_crossover(pairs, data, score_provider);
    for (score_index, agent) in children {
        population.insert(score_index, agent);
    }

    population
}

fn cull_agents<Gene>(
    mut population: Population<Gene>,
    selection: Selection,
) -> Population<Gene>
{
    let keys: Vec<Score> = population.get_agents().keys().map(|k| *k).collect();
    let cull_number = selection.count(&population);
    if cull_number >= keys.len() {
        return population;
    }
    
    match selection.selection_type() {
        SelectionType::LowestScore => population.cull_all_below(keys[cull_number]),
        SelectionType::HighestScore => population.cull_all_above(keys[cull_number]),
        SelectionType::RandomAny => panic!("RandomAny selection not yet implemented for cull agents")
    };
    population
}

fn get_mutated_agents<Gene>(
    agents: BTreeMap<Score, &Agent<Gene>>,
) -> Vec<Agent<Gene>>
where Standard: Distribution<Gene>,
Gene: Clone + Hash + Send
{
    let mut children = Vec::new();
    for (_, mut agent) in agents {
        let mut clone = agent.clone();
        clone.mutate();
        children.push(clone);
    }
    children
}

fn create_children_from_crossover<Gene, Data>(
    pairs: Vec<(Agent<Gene>, Agent<Gene>)>,
    data: &Data,
    score_provider: &mut ScoreProvider<Gene, Data>,
) -> Vec<(Score, Agent<Gene>)>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash
{
    let mut children = Vec::new();
    let mut rng = rand::thread_rng();
    for (parent_one, parent_two) in pairs {
        let child = crossover(&parent_one, &parent_two);
        let score_index = score_provider.get_score(&child, data, &mut rng);
        children.push((score_index, child));
    }
    return children;
}

fn get_random_subset<Gene>(
    agents: &BTreeMap<Score, Agent<Gene>>,
    rate: f64,
    preferred_minimum: usize
) -> BTreeMap<Score, &Agent<Gene>>
where Gene: Clone
{
    let number = rate_to_number(agents.len(), rate, preferred_minimum);
    let keys: Vec<Score> = agents.keys().map(|k| *k).collect();
    let mut rng = rand::thread_rng();
    let mut subset = BTreeMap::new();
    for _ in 0..number {
        let key = keys[rng.gen_range(0, keys.len())];
        let agent = agents.get(&key);
        if agent.is_some() {
            subset.insert(key, agent.unwrap());
        }
    }

    subset
}

fn get_highest_scored_agents<Gene>(
    agents: &BTreeMap<Score, Agent<Gene>>,
    rate: f64,
    preferred_minimum: usize
) -> BTreeMap<Score, &Agent<Gene>>
where Gene: Clone
{
    let number = rate_to_number(agents.len(), rate, preferred_minimum);
    let mut keys: Vec<Score> = agents.keys().map(|k| *k).collect();
    let keys_len = keys.len();
    keys.drain(0..(keys_len - number));
    let mut subset = BTreeMap::new();
    for key in keys {
        let agent = agents.get(&key);
        if agent.is_some() {
            subset.insert(key, agent.unwrap());
        }
    }

    subset
}

fn get_lowest_scored_agents<Gene>(
    agents: &BTreeMap<Score, Agent<Gene>>,
    rate: f64,
    preferred_minimum: usize
) -> BTreeMap<Score, &Agent<Gene>>
where Gene: Clone
{
    let number = rate_to_number(agents.len(), rate, preferred_minimum);
    let mut keys: Vec<Score> = agents.keys().map(|k| *k).collect();
    keys.truncate(number);
    let mut subset = BTreeMap::new();
    for key in keys {
        let agent = agents.get(&key);
        if agent.is_some() {
            subset.insert(key, agent.unwrap());
        }
    }

    subset
}

fn create_random_pairs<Gene>(
    agents: BTreeMap<Score, &Agent<Gene>>,
) -> Vec<(Agent<Gene>, Agent<Gene>)> 
where
Gene: Clone
{
    let keys: Vec<&Score> = agents.keys().collect();
    let mut rng = rand::thread_rng();
    let mut pairs = Vec::new();
    let count = keys.len();
    for _ in 0..count {
        let one_key = keys[rng.gen_range(0, keys.len())];
        let two_key = keys[rng.gen_range(0, keys.len())];

        let one_agent = agents.get(one_key);
        let two_agent = agents.get(two_key);
        if one_agent.is_some() && two_agent.is_some() {
            let one_agent = *one_agent.unwrap();
            let two_agent = *two_agent.unwrap();
            if !one_agent.has_same_genes(two_agent) {
                pairs.push((one_agent.clone(), two_agent.clone()));
            }
        }
    }

    pairs
}


pub fn cull_lowest_agents<Gene>(
    mut population: Population<Gene>,
    rate: f64,
    preferred_minimum: usize
) -> Population<Gene>
{
    let keys: Vec<Score> = population.get_agents().keys().map(|k| *k).collect();
    let cull_number = rate_to_number(keys.len(), rate, preferred_minimum);
    if cull_number >= keys.len() {
        return population;
    }
    population.cull_all_below(keys[cull_number]);
    population
}

fn rate_to_number(population: usize, rate: f64, preferred_minimum: usize) -> usize {
    if population < preferred_minimum {
        return population;
    }
    let number = (population as f64 * rate) as usize;
    if number < preferred_minimum {
        return preferred_minimum;
    }

    number
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_score_index(agent: &Agent<u8>, _data: &u8) -> Score {
        agent.get_genes()[0] as Score
    }

    #[test]
    fn selection_random_any_returns_correct_proportion() {
        let selection = Selection::with_values(SelectionType::RandomAny, 0.25, 0);

        let population = Population::new(8, 1, false, &0, &mut ScoreProvider::new(get_score_index, 25));

        let agent_map = selection.agents(&population);
        assert_eq!(2, agent_map.len());
    }

    #[test]
    fn selection_highest_score_returns_highest() {
        let selection = Selection::with_values(SelectionType::HighestScore, 0.25, 0);

        let population = Population::new(8, 1, false, &0, &mut ScoreProvider::new(get_score_index, 25));

        let agent_map = selection.agents(&population);
        assert_eq!(2, agent_map.len());

        let mut iter = population.get_agents().iter().rev();
        let (score, _) = iter.next().unwrap();
        assert!(agent_map.contains_key(score));
        let (score, _) = iter.next().unwrap();
        assert!(agent_map.contains_key(score));
    }

    #[test]
    fn selection_lowest_score_returns_lowest() {
        let selection = Selection::with_values(SelectionType::LowestScore, 0.25, 0);

        let population = Population::new(8, 1, false, &0, &mut ScoreProvider::new(get_score_index, 25));

        let agent_map = selection.agents(&population);
        assert_eq!(2, agent_map.len());

        let mut iter = population.get_agents().iter();
        let (score, _) = iter.next().unwrap();
        assert!(agent_map.contains_key(score));
        let (score, _) = iter.next().unwrap();
        assert!(agent_map.contains_key(score));
    }

    #[test]
    fn rate_to_number_standard_proportion() {
        assert_eq!(16, rate_to_number(20, 0.8, 0));
    }

    #[test]
    fn rate_to_number_population_is_zero() {
        assert_eq!(0, rate_to_number(0, 0.0, 0));
        assert_eq!(0, rate_to_number(0, 0.8, 0));
    }

    #[test]
    fn rate_to_number_full_proportion() {
        assert_eq!(20, rate_to_number(20, 1.0, 0));
    }

    #[test]
    fn rate_to_number_rounds_down() {
        assert_eq!(7, rate_to_number(10, 0.75, 0));
        assert_eq!(7, rate_to_number(10, 0.71, 0));
        assert_eq!(7, rate_to_number(10, 0.79, 0));
    }

    #[test]
    fn rate_to_number_minimum_preference_less_than_proportion() {
        assert_eq!(7, rate_to_number(10, 0.7, 5));
    }

    #[test]
    fn rate_to_number_minimum_preference_greater_than_proportion() {
        assert_eq!(8, rate_to_number(10, 0.7, 8));
    }

    #[test]
    fn rate_to_number_minimum_preference_greater_than_population() {
        assert_eq!(4, rate_to_number(4, 0.5, 5));
    }
}