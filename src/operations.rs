use super::agent::{Agent, mate};
use super::population::Population;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::thread;
use std::marker::Send;
use std::collections::BTreeMap;

pub fn mutate_some_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    rate: f64,
    data: &Data,
    get_score_index: &'static IndexFunction,
    threads: usize
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let groups = arrange_agents_into_groups(
        get_random_subset(population.get_agents().clone(), rate),
        threads
    );

    if threads > 1 {
        let mut handles = Vec::new();

        for agents in groups {           
            let data_copy = data.clone();
            handles.push(thread::spawn(move || get_mutated_agents(agents, data_copy, get_score_index)));
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    } else {
        for agents in groups {
            let children = get_mutated_agents(agents, data.clone(), get_score_index);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

fn get_mutated_agents<Gene, IndexFunction, Data>(
    agents: Vec<Agent<Gene>>,
    data: Data,
    get_score_index: IndexFunction
) -> Vec<(isize, Agent<Gene>)>
where Standard: Distribution<Gene>,
Gene: Clone + Hash,
IndexFunction: Fn(&Agent<Gene>, &Data) -> isize
{
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    for mut agent in agents {
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
    threads: usize
    ) -> Population<Gene>
 where
 Gene: Clone + Hash + Send + 'static, 
 IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
 Data: Clone + Send + 'static
{
    let groups = arrange_pairs_into_groups(
        create_random_pairs(
            get_random_subset(population.get_agents().clone(), rate / 2.0),
            get_random_subset(population.get_agents().clone(), rate / 2.0)
        ),
        threads
    );

    if threads > 1 {
        let mut handles = Vec::new();
        {       
            for pairs in groups {           
                let data_copy = data.clone();
                handles.push(thread::spawn(move || create_children(pairs, &data_copy, get_score_index)));
            }
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    } else {

        for pairs in groups { 
            let children = create_children(pairs, &data.clone(), get_score_index);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

fn create_children<Gene, IndexFunction, Data>(
    pairs: Vec<(Agent<Gene>, Agent<Gene>)>,
    data: &Data,
    get_score_index: &'static IndexFunction
) -> Vec<(isize, Agent<Gene>)>
where 
Gene: Clone + Hash,
IndexFunction: Fn(&Agent<Gene>, &Data) -> isize
{
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    for (parent_one, parent_two) in pairs {
        let child = mate(&parent_one, &parent_two);
        let score_index = get_score_index(&child, data) + rng.gen_range(-25, 25);;
        children.push((score_index, child));
    }
    return children;
}

fn get_random_subset<Gene>(
    agents: BTreeMap<isize, Agent<Gene>>,
    rate: f64
) -> BTreeMap<isize, Agent<Gene>>
where Gene: Clone
{
    let number = rate_to_number(agents.len(), rate);
    let keys: Vec<isize> = agents.keys().map(|k| *k).collect();
    let mut rng = rand::thread_rng();
    let mut subset = BTreeMap::new();
    for _ in 0..number {
        let key = keys[rng.gen_range(0, keys.len())];
        let agent = agents.get(&key);
        if agent.is_some() {
            subset.insert(key, agent.unwrap().clone());
        }
    }

    subset
}

fn arrange_pairs_into_groups<Gene>(
    pairs: Vec<(Agent<Gene>, Agent<Gene>)>,
    threads: usize
) -> Vec<Vec<(Agent<Gene>, Agent<Gene>)>>
where
Gene: Clone
{
    let mut groups = vec![Vec::new(); threads];
    let mut count = 0;
    for pair in pairs {
        groups[count % threads].push(pair);
        count += 1;
    }

    groups
}

fn arrange_agents_into_groups<Gene>(
    agents:  BTreeMap<isize, Agent<Gene>>,
    threads: usize
) -> Vec<Vec<Agent<Gene>>>
where Gene: Clone {
    let mut groups = vec![Vec::new(); threads];
    let mut count = 0;
    for (_score, agent) in agents {
        groups[count % threads].push(agent);
        count += 1;
    }

    groups
}

fn create_random_pairs<Gene>(
    one: BTreeMap<isize, Agent<Gene>>,
    two: BTreeMap<isize, Agent<Gene>>
) -> Vec<(Agent<Gene>, Agent<Gene>)> 
where
Gene: Clone
{
    let one_keys: Vec<isize> = one.keys().map(|k| *k).collect();
    let two_keys: Vec<isize> = two.keys().map(|k| *k).collect();
    let mut rng = rand::thread_rng();
    let mut pairs = Vec::new();
    let mut count = one_keys.len();
    if one_keys.len() > two_keys.len() {
        count = two_keys.len();
    }

    for _ in 0..count {
        let one_key = one_keys[rng.gen_range(0, one_keys.len())];
        let two_key = two_keys[rng.gen_range(0, two_keys.len())];

        let one_agent = one.get(&one_key);
        let two_agent = two.get(&two_key);
        if one_agent.is_some() && two_agent.is_some() {
            if !one_agent.unwrap().has_same_genes(two_agent.unwrap()) {
                pairs.push((one_agent.unwrap().clone(), two_agent.unwrap().clone()));
            }
        }
    }

    pairs
}

pub fn mate_alpha_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    rate: f64,
    data: &Data,
    get_score_index: &'static IndexFunction,
    threads: usize
) -> Population<Gene>
 where 
 Gene: Clone + Hash + Send + 'static, 
 IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
 Data: Clone + Send + 'static
   {
    let keys: Vec<isize> = population.get_agents().keys().map(|k| *k).collect();
    let mate_number = rate_to_number(keys.len(), rate);
    if mate_number >= keys.len() {
        return population;
    }

    let top_agents = population.get_agents().clone().split_off(&keys[mate_number]);

    let groups = arrange_pairs_into_groups(
        create_random_pairs(
            get_random_subset(top_agents.clone(), rate / 2.0),
            get_random_subset(top_agents, rate / 2.0)
        ),
        threads
    );

    if threads > 1 {
        let mut handles = Vec::new();
        {       
            for pairs in groups {           
                let data_copy = data.clone();
                handles.push(thread::spawn(move || create_children(pairs, &data_copy, get_score_index)));
            }
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }

    } else {

        for pairs in groups { 
            let children = create_children(pairs, &data.clone(), get_score_index);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

pub fn cull_lowest_agents<Gene>(
    mut population: Population<Gene>,
    rate: f64
) -> Population<Gene>
{
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

#[cfg(test)]
mod tests {
    use super::*;

}