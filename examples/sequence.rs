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

use self::Gene::{
    MovePointerLeft,
    MovePointerRight,
    IncreaseValueByOne,
    DecreaseValueByOne,
    CopyValueFromLeft,
    CopyValueFromRight
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::time::Instant;
use aristeia::agent::{Agent};

use aristeia::manager::Manager;


#[derive(Clone, PartialEq, Hash)]
enum Gene {
    MovePointerLeft,
    MovePointerRight,
    IncreaseValueByOne,
    DecreaseValueByOne,
    CopyValueFromLeft,
    CopyValueFromRight
}

pub fn main() {
    let now = Instant::now();

    let data = vec![0; 10];

    let mut manager = Manager::new(get_score_index, data.clone());
    manager.set_number_of_genes(30, false);
    manager.run(9999);
    let agents = manager.get_population().get_agents();

    println!("Duration: {}", now.elapsed().as_secs() as f64 + now.elapsed().subsec_nanos() as f64 * 1e-9);
    println!("Population: {}", agents.len());

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
        println!("{}", score_index);
        println!("{:?}", get_processed_data(agent.get_genes(), &data));
    }
}

fn get_processed_data(genes: &Vec<Gene>, data: &Vec<u8>) -> Vec<u8> {
    let mut copy = data.clone();
    let mut pointer = 0;
    for gene in genes {
        match gene {
            MovePointerLeft => move_pointer_left(&mut pointer, &mut copy),
            MovePointerRight => move_pointer_right(&mut pointer, &mut copy),
            IncreaseValueByOne => increase_value_by_one(&mut pointer, &mut copy),
            DecreaseValueByOne => decrease_value_by_one(&mut pointer, &mut copy),
            CopyValueFromLeft => copy_value_from_left(&mut pointer, &mut copy),
            CopyValueFromRight => copy_value_from_right(&mut pointer, &mut copy),
        }
    }

    return copy;
}

fn move_pointer_left(pointer: &mut usize, _data: &mut Vec<u8>) {
    if *pointer == 0 {
        return;
    }

    *pointer -= 1;
}

fn move_pointer_right(pointer: &mut usize, data: &mut Vec<u8>) {
    if *pointer == data.len() - 1 {
        return;
    }

    *pointer += 1;
}

fn increase_value_by_one(pointer: &mut usize, data: &mut Vec<u8>) {
    data[*pointer] += 1;
}

fn decrease_value_by_one(pointer: &mut usize, data: &mut Vec<u8>) {
    if data[*pointer] == 0 {
        return;
    }
    data[*pointer] -= 1;
}

fn copy_value_from_left(pointer: &mut usize, data: &mut Vec<u8>) {
    if *pointer == 0 {
        return;
    }

    data[*pointer] = data[*pointer-1];
}

fn copy_value_from_right(pointer: &mut usize, data: &mut Vec<u8>) {
    if *pointer == data.len() - 1 {
        return;
    }

    data[*pointer] = data[*pointer+1];
}

impl Distribution<Gene> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gene {
        match rng.gen_range(0, 6) {
            0 => MovePointerLeft,
            1 => MovePointerRight,
            2 => IncreaseValueByOne,
            3 => DecreaseValueByOne,
            4 => CopyValueFromLeft,
            _ => CopyValueFromRight
        }
    }
}

fn score_data(candidate: &Vec<u8>) -> isize {
    let mut score = 1.0;
    let candidate_length_squared = candidate.len().pow(2) as f64;
    let max_loss = 1.0 / candidate_length_squared;

    for i in 1..candidate.len() {
        let previous = candidate[i - 1];
        let expected = previous + 1;
        let value = candidate[i];
        if value == 0 {
            score = score - max_loss;
            continue;
        }

        let diff = value as f64 - expected as f64;
        score = score - (diff.abs() / candidate_length_squared);

        if score < 0.0 {
            score = 0.0;
            break;
        }
    }

    (score * 10000.0) as isize
}

fn get_score_index(agent: &Agent<Gene>, data: &Vec<u8>) -> isize {
    let processed = get_processed_data(agent.get_genes(), data);
    return score_data(&processed);
}
