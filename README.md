# Aristeia

[![Build Status](https://travis-ci.org/brendancox/aristeia.svg?branch=master)](https://travis-ci.org/brendancox/aristeia) 

An evolutionary computation library written in Rust.

## What can this library do?

Aristeia provides the generic logic required for genetic algorithms, allowing you to focus on the code specific to your particular use case.

It is still at an early stage so expect the API to change but also for improvements to be made.

## Getting started

Create a new project and add the following to the cargo.toml file:

```toml
[dependencies]
aristeia = "0.2.1"
```

In main.rs, start off with the following:

```rust
extern crate aristeia;

use aristeia::agent::Agent;
use aristeia::manager::Manager;
```

In the above code, we import the Manager, which will run the genetic algorithm system. We also import Agent so that we can investigate the 'fittest' set of genes after running.

Now, inside you main() function, delete the default code in there and let's add the code to create and run the manager:

```rust
let mut manager = Manager::new(get_score_index, 0);
manager.set_number_of_genes(5, true);
manager.run(1250);;
```

We've created a new manager, passing in a function called get_score_index which is what determines the fitness score of our agents. We define this function later in this example. We also pass in 0 as the second argument, which is for additional data. We aren't using the data parameter in this example, but you can look at some of examples in this library to see other ways that data can be used.

We also set the number of genes that each agent should have. The second argument is for saying whether agents have to have that number of genes, or whether it can vary a bit if we aren't getting any good scores. However, that 'varying' functionality has not yet been implemented.

Lastly, we run the system, specifying a score that the highest in the population must be greater than in order to complete the run. If we were to set that score too high, the system would currently run forever (until you press Ctrl+C to stop the program).

Once the run is complete, we'll want to get the agents and see what genes they had. Below your code for running the manager, add the following:

```rust
let agents = manager.get_population().get_agents();
```

To finish off the main function, we'll want to see what the highest scored agents are. Add this:

```rust
let mut viewing = 10;
for (score_index, agent) in population.get_agents().iter().rev() {
    println!("Score: {}", score_index);
    println!("{:?}", agent.get_genes());

    viewing -= 1;
    if viewing == 0 {
        break;
    }
}
```

So the last thing to do is add that scoring function. We're just wanting a quick example here, so we're just going to make the genes 'u8' (unsigned 8-bit integers), and the higher they are, the higher the score.

Add the following below your main function:

```rust
fn get_score_index(agent: &Agent<u8>, _data: &u8) -> isize {
    let mut score = 0;

    for gene in agent.get_genes() {
        score += *gene as isize;
    }

    score
}
```

Now run your code with ```cargo run```.

You'll get a list of the top scores in the population, along with sets of 5 integers for each, which represent their 'genes'.

Have a look in the examples directory of this library. The example described above can be found in simplest.rs.

## License

Aristeia is licensed under the Apache License, Version 2.0.

