use itertools::Itertools;

use mpc::{
    circuit::Circuit,
    gate::Gate,
};

pub fn generate_circuit(number_of_voters: usize, vote_threshold: usize, _number_of_options: usize, group_order: u16) -> Circuit<u16> {
    let minus_one = group_order - 1;

    let mut circuit = Circuit::new(number_of_voters as u16);

    let inputs: Vec<usize> = (0..number_of_voters).map(|i| circuit.add(Gate::<u16>::new_input(i))).collect();

    let last_and_gates: Vec<usize> = inputs.iter().combinations(vote_threshold)
        .map(
            |subset| subset.clone().iter().map(|item| **item).reduce(
                |acc, item| circuit.add(Gate::new_mul(acc, item))
            )
        ).map(|item| item.unwrap()).collect();

    let mul_last_gates: Vec<usize> = (1..last_and_gates.len() + 1).map(
            |l| last_and_gates.clone().iter().combinations(l).map(
                |subset| {
                    let mul_subset = subset.iter().map(|item| **item).reduce(
                            |acc, item| circuit.add(Gate::new_mul(acc, item))
                        ).unwrap();
                    if l + 1 % 2 == 0 {
                        circuit.add(Gate::new_mul_by_const(mul_subset, minus_one))
                    } else {
                        mul_subset
                    }
                }
            ).collect::<Vec<usize>>()
        ).flatten()
        .collect();

    let root = mul_last_gates.iter().map(|item| *item).reduce(|acc, item| circuit.add(Gate::new_add(acc, item))).unwrap();
    
    println!("len of c: {}", circuit.gates.len());
    circuit.set_root(root);
    circuit
}