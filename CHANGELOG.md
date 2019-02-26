# Changelog

## 0.2.2

* Added ScoreProvider trait that allows defining more customised ways of determining scores (perhaps for optimisation or interfacing with other systems).
* Added manager::create_manager function for simple way to still get the manager if the generic score provider is sufficient.
* The scoring function is now referred to as the fitness function.
* The fitness function returns are result rather than the score directly.
* Added the fitness::ScoreError struct for fitness function errors.

## 0.2.1

* Removed the 'multilevel' functions from th evolution module.
* Added the ability to configure number of iterations per cycle in the manager.
* Encapsulated scoring of genes within a struct called ScoreProvider
* Removed threading from operations internally. This may be added back in another form.

## 0.2.0

* Added manager module, allowing for only setting the necessary parameters before running, the rest being set to defaults which can be overwritten.
* Added a simpler API for defining operations (the 'new' method).
* Added a simpler API for defining selections (the 'new' method).

## 0.1.1

* Changed the evolution::run_iterations function to be public.
* Added various examples
* Added information on getting started to the readme.