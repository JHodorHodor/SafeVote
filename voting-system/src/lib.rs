mod util;

#[cfg(test)]
mod tests {
    use mpc::{
        circuit::Circuit,
        gate::Gate,
        field::Field,
    };
    use crate::util::generate_circuit;
    
    #[test]
    fn test_circuit() {
        let mut circuit = generate_circuit(4, 1, 2, 251);
        println!("{:?}", run(circuit, 251));
    }


    pub fn run(cir: Circuit<u16>, field_order: u16) -> u16 {

        let field = Field::<u16>::new(field_order);

        let mut n_gates = 0;
        let mut circuit = cir.clone();
        for (gate_id, gate_loc) in circuit.traverse() {

            let output = match circuit.get_gate(gate_loc) {
                Gate::Input { ref party, output: _ } => {
                    process_input(*party)
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

            n_gates += 1;
        }

        let output = circuit.get_root().get_output();

        println!("Finished with output {}", output);

        output
    }

    fn process_input(party: usize) -> u16 {
        if party == 0 {
            return 0u16;
        } else if party == 1 {
            return 0u16;
        } else if party == 2 {
            return 0u16;
        } else if party == 3 {
            return 0u16;
        } else {
            return 0u16;
        }
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
