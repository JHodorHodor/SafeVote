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
        println!("Party::new(): {}/{}", id, n_parties);
        Party {
            id, secret, rx, txs, field, circuit, threshold,
            shares: vec![HashMap::new(); n_parties]
        }
    }

    pub fn run(mut self) -> DataType {
        info!("Running party {} with secret {}", self.id, self.secret);
        println!("Running party {} with secret {}", self.id, self.secret);

        let mut n_gates = 0;
        let mut output_gate = None;
        for (gate_id, mut gate) in self.circuit.clone() {
            println!("Party{}: processing gate {}", self.id, gate_id);
            match gate {
                gate::Gate::Input { ref party, ref mut output } => {
                    *output = Some(self.process_input(gate_id, *party));
                }
                gate::Gate::Add { ref first, ref second, ref mut output } => {
                    *output = Some(self.process_add(gate_id, first, second));
                }
                gate::Gate::MulByConst { ref first, ref second, ref mut output } => {
                    *output = Some(self.process_mul_by_const(gate_id, first, second.clone()));
                }
                gate::Gate::Mul { ref first, ref second, ref mut output } => {
                    *output = Some(self.process_mul(gate_id, first, second));
                }
            }

            n_gates += 1;
            output_gate = Some(gate);
        }

        let output = output_gate.unwrap().get_output();
        let result = self.process_output(n_gates, output);

        info!("Party {} finished with output {}", self.id, result);
        println!("Party {} finished with output {}", self.id, result);

        result
    }

    fn process_input(&mut self, gate_id: usize, party: usize) -> DataType {
        println!("Party{}: process_input({}, {})", self.id, gate_id, party);
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

        println!("computed shares");

        self.shares[party][&gate_id].clone()
    }

    fn process_add(&self, _gate_id: usize, first: &Box<gate::Gate<DataType>>, second: &Box<gate::Gate<DataType>>) -> DataType {
        self.field.add(first.get_output(), second.get_output())
    }

    fn process_mul_by_const(&mut self, _gate_id: usize, first: &Box<gate::Gate<DataType>>, second: DataType) -> DataType {
        self.field.mul(first.get_output(), second)
    }

    fn process_mul(&mut self, gate_id: usize, first: &Box<gate::Gate<DataType>>, second: &Box<gate::Gate<DataType>>) -> DataType {
        let c = self.field.mul(first.get_output(), second.get_output());

        // TODO: move to setup phase
        let n_parties = self.circuit.get_n_parties();
        let poly = polynomial::Polynomial::random(c, self.threshold, self.field.clone());

        (0..n_parties)
                .map(|i| (i, poly.eval(DataType::from(i + 1))))
                .for_each(|(party, share)| {
                    let party = party as usize;
                    if party == self.id {
                        self.shares[party].insert(gate_id, share);
                    } else {
                        self.txs[party].send(message::Message::new(self.id, party, gate_id, share));
                    }
                });

        (0..n_parties as usize)
                .for_each(|party| {
                    while !self.shares[party].contains_key(&gate_id) {
                        let msg = self.rx.recv();
                        self.shares[party].insert(gate_id, msg.get_share());
                    }
                });

        let result = (0..n_parties)
                .map(|party| (self.shares[party as usize][&gate_id].clone(),
                                polynomial::Polynomial::lagrange(
                                    (0..n_parties).map(|i| DataType::from(i + 1)),
                                    DataType::from(party),
                                    self.field.clone()
                                )))
                .map(|(share, lagr)| self.field.mul(share, lagr))
                .fold(self.field.zero(), |a, b| self.field.add(a, b));

        result
    }
    
    fn process_output(&mut self, n_gates: usize, output: DataType) -> DataType {
        let n_parties = self.circuit.get_n_parties();

        (0..n_parties)
                .map(|i| (i, output.clone()))
                .for_each(|(party, share)| {
                    let party = party as usize;
                    if party == self.id {
                        self.shares[party].insert(n_gates, share);
                    } else {
                        self.txs[party].send(message::Message::new(self.id, party, n_gates, share));
                    }
                });

        (0..n_parties as usize)
                .for_each(|party| {
                    while !self.shares[party].contains_key(&n_gates) {
                        let msg = self.rx.recv();
                        self.shares[party].insert(n_gates, msg.get_share());
                    }
                });

        let result = (0..n_parties)
                .map(|party| (self.shares[party as usize][&n_gates].clone(),
                    polynomial::Polynomial::lagrange(
                        (0..n_parties).map(|i| DataType::from(i + 1)),
                        DataType::from(party),
                        self.field.clone()
                    )))
                .map(|(share, lagr)| self.field.mul(share, lagr))
                .fold(self.field.zero(), |a, b| self.field.add(a, b));
        
        result
    }
}
