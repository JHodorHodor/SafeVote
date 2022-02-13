use itertools::Itertools;

use mpc::{
    circuit::Circuit,
    gate::Gate,
};

pub(crate) fn generate_circuit(number_of_voters: usize, vote_threshold: usize, number_of_options: usize, group_order: u16) -> Circuit<u16> {
    let minus_one = group_order - 1;

    let mut circuit = Circuit::new(number_of_voters as u16);

    let roots = (0..number_of_options).map(|option_id| {
        let inputs: Vec<usize> = (0..number_of_voters).map(|i| circuit.add(Gate::<u16>::new_input(i, option_id))).collect();

        let last_and_gates: Vec<usize> = inputs.into_iter().combinations(vote_threshold)
            .map(
                |subset| subset.clone().into_iter().reduce(
                    |acc, item| circuit.add(Gate::new_mul(acc, item))
                )
            ).map(Option::unwrap).collect();

        let mul_last_gates: Vec<usize> = (1..last_and_gates.len() + 1).map(
                |l| last_and_gates.clone().into_iter().combinations(l).map(
                    |subset| {
                        let mul_subset = subset.into_iter().reduce(
                                |acc, item| circuit.add(Gate::new_mul(acc, item))
                            ).unwrap();
                        if l % 2 == 0 {
                            circuit.add(Gate::new_mul_by_const(mul_subset, minus_one))
                        } else {
                            mul_subset
                        }
                    }
                ).collect::<Vec<_>>()
            ).flatten()
            .collect();

            mul_last_gates.into_iter().reduce(|acc, item| circuit.add(Gate::new_add(acc, item))).unwrap()
    }).collect();
    
    println!("Circuit size: {}", circuit.size());
    circuit.set_roots(roots);
    circuit
}

#[cfg(test)]
mod tests {
    use mpc::{
        circuit::Circuit,
        gate::Gate,
        field::Field,
    };
    use super::generate_circuit;

    #[test]
    fn test_circuit_1() {
        let number_of_voters = 5;
        let vote_threshold = 3;
        let number_of_options = 2;
        let group_order: u16 = 251;
        let input = vec![vec![0, 0], vec![1, 0], vec![1, 1], vec![1, 0], vec![1, 0]];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);

        assert_eq!(run(circuit, 251, input), vec![1, 0])
    }

    #[test]
    fn test_circuit_2() {
        let number_of_voters = 4;
        let vote_threshold = 2;
        let number_of_options = 3;
        let group_order: u16 = 251;
        let input = vec![vec![0, 0, 0], vec![1, 0, 0], vec![1, 1, 0], vec![1, 1, 1]];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);

        assert_eq!(run(circuit, 251, input), vec![1, 1, 0])
    }

    #[test]
    fn test_circuit_3() {
        let number_of_voters = 3;
        let vote_threshold = 2;
        let number_of_options = 5;
        let group_order: u16 = 251;
        let input = vec![vec![1, 1, 1, 0, 0], vec![1, 1, 0, 0, 0], vec![1, 0, 0, 0, 0]];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);

        assert_eq!(run(circuit, 251, input), vec![1, 1, 0, 0, 0])
    }


    fn run(cir: Circuit<u16>, field_order: u16, input: Vec<Vec<u16>>) -> Vec<u16> {

        let field = Field::<u16>::new(field_order);

        let mut circuit = cir.clone();
        for gate_id in circuit.traverse() {

            let output = match circuit.get_gate(gate_id) {
                Gate::Input { ref party, ref circuit_id, output: _ } => {
                    input[*party][*circuit_id]
                }
                Gate::Add { ref first, ref second, output: _ } => {
                    process_add(circuit.get_gate(*first), circuit.get_gate(*second), &field)
                }
                Gate::MulByConst { ref first, ref second, output: _ } => {
                    process_mul_by_const(circuit.get_gate(*first), second.clone(), &field)
                }
                Gate::Mul { ref first, ref second, output: _ } => {
                    process_mul(circuit.get_gate(*first), circuit.get_gate(*second), &field)
                }
            };

            circuit.get_gate_mut(gate_id).set_output(output);
        }
        circuit.get_roots().into_iter().map(|gate_id| circuit.get_gate(gate_id).get_output()).collect()
    }

    fn process_add(first: &Gate<u16>, second: &Gate<u16>, field: &Field<u16>) -> u16 {
        field.add(first.get_output(), second.get_output())
    }

    fn process_mul_by_const(first: &Gate<u16>, second: u16, field: &Field<u16>) -> u16 {
        field.mul(first.get_output(), second)
    }

    fn process_mul(first: &Gate<u16>, second: &Gate<u16>, field: &Field<u16>) -> u16 {
        field.mul(first.get_output(), second.get_output())
    }
}
