# Aristeia

[![Build Status](https://travis-ci.org/brendancox/aristeia.svg?branch=master)](https://travis-ci.org/brendancox/aristeia) 

An evolutionary computation library written in Rust.

## What can this library do?

Aristeia provides the generic logic required for genetic algorithms, allowing you to focus on the code specific to your particular use case.

It is still at an early stage so expect the API to change but also for improvements to be made.

There is currently a very basic approach to providing threading at this point. 

The API requires defining values that, without guidance or experimentation, would be hard to determine. But improving this is precisely something that's in scope for work coming up. And guidance can be found below and by looking at examples in the examples directory of this library.

## Getting started

Create a new project and add the following to the cargo.toml file:

```toml
[dependencies]
aristeia = "0.1.1"
```

In main.rs, start off with the following to get all the types you'll need for this example:

```rust
extern crate aristeia;

use aristeia::evolution::run_iterations;
use aristeia::agent::Agent;
use aristeia::population::Population;
use aristeia::operations::{
    Operation,
    OperationType,
    Selection,
    SelectionType,
};
```

Now, inside you main() function, delete the default code in there and add the following to specify what should happen for each
generation as your algorithm evolves:

```rust
    let operations = vec![
        Operation::with_values(
            Selection::with_values(SelectionType::RandomAny, 0.1, 1),
            OperationType::Mutate,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::HighestScore, 0.1, 1),
            OperationType::Crossover,
            25,
            1),
        Operation::with_values(
            Selection::with_values(SelectionType::LowestScore, 0.1, 1),
            OperationType::Cull,
            25,
            1)
    ];
```

So what have we asked for above? First of all, there will be 3 operations per generation. 

1st - Mutation (hence ```OperationType::Mutate```), which will be done on a random subset of the population (hence ```SelectionType::RandomAny``` in the Selection). 
2nd - Crossover (```OperationType::Crossover```), which will be done on a proportion of the highest score population (```SelectionType::HighestScore```).
3rd - Cull (```OperationType::Cull```), where the lowest scored individuals will be culled (```SelectionType::LowestScore```).

I'll leave you to look at the function docs for what the various other arguments used are all about rather than go into that detail here.

Below where you've defined those operations, let's create the population:

```rust
let population = Population::new(100, 5, false, &0, get_score_index);
```

The key thing is we're creating a population of 100 individuals (named agents in this library) and each will have 5 genes. That last argument is also important, that's specifying a function we will use to score the set of genes that an agent has. We'll get back to that later.

First though, we'll finish the main function, add the call to run iterations, this where the library does all its work:

```rust
let population = run_iterations(population, 50, &0, &operations, get_score_index);
```

Following the above, the population would have been through the operations we defined 50 times. To finish off the main function, we'll want to see what the highest scored (or to use evolution-based terms, the 'fittest') agents are. Add this:

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

Have a look in the examples directory of this library. You'll find the code we've run through in simplest.rs.

## License

Aristeia is licensed under the Apache license version 2.0.

