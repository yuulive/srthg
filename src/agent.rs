use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Clone)]
pub struct Agent <Gene: Clone> {
    genes: Vec<Gene>,
    hash: u64
}

impl <Gene> Agent<Gene> where Standard: Distribution<Gene>, Gene: Clone + PartialEq + Hash {
    pub fn new(number_of_genes: usize) -> Self {
        let mut genes = Vec::with_capacity(number_of_genes);
        for _ in 0..number_of_genes {
            genes.push(rand::random());
        }

        let mut s = DefaultHasher::new();
        genes.hash(&mut s);
        let hash = s.finish();

        Self {
            genes: genes,
            hash: hash
        }
    }

    pub fn get_genes(&self) -> &Vec<Gene> {
        return &self.genes;
    }

    pub fn crossover_some_genes(&mut self, other: &Self) {
        let mut rng = rand::thread_rng();
        let mut gene_count = self.genes.len();
        if gene_count > other.genes.len() {
            gene_count = other.genes.len();
        }
        let crossover_point = rng.gen_range(0, gene_count);

        self.genes.truncate(crossover_point);
        let mut other_genes = other.get_genes().clone();
        other_genes.drain(..crossover_point);
        self.genes.append(&mut other_genes);

        let mut s = DefaultHasher::new();
        self.genes.hash(&mut s);
        self.hash = s.finish();
    }

    pub fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        let gene_count = self.genes.len();

        for _ in 0..5 {
           self.genes.remove(rng.gen_range(0, gene_count));
           self.genes.insert(rng.gen_range(0, gene_count - 1), rand::random());
        }

        let mut s = DefaultHasher::new();
        self.genes.hash(&mut s);
        self.hash = s.finish();
    }

    pub fn has_same_genes(&self, other: &Self) -> bool {
        self.hash == other.hash
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }
}

pub fn mate <Gene> (parent1: &Agent<Gene>, parent2: &Agent<Gene>) -> Agent<Gene> 
where Standard: Distribution<Gene>, Gene: Clone + PartialEq + Hash {
    let mut child = parent1.clone();

    child.crossover_some_genes(parent2);

    return child;
}