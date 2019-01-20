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

extern crate aristeia;

use aristeia::evolution::run_iterations;
use aristeia::agent::Agent;
use aristeia::population::Population;
use aristeia::operations::{
    Operation,
    OperationType,
    Selection,
    SelectionType,
};

fn main() {

    let operations = vec![
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.1, 1),
            OperationType::Mutate,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::HighestScore, 0.1, 1),
            OperationType::Crossover,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::LowestScore, 0.1, 1),
            OperationType::Cull,
            25,
            1)
    ];

    let population = Population::new(100, 5, false, &0, get_score_index);
    let population = run_iterations(population, 50, &0, &operations, get_score_index);

    println!("Population: {}", population.len());

    let mut viewing = 10;
    for (score_index, agent) in population.get_agents().iter().rev() {
        println!("Score: {}", score_index);
        println!("{:?}", agent.get_genes());

        viewing -= 1;
        if viewing == 0 {
            break;
        }
    }
}

fn get_score_index(agent: &Agent<u8>, _data: &u8) -> isize {
    let mut score = 0;

    for gene in agent.get_genes() {
        score += *gene as isize;
    }

    score
}