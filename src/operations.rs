use super::agent::{Agent, mate};
use super::population::Population;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::thread;
use std::marker::Send;

pub fn mutate_some_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    rate: f64,
    data: &Data,
    get_score_index: &'static IndexFunction,
    threads: usize) -> Population<Gene>
  where Standard: Distribution<Gene>,
        Gene: Clone + PartialEq + Hash + Send + 'static,
        IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
        Data: Clone + Send + 'static
        {

    let clones = remove_random_agents_into_groups(&mut population, threads, rate);

    if threads > 1 {
        let mut handles = Vec::new();

        for cloned_agents in clones {           
            let data_copy = data.clone();
            handles.push(thread::spawn(move || get_mutated_agents(cloned_agents, data_copy, get_score_index)));
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    } else {
        for cloned_agents in clones {
            let children = get_mutated_agents(cloned_agents, data.clone(), get_score_index);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

fn get_mutated_agents<Gene, IndexFunction, Data>(
    agents: Vec<(isize, Agent<Gene>)>,
    data: Data,
    get_score_index: IndexFunction
) -> Vec<(isize, Agent<Gene>)>
where Standard: Distribution<Gene>,
Gene: Clone + PartialEq + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static
 {
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    for (_key, mut agent) in agents {
        agent.mutate();
        let score_index = get_score_index(&agent, &data) + rng.gen_range(-25, 25);
        children.push((score_index, agent));
    }
    children
}

pub fn mate_some_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    rate: f64,
    data: &Data,
    get_score_index: &'static IndexFunction,
    threads: usize,
    max_diff: isize
    ) -> Population<Gene>
 where Standard: Distribution<Gene>, 
 Gene: Clone + PartialEq + Hash + Send + 'static, 
 IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
 Data: Clone + Send + 'static
   {

    let clones_one = make_vec_of_cloned_agents_for_threads(&mut population, threads, rate / 2.0);
    let mut clones_two = make_vec_of_cloned_agents_for_threads(&mut population, threads, rate / 2.0);

    if threads > 1 {
        let mut handles = Vec::new();
        {       
            for cloned_agents in clones_one {           
                let data_copy = data.clone();
                let other_agents = clones_two.pop();
                if other_agents.is_some() {
                    let mut other_agents = other_agents.unwrap();

                    handles.push(thread::spawn(move || create_children(cloned_agents, other_agents, &data_copy, get_score_index, max_diff)));
                }
            }
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    } else {

        for cloned_agents in clones_one { 
            let data_copy = data.clone();
            let other_agents = clones_two.pop();
            if other_agents.is_some() {
                let mut other_agents = other_agents.unwrap();
                let children = create_children(cloned_agents, other_agents, &data_copy, get_score_index, max_diff);
                for (score_index, agent) in children {
                    population.insert(score_index, agent);
                }
            }
        }
    }

    population
}

fn create_children<Gene, IndexFunction, Data>(
    parent1_agents: Vec<(isize, Agent<Gene>)>,
    parent2_agents: Vec<(isize, Agent<Gene>)>,
    data: &Data,
    get_score_index: &'static IndexFunction,
    max_diff: isize
    ) -> Vec<(isize, Agent<Gene>)>
 where Standard: Distribution<Gene>, 
 Gene: Clone + PartialEq + Hash + Send + 'static, 
 IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
 Data: Clone + Send 
  {
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    let mut other_parents = parent2_agents.iter();
    for (key, agent) in parent1_agents {
        let other = other_parents.next();
        if other.is_some() {
            let (other_key, other_agent) = other.unwrap();
            let diff = key - other_key;
            if diff.abs() < max_diff {
                if !agent.has_same_genes(&other_agent) {
                    let child = mate(&other_agent, &agent);
                    let score_index = get_score_index(&child, data) + rng.gen_range(-25, 25);;
                    children.push((score_index, child));
                }
            }
        }
    }
    return children;
}

fn make_vec_of_cloned_agents_for_threads<Gene>(
        population: &mut Population<Gene>,
        threads: usize,
        rate: f64
    ) -> Vec<Vec<(isize, Agent<Gene>)>>
where Gene: Clone + PartialEq + Hash, Standard: Distribution<Gene> {
    let mut clones = vec![Vec::new(); threads];
    for count in 0..rate_to_number(population.len(), rate) {
        let key = population.get_random_score();
        let agent = population.get(key);
        if agent.is_some() {
            clones[count % threads].push((key, agent.unwrap().clone()));
        }
    }

    clones
}

fn remove_random_agents_into_groups<Gene>(
        population: &mut Population<Gene>,
        threads: usize,
        rate: f64
    ) -> Vec<Vec<(isize, Agent<Gene>)>> 
where Gene: Clone + PartialEq + Hash, Standard: Distribution<Gene> {
    let mut clones = vec![Vec::new(); threads];
    for count in 0..rate_to_number(population.len(), rate) {
        let key = population.get_random_score();
        let agent = population.remove(key);
        if agent.is_some() {
            clones[count % threads].push((key, agent.unwrap()));
        }
    }
    clones
}

pub fn mate_alpha_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    rate: f64,
    data: &Data,
    get_score_index: &'static IndexFunction,
    threads: usize,
    max_diff: isize) -> Population<Gene>
 where Standard: Distribution<Gene>, 
 Gene: Clone + PartialEq + Hash + Send + 'static, 
 IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
 Data: Clone + Send + 'static
   {
    let keys: Vec<isize> = population.get_agents().keys().map(|k| *k).collect();
    let mate_number = rate_to_number(keys.len(), rate);
    if mate_number >= keys.len() {
        return population;
    }
    let mut top_agents = population.get_agents().clone();
    let top_agents = top_agents.split_off(&keys[mate_number]);
    let mut top_pop = Population::new_empty(false);
    top_pop.set_agents(top_agents);

    let clones_one = make_vec_of_cloned_agents_for_threads(&mut top_pop, threads, rate / 2.0);
    let mut clones_two = make_vec_of_cloned_agents_for_threads(&mut top_pop, threads, rate / 2.0);

    if threads > 1 {
        let mut handles = Vec::new();
        {       
            for cloned_agents in clones_one {           
                let data_copy = data.clone();
                let other_agents = clones_two.pop();
                if other_agents.is_some() {
                    let mut other_agents = other_agents.unwrap();

                    handles.push(thread::spawn(move || create_children(cloned_agents, other_agents, &data_copy, get_score_index, max_diff)));
                }
            }
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }

    } else {

        for cloned_agents in clones_one { 
            let data_copy = data.clone();
            let other_agents = clones_two.pop();
            if other_agents.is_some() {
                let mut other_agents = other_agents.unwrap();
                let children = create_children(cloned_agents, other_agents, &data_copy, get_score_index, max_diff);
                for (score_index, agent) in children {
                    population.insert(score_index, agent);
                }
            }
        }
    }

    population
}

pub fn cull_lowest_agents<Gene>(mut population: Population<Gene>, rate: f64) -> Population<Gene>
 where Gene: Clone + PartialEq + Hash, Standard: Distribution<Gene> {
    let keys: Vec<isize> = population.get_agents().keys().map(|k| *k).collect();
    let cull_number = rate_to_number(keys.len(), rate);
    if cull_number >= keys.len() {
        return population;
    }
    population.cull_all_below(keys[cull_number]);
    population
}

fn rate_to_number(population: usize, rate: f64) -> usize {
    if population == 0 {
        return 0;
    }
    let mut number = (population as f64 * rate) as usize;
    if number == 0 {
        number = 1;
    }

    number
}