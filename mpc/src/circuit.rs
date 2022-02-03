use crate::gate;

use std::iter::Iterator;

#[derive(Clone)]
pub struct Circuit<DataType: Clone> {
    root: gate::Gate<DataType>,
    n_parties: u8,
}

impl<DataType: Clone> Circuit<DataType> {
    
    pub fn new(root: gate::Gate<DataType>, n_parties: u8) -> Self {
        Circuit {
            root, n_parties
        }
    }

    pub fn get_n_parties(&self) -> u8 {
        self.n_parties
    }

    pub fn get_output_gate(&self) -> &gate::Gate<DataType> {
        &self.root
    }

    /* pub fn traverse(self) -> impl Iterator<Item = gate::Gate> {
        let mut result = vec![];

        let mut stack = vec![self.root];
        while let Some(next) = stack.pop() {
            result.push(next.clone());
            match next {
                gate::Gate::Add { first, second } => {
                    stack.push(*first);
                    stack.push(*second);
                },
                gate::Gate::Mul { first, second } => {
                    stack.push(*first);
                    stack.push(*second);
                },
                _ => {}
            }
        }

        return result.into_iter().rev();
    } */
}

impl<DataType: Clone> IntoIterator for Circuit<DataType> {
    type Item = (usize, gate::Gate<DataType>);
    type IntoIter = std::iter::Enumerate<std::iter::Rev<std::vec::IntoIter<gate::Gate<DataType>>>>;
    
    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];

        let mut stack = vec![self.root];
        while let Some(next) = stack.pop() {
            result.push(next.clone());
            match next {
                gate::Gate::Add { first, second, output: _ } => {
                    stack.push(*first);
                    stack.push(*second);
                },
                gate::Gate::Mul { first, second, output: _ } => {
                    stack.push(*first);
                    stack.push(*second);
                },
                gate::Gate::MulByConst { first, second: _, output: _ } => {
                    stack.push(*first);
                },
                _ => {}
            }
        }

        return result.into_iter().rev().enumerate();
    }
}