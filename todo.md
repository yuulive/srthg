To do:

_Tests_
* Unit tests for operations
* Unit tests for evolution functions
* Integration tests. Evolution tests will just about be this. Perhaps have simple use case tests in addition however.

_Docs_
* Function docs
* Read me (about and brief summary of what to do)
* Getting started doc (more detail on what to do, but still concise)
* Cargo.toml metadata
* License

_API_
* Go through API guidelines: https://rust-lang-nursery.github.io/api-guidelines/about.html
* Review evolution functions

_Development_
* Intelligent configuration. Runs test populations and modifies parameters in attempt to find optimum. May have suggestions
for the developer regarding their side, e.g. returning larger score indexes.
* Long running management. In cases where processes will run for a long time, could spawn threads of populations
and join them when they are ready.
* Command-line runner. So that a developer can simply code the minimum and then link to a command line output, which may also
bring in the above. Not sure if this will belong in this library or an associated library.
* Even more likely a separate library: helper functions to more quickly put together various use cases, such as those involving
numeric trends over time or something like the travelling salesman problem.
