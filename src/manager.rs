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

use super::operations::ScoreFunction;
use super::population::Population;
use super::evolution::run_iterations;
use rand::{
    distributions::{Distribution, Standard}
};
use std::hash::Hash;
use super::operations::{
    Operation,
    OperationType,
    Selection,
    SelectionType,
    cull_lowest_agents
};
use std::thread; 
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use super::agent::Agent;
use std::collections::BTreeMap;
use std::sync::mpsc::{Sender, Receiver};

pub struct Manager <Gene, Data>
where
Gene: 'static,
Data: 'static
{
    score_function: ScoreFunction<Gene, Data>,
    main_population: Population<Gene>,
    data: Data,
    number_of_genes: usize,
    strict_gene_length: bool,
    initial_population_size: usize,
    current_highest: isize,
    agent_sender: Sender<BTreeMap<isize, Agent<Gene>>>,
    agent_receiver: Receiver<BTreeMap<isize, Agent<Gene>>>
}

impl <Gene, Data> Manager <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{

    pub fn new(score_function: ScoreFunction<Gene, Data>, data: Data) -> Self {

        let (tx, rx) = channel::<BTreeMap<isize, Agent<Gene>>>();

        Self {
            score_function: score_function,
            main_population: Population::new_empty(false),
            data: data,
            number_of_genes: 10,
            strict_gene_length: false,
            initial_population_size: 100,
            current_highest: 0,
            agent_sender: tx,
            agent_receiver: rx
        }
    }

    pub fn set_number_of_genes(&mut self, number: usize, strict: bool) {
        self.number_of_genes = number;
        self.strict_gene_length = strict;
    }

    pub fn set_initial_population_size(&mut self, size: usize) {
        self.initial_population_size = size;
    }

    pub fn run(&mut self, goal: isize) {
        self.main_population = Population::new(self.initial_population_size, self.number_of_genes, false, &self.data, self.score_function);
        
        let operations = self.get_operations();

        while self.current_highest < goal {

            self.spawn_population_in_new_thread();

            let cloned_population = self.main_population.clone();
            self.main_population = run_iterations(cloned_population, 100, &self.data, &operations, self.score_function);

            let result = self.agent_receiver.try_recv();
            if result.is_ok() {
                for (score, agent) in result.ok().unwrap() {
                    self.main_population.insert(score, agent);
                }
            }

            let (highest, _) = self.main_population.get_agents().iter().rev().next().unwrap();
            self.current_highest = *highest;
        }
    }

    pub fn get_population(&self) -> &Population<Gene> {
        return &self.main_population;
    }

    fn get_operations(&self) -> Vec<Operation<Gene, Data>> {
        vec![
            Operation::with_values(Selection::with_values(SelectionType::RandomAny, 0.1, 1), OperationType::Mutate, 25, 1),
            Operation::with_values(Selection::with_values(SelectionType::HighestScore, 0.2, 1), OperationType::Crossover, 25, 1),
            Operation::with_values(Selection::with_values(SelectionType::RandomAny, 0.5, 1), OperationType::Crossover, 25, 1),
            Operation::with_values(Selection::with_values(SelectionType::LowestScore, 0.02, 1), OperationType::Cull, 25, 1)
        ]
    }

    fn spawn_population_in_new_thread(&self) -> JoinHandle<Population<Gene>> {
            let initial_population_size = self.initial_population_size;
            let number_of_genes = self.number_of_genes;
            let data = self.data.clone();
            let score_function = self.score_function;
            let operations = self.get_operations();

            let tx = self.agent_sender.clone();

            let handle = thread::spawn(move || {
                let population = run_iterations(Population::new(initial_population_size, number_of_genes, false, &data, score_function), 100, &data, &operations, score_function);
                tx.send(population.get_agents().clone()).unwrap();
                population
            });

            handle
    }
}