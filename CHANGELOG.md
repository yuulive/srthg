# Changelog

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