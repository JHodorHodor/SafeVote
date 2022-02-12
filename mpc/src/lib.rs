pub mod party;
pub mod circuit;
pub mod gate;
pub mod field;
pub mod share_receiver;
pub mod share_sender;
pub mod message;
mod polynomial;

#[cfg(test)]
mod tests {
    use super::gate::Gate;
    use super::circuit::Circuit;
    #[test]
    fn test_circuit_traversal() {
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

        println!("{:?}", circuit.traverse().collect::<Vec<_>>());
    }

    #[test]
    fn test_party_new() {
        env_logger::init();

        let rx = ShareReceiver { msg: None };
        let tx = ShareSender { chan: Rc::new(RefCell::new(rx.clone())) };
        
        /* let party = super::party::Party::new(0, 1, Box::new(rx), vec![Box::new(tx)],
            super::field::Field::new(97),
            super::circuit::Circuit::new(super::gate::Gate::<u8>::new_input(0), 1),
            1);
        assert_eq!(party.run(), 1); */
    }

    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct ShareReceiver {
        msg: Option<super::message::Message<u8>>
    }

    impl super::share_receiver::ShareReceiver<super::message::Message<u8>> for ShareReceiver {
        fn recv(&mut self) -> super::message::Message<u8> {
            (*self.msg.as_ref().unwrap()).clone()
        }
    }

    struct ShareSender {
        chan: Rc<RefCell<ShareReceiver>>
    }

    impl super::share_sender::ShareSender<super::message::Message<u8>> for ShareSender {
        fn send(&mut self, msg: super::message::Message<u8>) {
            self.chan.borrow_mut().msg = Some(msg);
        }
    }
}
