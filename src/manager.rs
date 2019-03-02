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

use super::fitness::{Score, ScoreProvider, GeneralScoreProvider, FitnessFunction};
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

/// Returns a Manager object that will run the genetic algorithm.
/// Use this function if you're just writing a fitness function and not 
/// a special ScoreProvider.
/// fitness_function: A function you must define that determines the fitness of your agents.
/// data: additional immutable data to be used by during the run of the algorithm. Could be used as
/// a cache containing pre-calculated values or an initial state for data that will be changed when reading
/// the genes. Just use 0 if you have no other use for this argument.
pub fn create_manager<Gene, Data> (
    fitness_function: FitnessFunction<Gene, Data>,
    data: Data
) -> Manager<Gene, Data, GeneralScoreProvider<Gene, Data>>
where 
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    let manager: Manager<Gene, Data, GeneralScoreProvider<Gene, Data>> = Manager::new(fitness_function, data);
    manager 
}

pub struct Manager <Gene, Data, SP>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static,
SP: Clone + ScoreProvider<Gene, Data> + 'static
{
    main_population: Population<Gene>,
    data: Data,
    number_of_genes: usize,
    strict_gene_length: bool,
    initial_population_size: usize,
    current_highest: Score,
    agent_sender: Sender<BTreeMap<Score, Agent<Gene>>>,
    agent_receiver: Receiver<BTreeMap<Score, Agent<Gene>>>,
    number_of_child_threads: u8,
    max_child_threads: u8,
    operations: Vec<Operation<Gene, Data>>,
    iterations_per_cycle: usize,
    score_provider: SP
}

impl <Gene, Data, SP> Manager <Gene, Data, SP>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static,
SP: Clone + Send + ScoreProvider<Gene, Data>
{
    pub fn new(fitness_function: FitnessFunction<Gene, Data>, data: Data) -> Self {

        let (tx, rx) = channel::<BTreeMap<Score, Agent<Gene>>>();

        let operations = vec![
            Operation::new(OperationType::Mutate, Selection::new(SelectionType::RandomAny, 0.1)),
            Operation::new(OperationType::Crossover, Selection::new(SelectionType::HighestScore, 0.2)),
            Operation::new(OperationType::Crossover, Selection::new(SelectionType::RandomAny, 0.2)),
            Operation::new(OperationType::Cull, Selection::new(SelectionType::LowestScore, 0.1)),
        ];

        Self {
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
            operations: operations,
            iterations_per_cycle: 100,
            score_provider: SP::new(fitness_function, 25)
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

    pub fn set_iterations_per_cycle(&mut self, number: usize) {
        self.iterations_per_cycle = number;
    }

    pub fn run(&mut self, goal: Score) {
        self.main_population = Population::new(self.initial_population_size, self.number_of_genes, false, &self.data, &mut self.score_provider);

        while self.current_highest < goal {

            if self.number_of_child_threads < self.max_child_threads {
                for _ in 0..(self.max_child_threads - self.number_of_child_threads) {
                    self.spawn_population_in_new_thread();
                }
            }

            let cloned_population = self.main_population.clone();
            self.main_population = run_iterations(cloned_population, self.iterations_per_cycle, &self.data, &self.operations, &mut self.score_provider);

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
        let operations = self.operations.clone();
        let iterations_per_cycle = self.iterations_per_cycle;
        let mut score_provider = self.score_provider.clone();

        let tx = self.agent_sender.clone();

        thread::spawn(move || {
            let population = run_iterations(Population::new(initial_population_size, number_of_genes, false, &data, &mut score_provider), iterations_per_cycle, &data, &operations, &mut score_provider);
            let population = cull_lowest_agents(population, 0.5, 1);
            match tx.send(population.get_agents().clone()) {
                Ok(()) => (),
                Err(_) => () // The parent thread probably finished its run. That doesn't really matter.
            }
        });

        self.number_of_child_threads += 1;
    }
}