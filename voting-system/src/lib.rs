mod util;

#[cfg(test)]
mod tests {
    use mpc::{
        circuit::Circuit,
        gate::Gate,
        field::Field,
    };
    use crate::util::generate_circuit;

    /*
    #[test]
    fn test_circuit_1() {
        let number_of_voters = 5;
        let vote_threshold = 3;
        let number_of_options = 1;
        let group_order: u16 = 251;
        let input = vec![1u16, 0u16, 1u16, 1u16, 0u16];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);
        
        assert_eq!(run(circuit, 251, input), 1)
    }

    #[test]
    fn test_circuit_2() {
        let number_of_voters = 5;
        let vote_threshold = 2;
        let number_of_options = 1;
        let group_order: u16 = 251;
        let input = vec![0u16, 0u16, 1u16, 1u16, 1u16];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);

        assert_eq!(run(circuit, 251, input), 1)
    }

    #[test]
    fn test_circuit_3() {
        let number_of_voters = 5;
        let vote_threshold = 4;
        let number_of_options = 1;
        let group_order: u16 = 251;
        let input = vec![0u16, 0u16, 1u16, 1u16, 0u16];

        let circuit = generate_circuit(number_of_voters, vote_threshold, number_of_options, group_order);

        assert_eq!(run(circuit, 251, input), 0)
    }
    */

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


    pub fn run(cir: Circuit<u16>, field_order: u16, input: Vec<Vec<u16>>) -> Vec<u16> {

        let field = Field::<u16>::new(field_order);

        let mut circuit = cir.clone();
        for (_gate_id, gate_loc) in circuit.traverse() {

            let output = match circuit.get_gate(gate_loc) {
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

            circuit.get_gate_mut(gate_loc).set_output(output);
        }
        circuit.get_roots().iter().map(|gate_id| circuit.get_gate(*gate_id).get_output()).collect()
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
