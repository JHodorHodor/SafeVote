use crate::gate;

use std::iter::Iterator;

#[derive(Clone)]
pub struct Circuit<DataType: Clone> {
    gates: Vec<gate::Gate<DataType>>,
    root: usize,
    n_parties: u16,
}

impl<DataType: Clone> Circuit<DataType> {
    
    pub fn new(n_parties: u16) -> Self {
        Circuit {
            gates: vec![],
            root: 0, 
            n_parties
        }
    }

    pub fn set_root(&mut self, gate_idx: usize) {
        self.root = gate_idx;
    }

    pub fn add(&mut self, gate: gate::Gate<DataType>) -> usize {
        self.gates.push(gate);
        self.gates.len() - 1
    }

    pub(crate) fn get_n_parties(&self) -> u16 {
        self.n_parties
    }

    pub fn get_root(&self) -> &gate::Gate<DataType> {
        self.get_gate(self.root)
    }

    pub fn get_gate(&self, idx: usize) -> &gate::Gate<DataType> {
        &self.gates[idx]
    }

    pub fn get_gate_mut(&mut self, idx: usize) -> &mut gate::Gate<DataType> {
        &mut self.gates[idx]
    }

    pub fn traverse(&self) -> impl Iterator<Item = usize> {
        (0..self.gates.len()).into_iter()
    }

    pub fn size(&self) -> usize {
        self.gates.len()
    }
}
