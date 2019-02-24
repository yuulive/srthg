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

use super::population::Population;
use super::operations::{
    Operation
};
use super::fitness::{ScoreProvider};
use rand::{
    distributions::{Distribution, Standard}
};
use std::hash::Hash;

pub fn run_iterations<Gene, Data, SP>(
    mut population: Population<Gene>,
    iterations: usize,
    data: &Data,
    operations: &Vec<Operation<Gene, Data>>,
    score_provider: &mut SP
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static,
SP: Clone + ScoreProvider<Gene, Data>
{
    for _ in 0..iterations {
        for operation in operations.iter() {
            population = operation.run(population, data, score_provider);
        }
    }

    population
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::agent::Agent;
    use super::super::fitness::{Score, ScoreError, GeneralScoreProvider};

    fn get_score_index(agent: &Agent<u8>, _data: &u8) -> Result<Score, ScoreError> {
        let score = agent.get_genes()[0] as Score;
        Ok(score)
    }

    #[test]
    fn run_iterations_nothing_to_do() {
        let mut score_provider = GeneralScoreProvider::new(get_score_index, 25);
        let population = run_iterations(Population::new_empty(false), 0, &0, &Vec::new(), &mut score_provider);
        assert_eq!(0, population.len());
    }
}
