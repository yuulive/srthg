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

use aristeia::evolution::threaded_population_from_multilevel_sub_populations;
use self::Cities::{
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
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use aristeia::operations::{
    Operation,
    OperationType,
    Selection,
    SelectionType,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Cities {
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
    let mut data = HashMap::new();
    for city in all_cities {
        for other in &cities_clone {
            if city != *other {
                data.insert((city.clone(), other.clone()), distance_between_points(get_coordinates(&city), get_coordinates(&other)));
            }
        }
    }

    let operations = vec![
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.1, 1),
            OperationType::Mutate,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::HighestScore, 0.2, 1),
            OperationType::Crossover,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.5, 1),
            OperationType::Crossover,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::LowestScore, 0.02, 1),
            OperationType::Cull,
            25,
            1)
    ];

    let population = threaded_population_from_multilevel_sub_populations(2, 2, &data, 10, 20, 50, get_score_index, &operations);
    let agents = population.get_agents();

    println!("Population: {}", agents.len());
    println!("Duration: {}", now.elapsed().as_secs() as f64 + now.elapsed().subsec_nanos() as f64 * 1e-9);

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

impl Distribution<Cities> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cities {
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

fn distance_between_points(first: (f64, f64), second: (f64, f64)) -> f64 {
    let (x1, y1) = first;
    let (x2, y2) = second;
    let distance = ((x2 - x1).powi(2)  - (y2 - y1).powi(2)).abs().sqrt();
    distance
}

fn get_coordinates(city: &Cities) -> (f64, f64) {
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

fn get_distance(agent: &Agent<Cities>, data: &HashMap<(Cities, Cities), f64>) -> f64 {
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

fn get_score_index(agent: &Agent<Cities>, data: &HashMap<(Cities, Cities), f64>) -> isize {
    let distance = get_distance(agent, data);

    let mut repeats = 0;
    let mut cities = HashSet::new();
    for city in agent.get_genes() {
        if !cities.insert(city) {
            // False returned if HashSet did have value.
            repeats += 1;
        }
    }

    let score = (6.0 * agent.get_genes().len() as f64 - distance) * 100.0 * (1.0 - repeats as f64 * 0.1);

    return score as isize;
}
