use super::agent::{Agent, mate};
use super::population::Population;
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::thread;
use std::marker::{Send, PhantomData};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub enum OperationType {
    Mutate,
    Mate,
    Cull
}

#[derive(Clone, Copy)]
pub enum SelectionType {
    RandomAny,
    HighestScore,
    LowestScore
}

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

    pub fn selection_type(&self) -> SelectionType {
        self.selection_type
    }

    pub fn proportion(&self) -> f64 {
        self.proportion
    }

    pub fn preferred_minimum(&self) -> usize {
        self.preferred_minimum
    }

    pub fn agents <'a, Gene> (&self, population: &'a Population<Gene>) -> BTreeMap<isize, &'a Agent<Gene>>
    where
    Gene: Clone
    {
        match self.selection_type {
            SelectionType::RandomAny => get_random_subset(population.get_agents(), self.proportion, self.preferred_minimum),
            _ => get_random_subset(population.get_agents(), self.proportion, self.preferred_minimum)
        }
    }

    pub fn count <Gene> (&self, population: &Population<Gene>) -> usize {
        rate_to_number(population.len(), self.proportion, self.preferred_minimum)
    }
}

#[derive(Clone)]
pub struct Operation <Gene, Data, IndexFunction>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    selection: Selection,
    operation_type: OperationType,
    get_score_index: &'static IndexFunction,
    offset: isize,
    threads: usize,
    gene: PhantomData<Gene>,
    data: PhantomData<Data>
}

impl <Gene, Data, IndexFunction> Operation <Gene, Data, IndexFunction>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    pub fn new(
        selection: Selection,
        operation_type: OperationType,
        get_score_index: &'static IndexFunction,
        offset: isize,
        threads: usize
        ) -> Self {
        Self {
            selection: selection,
            operation_type: operation_type,
            get_score_index: get_score_index,
            offset: offset,
            threads: threads,
            gene: PhantomData,
            data: PhantomData
        }
    }

    pub fn run (&self, population: Population<Gene>, data: &Data) -> Population<Gene>
    {
        match self.operation_type {
            OperationType::Mutate => mutate_agents(population, self.selection, data, self.get_score_index, self.offset, self.threads),
            OperationType::Mate => mate_agents(population, self.selection, data, self.get_score_index, self.offset, self.threads),
            OperationType::Cull => cull_agents(population, self.selection)
        }
    }
}

pub fn mutate_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    selection: Selection,
    data: &Data,
    get_score_index: &'static IndexFunction,
    offset: isize,
    threads: usize
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let groups = arrange_agents_into_groups(
        selection.agents(&population),
        threads
    );

    if threads > 1 {
        let mut handles = Vec::new();

        for agents in groups {           
            let data_copy = data.clone();
            handles.push(thread::spawn(move || get_mutated_agents(agents, &data_copy, get_score_index, offset)));
        }

        for handle in handles {
            let children = handle.join().unwrap();
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    } else {
        for agents in groups {
            let children = get_mutated_agents(agents, data, get_score_index, offset);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

pub fn mate_agents<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    selection: Selection,
    data: &Data,
    get_score_index: &'static IndexFunction,
    offset: isize,
    threads: usize
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
{
    let groups = arrange_pairs_into_groups(
        create_random_pairs(
            selection.agents(&population)
        ),
        threads
    );

    if threads > 1 {
        let mut handles = Vec::new();
        {       
            for pairs in groups {           
                let data_copy = data.clone();
                handles.push(thread::spawn(move || create_children(pairs, &data_copy, get_score_index, offset)));
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
            let children = create_children(pairs, data, get_score_index, offset);
            for (score_index, agent) in children {
                population.insert(score_index, agent);
            }
        }
    }

    population
}

pub fn cull_agents<Gene>(
    mut population: Population<Gene>,
    selection: Selection,
) -> Population<Gene>
{
    let keys: Vec<isize> = population.get_agents().keys().map(|k| *k).collect();
    let cull_number = selection.count(&population);
    if cull_number >= keys.len() {
        return population;
    }
    
    match selection.selection_type() {
        SelectionType::LowestScore => population.cull_all_below(keys[cull_number]),
        SelectionType::HighestScore => (),
        SelectionType::RandomAny => ()
    };
    population
}

fn get_mutated_agents<Gene, IndexFunction, Data>(
    agents: Vec<Agent<Gene>>,
    data: &Data,
    get_score_index: IndexFunction,
    offset: isize
) -> Vec<(isize, Agent<Gene>)>
where Standard: Distribution<Gene>,
Gene: Clone + Hash,
IndexFunction: Fn(&Agent<Gene>, &Data) -> isize
{
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    for mut agent in agents {
        agent.mutate();
        let score_index = get_score_index(&agent, data) + rng.gen_range(-offset, offset);
        children.push((score_index, agent));
    }
    children
}

fn create_children<Gene, IndexFunction, Data>(
    pairs: Vec<(Agent<Gene>, Agent<Gene>)>,
    data: &Data,
    get_score_index: &'static IndexFunction,
    offset: isize
) -> Vec<(isize, Agent<Gene>)>
where 
Gene: Clone + Hash,
IndexFunction: Fn(&Agent<Gene>, &Data) -> isize
{
    let mut rng = rand::thread_rng();
    let mut children = Vec::new();
    for (parent_one, parent_two) in pairs {
        let child = mate(&parent_one, &parent_two);
        let score_index = get_score_index(&child, data) + rng.gen_range(-offset, offset);;
        children.push((score_index, child));
    }
    return children;
}

fn get_random_subset<Gene>(
    agents: &BTreeMap<isize, Agent<Gene>>,
    rate: f64,
    preferred_minimum: usize
) -> BTreeMap<isize, &Agent<Gene>>
where Gene: Clone
{
    let number = rate_to_number(agents.len(), rate, preferred_minimum);
    let keys: Vec<isize> = agents.keys().map(|k| *k).collect();
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
    agents:  BTreeMap<isize, &Agent<Gene>>,
    threads: usize
) -> Vec<Vec<Agent<Gene>>>
where Gene: Clone {
    let mut groups = vec![Vec::new(); threads];
    let mut count = 0;
    for (_score, agent) in agents {
        groups[count % threads].push(agent.clone());
        count += 1;
    }

    groups
}

fn create_random_pairs<Gene>(
    agents: BTreeMap<isize, &Agent<Gene>>,
) -> Vec<(Agent<Gene>, Agent<Gene>)> 
where
Gene: Clone
{
    let keys: Vec<&isize> = agents.keys().collect();
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
    let keys: Vec<isize> = population.get_agents().keys().map(|k| *k).collect();
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

}