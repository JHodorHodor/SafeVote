use crate::circuit;
use crate::gate;
use crate::field;
use crate::share_receiver;
use crate::share_sender;
use crate::message;
use crate::polynomial;

use log::{info, debug};

use std::collections::HashMap;

pub struct Party<DataType: Clone> {
    id: usize,
    secret: DataType,
    rx: Box<dyn share_receiver::ShareReceiver<message::Message<DataType>>>,
    txs: Vec<Box<dyn share_sender::ShareSender<message::Message<DataType>>>>,
    shares: Vec<HashMap<usize, DataType>>,
    field: field::Field<DataType>,
    circuit: circuit::Circuit<DataType>,
    threshold: usize,
}

impl<DataType> Party<DataType>
where DataType: field::FieldElement +
                From<u8> +
                std::fmt::Debug +
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

        let mut n_gates = 0;
        let mut circuit = self.circuit.clone();
        for (gate_id, gate_loc) in circuit.traverse() {
            debug!("Party{}: processing gate {} (location = {})", self.id, gate_id, gate_loc);

            let output = match circuit.get_gate(gate_loc) {
                gate::Gate::Input { ref party, output: _ } => {
                    self.process_input(gate_id, *party)
                }
                gate::Gate::Add { ref first, ref second, output: _ } => {
                    self.process_add(gate_id, circuit.get_gate(*first), circuit.get_gate(*second))
                }
                gate::Gate::MulByConst { ref first, ref second, output: _ } => {
                    self.process_mul_by_const(gate_id, circuit.get_gate(*first), second.clone())
                }
                gate::Gate::Mul { ref first, ref second, output: _ } => {
                    self.process_mul(gate_id, circuit.get_gate(*first), circuit.get_gate(*second))
                }
            };

            circuit.get_gate_mut(gate_loc).set_output(output);

            n_gates += 1;
        }

        let output = circuit.get_root().get_output();
        let result = self.process_output(n_gates, output);

        info!("Party {} finished with output {}", self.id, result);

        result
    }

    fn process_input(&mut self, gate_id: usize, party: usize) -> DataType {
        debug!("Party{}: process_input({}, {})", self.id, gate_id, party);

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
                if msg.to == self.id && msg.gate == gate_id {
                    self.shares[party].insert(gate_id, msg.get_share());
                }
            }
        }

        debug!("Party{}: shares[{}][{}] = {}", self.id, party, gate_id, self.shares[party][&gate_id]);

        self.shares[party][&gate_id].clone()
    }

    fn process_add(&self, _gate_id: usize, first: &gate::Gate<DataType>, second: &gate::Gate<DataType>) -> DataType {
        debug!("Party{}: process_add({}, {})", self.id, first.get_output(), second.get_output());
        self.field.add(first.get_output(), second.get_output())
    }

    fn process_mul_by_const(&mut self, _gate_id: usize, first: &gate::Gate<DataType>, second: DataType) -> DataType {
        self.field.mul(first.get_output(), second)
    }

    fn process_mul(&mut self, gate_id: usize, first: &gate::Gate<DataType>, second: &gate::Gate<DataType>) -> DataType {
        let c = self.field.mul(first.get_output(), second.get_output());

        debug!("Party{}: process_mul({}, {}, {}) c = {}",
            self.id, gate_id, first.get_output(), second.get_output(), c);

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

        (1..n_parties)
                .for_each(|_| {
                    loop {
                        let msg = self.rx.recv();
                        if msg.to == self.id && msg.gate == gate_id {
                            self.shares[msg.from].insert(gate_id, msg.get_share());
                            return;
                        }
                    }
                });

        let result = (0..n_parties)
                .map(|party| (self.shares[party as usize][&gate_id].clone(),
                                polynomial::Polynomial::lagrange(
                                    (0..n_parties).map(|i| DataType::from(i + 1)),
                                    DataType::from(party + 1),
                                    self.field.clone()
                                )))
                .map(|(share, lagr)| self.field.mul(share, lagr))
                .fold(self.field.zero(), |a, b| self.field.add(a, b));

        result
    }
    
    fn process_output(&mut self, n_gates: usize, output: DataType) -> DataType {
        let n_parties = self.circuit.get_n_parties();

        debug!("Party{}: process_output({}, {})", self.id, n_gates, output);

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

        (1..n_parties)
                .for_each(|_| {
                    loop {
                        let msg = self.rx.recv();
                        if msg.to == self.id && msg.gate == n_gates {
                            self.shares[msg.from].insert(n_gates, msg.get_share());
                            return;
                        }
                    }
                });

        debug!("Party{}: interpolating {:?}", self.id, (0..n_parties).map(|p| self.shares[p as usize][&n_gates].clone()).collect::<Vec<_>>());

        let result = (0..n_parties)
                .map(|party| (self.shares[party as usize][&n_gates].clone(),
                    polynomial::Polynomial::lagrange(
                        (0..n_parties).map(|i| DataType::from(i + 1)),
                        DataType::from(party + 1),
                        self.field.clone()
                    )))
                .map(|(share, lagr)| self.field.mul(share, lagr))
                .fold(self.field.zero(), |a, b| self.field.add(a, b));
        
        result
    }
}
