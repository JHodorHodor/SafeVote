use crate::circuit;
use crate::gate;
use crate::field;
use crate::share_receiver;
use crate::share_sender;
use crate::message;
use crate::polynomial;

use log::info;

use std::collections::HashMap;

pub struct Party<DataType: Clone> {
    id: usize,
    secret: DataType,
    rx: Box<dyn share_receiver::ShareReceiver<message::Message<DataType>>>,
    // done: bool,
    txs: Vec<Box<dyn share_sender::ShareSender<message::Message<DataType>>>>,
    shares: Vec<HashMap<usize, DataType>>,
    field: field::Field<DataType>,
    circuit: circuit::Circuit<DataType>,
    threshold: usize,
}

impl<DataType> Party<DataType>
where DataType: field::FieldElement +
                From<u8> +
                std::fmt::Display {

    pub fn new(id: usize, secret: DataType,
                rx: Box<dyn share_receiver::ShareReceiver<message::Message<DataType>>>, 
                txs: Vec<Box<dyn share_sender::ShareSender<message::Message<DataType>>>>,
                field: field::Field<DataType>,
                circuit: circuit::Circuit<DataType>,
                threshold: usize) -> Self {
        let n_parties = circuit.get_n_parties() as usize;
        Party {
            id, secret, rx, txs, field, circuit, threshold,
            shares: vec![HashMap::new(); n_parties]
        }
    }

    pub fn run(mut self) -> DataType {
        info!("Running party {} with secret {}", self.id, self.secret);

        for (gate_id, mut gate) in self.circuit.clone() {
            match gate {
                gate::Gate::Input { party, ref mut output } => {
                    *output = Some(self.process_input(gate_id, party));
                }
                gate::Gate::Add { first, second, ref mut output } => {
                    *output = Some(self.process_add(gate_id, first, second));
                }
                gate::Gate::MulByConst { first, second, ref mut output } => {
                    *output = Some(self.process_mul_by_const(gate_id, first, second));
                }
                gate::Gate::Mul { first, second, ref mut output } => {
                    *output = Some(self.process_mul(gate_id, first, second));
                }
            }
        }

        let output = self.circuit.get_output_gate().get_output();
        let result = self.process_output(output);

        info!("Party {} finished with output {}", self.id, result);

        result
    }

    fn process_input(&mut self, gate_id: usize, party: usize) -> DataType {
        if self.id == party {
            let poly = polynomial::Polynomial::random(self.secret.clone(), self.threshold, self.field.clone());
            (0..self.circuit.get_n_parties())
                .map(|i| (i, poly.eval(DataType::from(i + 1))))
                .for_each(|(party, share)| {
                    let party = party as usize;
                    if party == self.id {
                        self.shares[party].insert(gate_id, share);
                    } else {
                        self.txs[party].send(message::Message::new(self.id, party, gate_id, share));
                    }
                });
        } else {
            while !self.shares[party].contains_key(&gate_id) {
                let msg = self.rx.recv();
                self.shares[party].insert(gate_id, msg.get_share());
            }
        }

        self.shares[party][&gate_id].clone()
    }

    fn process_add(&self, _gate_id: usize, first: Box<gate::Gate<DataType>>, second: Box<gate::Gate<DataType>>) -> DataType {
        self.field.add(first.get_output(), second.get_output())
    }

    fn process_mul_by_const(&mut self, _gate_id: usize, first: Box<gate::Gate<DataType>>, second: DataType) -> DataType {
        self.field.mul(first.get_output(), second)
    }

    fn process_mul(&mut self, _gate_id: usize, first: Box<gate::Gate<DataType>>, second: Box<gate::Gate<DataType>>) -> DataType {
        // TODO: impl
        let c = self.field.mul(first.get_output(), second.get_output());

        c
    }
    
    fn process_output(&mut self, output: DataType) -> DataType {
        // TODO: impl interpolate output shares
        output
    }
}
