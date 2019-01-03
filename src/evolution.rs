use super::population::Population;
use super::operations::{
    mutate_some_agents,
    mate_some_agents,
    cull_lowest_agents,
    mate_alpha_agents
};
use std::thread;
use rand::{
    distributions::{Distribution, Standard}
};
use std::hash::Hash;
use super::agent::Agent;

pub fn population_from_multilevel_sub_populations<Gene, IndexFunction, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction
) -> Population<Gene> 
where
Gene: Clone + Hash + Send + 'static,
Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let number_of_initial_populations = sub_populations_per_level.pow(levels);
    let mut populations = Vec::new();
    for _ in 0..number_of_initial_populations {
        populations.push(
            run_iterations(
                Population::new(initial_population_size, number_of_genes, false, &data, get_score_index),
                iterations_on_each_population,
                &data,
                get_score_index
            )
        );
    }

    populations_from_existing_multillevel(populations, levels, sub_populations_per_level, &data, iterations_on_each_population, get_score_index)
}

pub fn threaded_population_from_multilevel_sub_populations<Gene, IndexFunction, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction
) -> Population<Gene> 
where
Gene: Clone + Send + Hash + 'static,
Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let mut populations = Vec::new();
    let mut handles = Vec::new();
    for _ in 0..sub_populations_per_level {
        let data_copy = data.clone();
        handles.push(thread::spawn(move || population_from_multilevel_sub_populations(levels - 1, sub_populations_per_level, data_copy, number_of_genes, initial_population_size, iterations_on_each_population, get_score_index)));
    }

    for handle in handles {
        populations.push(handle.join().unwrap());
    }

    populations_from_existing_multillevel(populations, 1, sub_populations_per_level, data, iterations_on_each_population, get_score_index)
}

fn populations_from_existing_multillevel<Gene, IndexFunction, Data>(
    mut populations: Vec<Population<Gene>>,
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction
) -> Population<Gene>
where 
Gene: Clone + Hash + Send + 'static,
Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let cull_proportion = 1.0 - 1.0 / sub_populations_per_level as f64;
    for level in (0..levels).rev() {
        let number_of_new_populations = sub_populations_per_level.pow(level);
        let mut new_populations = Vec::new();
        for _ in 0..number_of_new_populations {
            let mut population = Population::new_empty(false);
            for _ in 0..sub_populations_per_level {
                let agents = populations.pop().unwrap().get_agents().clone();
                for (score, agent) in agents {
                    population.insert(score, agent);
                }
            }
            new_populations.push(
                cull_lowest_agents(
                    run_iterations(population, iterations_on_each_population, data, get_score_index),
                    cull_proportion,
                    1
                )
            );
        }

        populations = new_populations;
    }

    populations.pop().unwrap()
}

fn run_iterations<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    iterations: usize,
    data: &Data,
    get_score_index: &'static IndexFunction
) -> Population<Gene>
where
Gene: Clone + Hash + Send + 'static,
Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    for _ in 0..iterations {
        population = mutate_some_agents(population, 0.1, 1, data, get_score_index, 25, 1);
        population = mate_alpha_agents(population, 0.2, 1, data, get_score_index, 25, 1);
        population = mate_some_agents(population, 0.5, 1, data, get_score_index, 25, 1);
        population = cull_lowest_agents(population, 0.02, 1);
    }

    population
}
