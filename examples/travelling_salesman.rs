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
extern crate rand;

use aristeia::evolution::run_iterations;

// We do this so that we don't have to prefix the city names with City::
use self::City::{
    Wellington,
    PalmerstonNorth,
    NewPlymouth,
    Hastings,
    Gisborne,
    Taupo,
    Rotorua,
    Hamilton,
    Tauranga,
    Auckland
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use aristeia::agent::Agent;
use aristeia::population::Population;
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use aristeia::operations::{
    Operation,
    OperationType,
    Selection,
    SelectionType,
    ScoreProvider
};

// These are cities in the North Island of New Zealand.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum City {
    Wellington,
    PalmerstonNorth,
    NewPlymouth,
    Hastings,
    Gisborne,
    Taupo,
    Rotorua,
    Hamilton,
    Tauranga,
    Auckland
}

pub fn main() {
    // We'll be interested in how long the process takes.
    let now = Instant::now();

    let all_cities = vec![
        Wellington,
        PalmerstonNorth,
        NewPlymouth,
        Hastings,
        Gisborne,
        Taupo,
        Rotorua,
        Hamilton,
        Tauranga,
        Auckland
    ];
    let cities_clone = all_cities.clone();

    // There is always a data variable, for which the type is quite flexible and what you do with it is up to you.
    // In this case, we're using it cache the distances between each city rather than doing that calculation every time
    // we score a set of genes.
    let mut data = HashMap::new();
    for city in all_cities {
        for other in &cities_clone {
            if city != *other {
                data.insert((city.clone(), other.clone()), distance_between_points(get_coordinates(&city), get_coordinates(&other)));
            }
        }
    }

    // Here we define what happens for each "generation" of the process.
    let operations = vec![
        // We will mutate a random selection of 10% (that's the 0.1 in the Selection) of the population, but also a minimum of 1.
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.1, 1),
            OperationType::Mutate),
        // We will get highest scored 20% and randomly pair them, creating children with crossed over genes out of those.
        Operation::with_values(
            Selection::with_values(SelectionType::HighestScore, 0.2, 1),
            OperationType::Crossover),
        // We will take a random set of 50% of the population, randomly pair them and produce children with crossed over
        // genes out of those.
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.5, 1),
            OperationType::Crossover),
        // We will take the lowest 2% of the population and get rid of them. Note that just like in the previous operations,
        // the minimum is set to 1. So there'll always be at least 1 agent culled.
        Operation::with_values(
            Selection::with_values(SelectionType::LowestScore, 0.02, 1),
            OperationType::Cull)
    ];

    let mut score_provider = ScoreProvider::new(get_score_index, 25);

    // Create a population of 20 agents which each have a set of 10 randomly chosen genes.
    // We need to pass in the data as this is used for scoring the agents. 
    // We also pass in a reference to the scoring function defined towards the end of this file.
    let population = Population::new(20, 10, false, &data, &mut score_provider);

    // Now we run 50 iterations (or generations) on this population, meaning we run the operations we defined above
    // 50 times over. Again, we need the data and scoring function references as these are used for scoring new agents.
    let population = run_iterations(population, 50, &data, &operations, &mut score_provider);

    let agents = population.get_agents();

    println!("Population: {}", agents.len());
    println!("Duration: {}", now.elapsed().as_secs() as f64 + now.elapsed().subsec_nanos() as f64 * 1e-9);

    // This will the print the highest score and those that follow.
    let mut first = true;
    let mut first_score = 0;
    for (score_index, agent) in agents.iter().rev() {
        if first {
            first = false;
            first_score = *score_index;
        }
        if score_index < &(first_score - 20) {
            break;
        }
        println!("Score: {}", score_index);
        println!("{:?}", agent.get_genes());
    }
}

// We need to define how random values are generated for the genes.
// If you have a set of genes where you want the likelihood of each of them being returned to be equal,
// you would probably define it a lot like this.
impl Distribution<City> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> City {
        match rng.gen_range(0, 10) {
            0 => Wellington,
            1 => PalmerstonNorth,
            2 => NewPlymouth,
            3 => Hastings,
            4 => Gisborne,
            5 => Taupo,
            6 => Rotorua,
            7 => Hamilton,
            8 => Tauranga,
            _ => Auckland
        }
    }
}

// This just gives us the simple distance between 2 points on a 2d plane.
// I could have been more technically correct and used a formula that determines
// the distance between points on a globe (called the "haversine formula").
// I also could have got something like the driving distance or the travel time from somewhere,
// but for a demonstration of how the library works, I'm keeping things simple.
fn distance_between_points(first: (f64, f64), second: (f64, f64)) -> f64 {
    let (x1, y1) = first;
    let (x2, y2) = second;
    let distance = ((x2 - x1).powi(2)  - (y2 - y1).powi(2)).abs().sqrt();
    distance
}

// These are coordinates chosen from some point within or close to these cities.
fn get_coordinates(city: &City) -> (f64, f64) {
    match city {
        Wellington => (-41.30, 174.77),
        PalmerstonNorth => (-40.35, 175.61),
        NewPlymouth => (-39.07, 174.11),
        Hastings => (-39.64, 176.85),
        Gisborne => (-38.67, 178.01),
        Taupo => (-38.68, 176.08),
        Rotorua => (-38.14, 176.24),
        Hamilton => (-37.79, 175.28),
        Tauranga => (-37.69, 176.16),
        Auckland => (-36.85, 174.76)
    }
}

// Given an agent, and therefore it's full set of genes, or cities. Calculate the total distance
// travelled when going through all those cities in that order.
fn get_distance(agent: &Agent<City>, data: &HashMap<(City, City), f64>) -> f64 {
    let mut distance = 0.0;
    let mut previous_city = Wellington; // Initialise with something.
    let mut first = true;
    for gene in agent.get_genes() {
        if first {
            previous_city = gene.clone();
            first = false;
            continue;
        }

        if &previous_city != gene {
            distance += data.get(&(previous_city, gene.clone())).unwrap();
        }

        previous_city = gene.clone();
    }

    distance
}

// The scoring function used to determine the score on an agent, based on its genes.
fn get_score_index(agent: &Agent<City>, data: &HashMap<(City, City), f64>) -> isize {
    let distance = get_distance(agent, data);

    let mut repeats = 0;
    let mut cities = HashSet::new();
    for city in agent.get_genes() {
        if !cities.insert(city) {
            // False returned if HashSet did have value.
            repeats += 1;
        }
    }

    // To talk through this:
    // 6.0 is about the distance between the two furthest cities (using the coordinates as units, I'm not actually even bothering to convert to km or miles).
    // The above is multiplied by the length of the genes, because you could have a set that goes back and forth between the two furthest citis.
    // So that gives the longest possible distance, now subtract the distance calculated for the set of genes we're scoring.
    // Multiply by 100.0 - this is actually just to ensure the scores have a decent spread.
    // The last set of brackets is a penalty on the score for any cities visited twice, the idea of this example is that 
    // the salesman should be visiting each city once.
    let score = (6.0 * agent.get_genes().len() as f64 - distance) * 100.0 * (1.0 - repeats as f64 * 0.1);

    return score as isize;
}
