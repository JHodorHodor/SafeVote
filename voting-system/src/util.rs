use mpc::{
    circuit::Circuit,
    gate::Gate,
};

pub fn generate_circuit() -> Circuit<u8> {
    let mut circuit = Circuit::new(3);

    let i0 = circuit.add(Gate::<u8>::new_input(0));
    let i1 = circuit.add(Gate::<u8>::new_input(1));
    let i2 = circuit.add(Gate::<u8>::new_input(2));
    let m01 = circuit.add(Gate::new_mul(i0, i1));
    let m02 = circuit.add(Gate::new_mul(i0, i2));
    let m12 = circuit.add(Gate::new_mul(i1, i2));
    let k0 = circuit.add(Gate::new_mul(m01, m02));
    let k1 = circuit.add(Gate::new_mul(m01, m12));
    let k2 = circuit.add(Gate::new_mul(m12, m02));
    let k00 = circuit.add(Gate::new_mul_by_const(k0, 12));
    let k10 = circuit.add(Gate::new_mul_by_const(k1, 12));
    let k20 = circuit.add(Gate::new_mul_by_const(k2, 12));
    let s = circuit.add(Gate::new_mul(k0, m12));
    let n1 = circuit.add(Gate::new_add(m01, m02));
    let n2 = circuit.add(Gate::new_add(n1, m12));
    let n3 = circuit.add(Gate::new_add(n2, k00));
    let n4 = circuit.add(Gate::new_add(n3, k10));
    let n5 = circuit.add(Gate::new_add(n4, k20));
    let n6 = circuit.add(Gate::new_add(n5, s));
    circuit.set_root(n6);
    circuit
}