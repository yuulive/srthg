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

extern crate xu;

use xu::agent::Agent;
use xu::manager::create_manager;
use xu::fitness::ScoreError;

fn main() {

    let mut manager = create_manager(fitness_function, 0);
    manager.set_number_of_genes(5, true);
    manager.run(1250);
    let agents = manager.get_population().get_agents();

    println!("Population: {}", agents.len());

    let mut viewing = 10;
    for (score_index, agent) in agents.iter().rev() {
        println!("Score: {}", score_index);
        println!("{:?}", agent.get_genes());

        viewing -= 1;
        if viewing == 0 {
            break;
        }
    }
}

fn fitness_function(agent: &Agent<u8>, _data: &u8) -> Result<u64, ScoreError> {
    let mut score = 0;

    for gene in agent.get_genes() {
        score += *gene as u64;
    }

    Ok(score)
}