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
use std::sync::mpsc::channel;
use super::agent::Agent;
use std::collections::BTreeMap;
use std::sync::mpsc::{Sender, Receiver};

pub struct Manager <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    score_function: ScoreFunction<Gene, Data>,
    main_population: Population<Gene>,
    data: Data,
    number_of_genes: usize,
    strict_gene_length: bool,
    initial_population_size: usize,
    current_highest: isize,
    agent_sender: Sender<BTreeMap<isize, Agent<Gene>>>,
    agent_receiver: Receiver<BTreeMap<isize, Agent<Gene>>>,
    number_of_child_threads: u8,
    max_child_threads: u8,
    operations: Vec<Operation<Gene, Data>>
}

impl <Gene, Data> Manager <Gene, Data>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{

    pub fn new(score_function: ScoreFunction<Gene, Data>, data: Data) -> Self {

        let (tx, rx) = channel::<BTreeMap<isize, Agent<Gene>>>();

        let operations = vec![
            Operation::new(OperationType::Mutate, Selection::new(SelectionType::RandomAny, 0.1)),
            Operation::new(OperationType::Crossover, Selection::new(SelectionType::HighestScore, 0.2)),
            Operation::new(OperationType::Crossover, Selection::new(SelectionType::RandomAny, 0.2)),
            Operation::new(OperationType::Cull, Selection::new(SelectionType::LowestScore, 0.1)),
        ];

        Self {
            score_function: score_function,
            main_population: Population::new_empty(false),
            data: data,
            number_of_genes: 10,
            strict_gene_length: false,
            initial_population_size: 100,
            current_highest: 0,
            agent_sender: tx,
            agent_receiver: rx,
            number_of_child_threads: 0,
            max_child_threads: 3,
            operations: operations
        }
    }

    pub fn set_number_of_genes(&mut self, number: usize, strict: bool) {
        self.number_of_genes = number;
        self.strict_gene_length = strict;
    }

    pub fn set_initial_population_size(&mut self, size: usize) {
        self.initial_population_size = size;
    }

    pub fn set_operations(&mut self, operations: Vec<Operation<Gene, Data>>) {
        self.operations = operations;
    }

    pub fn set_max_child_threads(&mut self, max_number: u8) {
        self.max_child_threads = max_number;
    }

    pub fn run(&mut self, goal: isize) {
        self.main_population = Population::new(self.initial_population_size, self.number_of_genes, false, &self.data, self.score_function);

        while self.current_highest < goal {

            if self.number_of_child_threads < self.max_child_threads {
                for _ in 0..(self.max_child_threads - self.number_of_child_threads) {
                    self.spawn_population_in_new_thread();
                }
            }

            let cloned_population = self.main_population.clone();
            self.main_population = run_iterations(cloned_population, 100, &self.data, &self.operations, self.score_function);

            let mut check_messages = true;
            while check_messages {
                let result = self.agent_receiver.try_recv();
                if result.is_ok() {
                    for (score, agent) in result.ok().unwrap() {
                        self.main_population.insert(score, agent);
                    }
                    self.number_of_child_threads -= 1;
                } else {
                    check_messages = false;
                }
            }

            let (highest, _) = self.main_population.get_agents().iter().rev().next().unwrap();
            self.current_highest = *highest;
        }
    }

    pub fn get_population(&self) -> &Population<Gene> {
        return &self.main_population;
    }

    fn spawn_population_in_new_thread(&mut self) {
        let initial_population_size = self.initial_population_size;
        let number_of_genes = self.number_of_genes;
        let data = self.data.clone();
        let score_function = self.score_function;
        let operations = self.operations.clone();

        let tx = self.agent_sender.clone();

        thread::spawn(move || {
            let population = run_iterations(Population::new(initial_population_size, number_of_genes, false, &data, score_function), 100, &data, &operations, score_function);
            let population = cull_lowest_agents(population, 0.5, 1);
            tx.send(population.get_agents().clone()).unwrap();
        });

        self.number_of_child_threads += 1;
    }
}