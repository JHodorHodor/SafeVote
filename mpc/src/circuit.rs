use crate::gate;

use std::iter::Iterator;

#[derive(Clone)]
pub struct Circuit<DataType: Clone> {
    pub gates: Vec<gate::Gate<DataType>>,
    roots: Vec<usize>,
    n_parties: u16,
}

impl<DataType: Clone> Circuit<DataType> {
    
    pub fn new(n_parties: u16) -> Self {
        Circuit {
            gates: vec![],
            roots: vec![], 
            n_parties
        }
    }

    pub fn set_roots(&mut self, gate_ids: Vec<usize>) {
        self.roots = gate_ids;
    }

    pub fn add(&mut self, gate: gate::Gate<DataType>) -> usize {
        self.gates.push(gate);
        self.gates.len() - 1
    }

    pub fn get_n_parties(&self) -> u16 {
        self.n_parties
    }

    pub fn get_roots(&self) -> Vec<usize> {
        self.roots.clone()
    }

    pub fn get_gate(&self, idx: usize) -> &gate::Gate<DataType> {
        &self.gates[idx]
    }

    pub fn get_gate_mut(&mut self, idx: usize) -> &mut gate::Gate<DataType> {
        &mut self.gates[idx]
    }

    pub fn traverse(&self) -> impl Iterator<Item = (usize, usize)> {
        /* let mut result = vec![];

        let mut visited = vec![false; self.gates.len()];

        let mut stack = vec![self.root];
        while let Some(next_idx) = stack.pop() {
            if !visited[next_idx] {
                result.push(next_idx);

                match self.gates[next_idx] {
                    gate::Gate::Add { first, second, output: _ } => {
                        stack.push(first);
                        stack.push(second);
                    },
                    gate::Gate::Mul { first, second, output: _ } => {
                        stack.push(first);
                        stack.push(second);
                    },
                    gate::Gate::MulByConst { first, second: _, output: _ } => {
                        stack.push(first);
                    },
                    _ => {}
                }

                visited[next_idx] = true;
            }
        }

        println!("traverse: {:?}", result);
        
        result.into_iter().rev().enumerate() */

        let mut stack = vec![];
        let mut visited = vec![false; self.gates.len()];

        self.roots.iter().for_each(|root_id| self.topological_sort(*root_id, &mut visited, &mut stack));

        stack.into_iter().enumerate()
    }

    fn topological_sort(&self, gate: usize, visited: &mut Vec<bool>, stack: &mut Vec<usize>) {
        if visited[gate] {
            return;
        }

        visited[gate] = true;

        match self.gates[gate] {
            gate::Gate::Add { first, second, output: _ } => {
                self.topological_sort(first, visited, stack);
                self.topological_sort(second, visited, stack);
            },
            gate::Gate::Mul { first, second, output: _ } => {
                self.topological_sort(first, visited, stack);
                self.topological_sort(second, visited, stack);
            },
            gate::Gate::MulByConst { first, second: _, output: _ } => {
                self.topological_sort(first, visited, stack);
            },
            _ => {}
        }

        stack.push(gate);
    }
}
