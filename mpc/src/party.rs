use crate::{
    circuit, gate, field, share_receiver, share_sender, message, polynomial
};

use log::{info, debug};

use std::collections::{HashSet, HashMap};

pub struct Party<DataType: Clone> {
    id: usize,
    secret: Vec<DataType>,
    rx: Box<dyn share_receiver::ShareReceiver<message::Message<DataType>>>,
    txs: Vec<Box<dyn share_sender::ShareSender<message::Message<DataType>>>>,
    shares: Vec<HashMap<usize, DataType>>,
    r_share: HashMap<usize, (DataType, DataType)>,
    field: field::Field<DataType>,
    circuit: circuit::Circuit<DataType>,
    threshold: usize,
    past_messages: HashSet<message::Message<DataType>>,
}

impl<DataType> Party<DataType>
where DataType: field::FieldElement +
                From<u16> +
                std::fmt::Debug +
                std::fmt::Display {

    pub fn new(id: usize,
                secret: Vec<DataType>,
                rx: Box<dyn share_receiver::ShareReceiver<message::Message<DataType>>>, 
                txs: Vec<Box<dyn share_sender::ShareSender<message::Message<DataType>>>>,
                field: field::Field<DataType>,
                circuit: circuit::Circuit<DataType>,
                threshold: usize) -> Self {
        let n_parties = circuit.get_n_parties() as usize;
        Party {
            id, secret, rx, txs, field, circuit, threshold,
            shares: vec![HashMap::new(); n_parties],
            r_share: HashMap::new(),
            past_messages: HashSet::new(),
        }
    }
    
    pub fn setup(mut self) -> Self {
        info!("Setupping party {}", self.id);
        
        for gate_id in self.circuit.traverse() {
            if matches!(self.circuit.get_gate(gate_id), gate::Gate::Mul { first: _, second: _, output: _ }) {
                debug!("Party {} preparing r_shares for gate({})", self.id, gate_id);

                let r = self.field.random();
                debug!("Party{}: gate({}) r = {}", self.id, gate_id, r);
                let s_poly = polynomial::Polynomial::random(r.clone(), self.threshold, self.field.clone());
                let t_poly = polynomial::Polynomial::random(r, self.threshold * 2, self.field.clone());

                let s_shares = self.broadcast_poly(s_poly, gate_id);
                let t_shares = self.broadcast_poly(t_poly, gate_id);
                
                self.r_share.insert(gate_id, (
                    s_shares.into_iter().fold(DataType::from(0), |a, b| self.field.add(a, b)),
                    t_shares.into_iter().fold(DataType::from(0), |a, b| self.field.add(a, b))
                ));

                debug!("Party{}: gate({}) r_share = {:?}", self.id, gate_id, self.r_share[&gate_id]);
            }
        }

        info!("Party {} setup finished", self.id);

        self
    }

    pub fn run(mut self) -> Vec<DataType> {
        info!("Running party {} with secret {:?}", self.id, self.secret);

        let mut n_gates = 0;
        let mut circuit = self.circuit.clone();
        for gate_id in circuit.traverse() {
            debug!("Party{}: processing gate {}", self.id, gate_id);

            let output = match circuit.get_gate(gate_id) {
                gate::Gate::Input { ref party, ref circuit_id, output: _ } => {
                    self.process_input(gate_id, *party, *circuit_id)
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

            circuit.get_gate_mut(gate_id).set_output(output);

            n_gates += 1;
        }

        let results = circuit.get_roots().into_iter().map(
            |gate_id| self.process_output(n_gates, circuit.get_gate(gate_id).get_output())
        ).collect();

        info!("Party {} finished with output {:?}", self.id, results);

        results
    }

    fn safe_recv(&mut self, gate_id: usize) -> message::Message<DataType> {
        let msg = match self.past_messages.iter().find(|&m| m.get_to() == self.id && m.get_gate() == gate_id) {
            Some(msg) => msg.clone(),
            None => loop {
                        let msg = self.rx.recv();
                        if msg.get_to() == self.id && msg.get_gate() == gate_id {
                            break msg;
                        } else {
                            self.past_messages.insert(msg.clone());
                        }
                    }            
        };
        let s = self.past_messages.remove(&msg);

        debug!("is_removed: {}", s);
        
        msg
    }

    fn process_input(&mut self, gate_id: usize, party: usize, circuit_id: usize) -> DataType {
        debug!("Party{}: process_input({}, {})", self.id, gate_id, party);

        if self.id == party {
            let poly = polynomial::Polynomial::random(self.secret[circuit_id].clone(), self.threshold, self.field.clone());
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
            let msg = self.safe_recv(gate_id);
            self.shares[party].insert(gate_id, msg.get_share());
        }

        debug!("Party{}: shares[{}][{}] = {}", self.id, party, gate_id, self.shares[party][&gate_id]);

        self.shares[party][&gate_id].clone()
    }

    fn process_add(&self, _gate_id: usize, first: &gate::Gate<DataType>, second: &gate::Gate<DataType>) -> DataType {
        debug!("Party{}: process_add({}, {})", self.id, first.get_output(), second.get_output());
        self.field.add(first.get_output(), second.get_output())
    }

    fn process_mul_by_const(&mut self, _gate_id: usize, first: &gate::Gate<DataType>, second: DataType) -> DataType {
        debug!("Party{}: process_mul_by_const({}, {})", self.id, first.get_output(), second);
        self.field.mul(first.get_output(), second)
    }

    fn process_mul(&mut self, gate_id: usize, first: &gate::Gate<DataType>, second: &gate::Gate<DataType>) -> DataType {
        let c_share = self.field.mul(first.get_output(), second.get_output());

        debug!("Party{}: process_mul({}, {}, {}) c_share = {}",
            self.id, gate_id, first.get_output(), second.get_output(), c_share);

        let g_share = self.field.add(c_share, self.r_share[&gate_id].1.clone());

        debug!("Party{}: process_mul({}, {}, {}) g_share = {}",
            self.id, gate_id, first.get_output(), second.get_output(), g_share);
        
        let shares = self.broadcast_share(g_share, gate_id);

        let g = polynomial::Polynomial::interpolate(shares, &self.field,
            self.circuit.get_n_parties() as usize, |x| DataType::from(x as u16));

        let result = self.field.sub(g, self.r_share[&gate_id].0.clone());

        result
    }
    
    fn process_output(&mut self, n_gates: usize, output: DataType) -> DataType {
        let n_parties = self.circuit.get_n_parties() as usize;

        debug!("Party{}: process_output({}, {})", self.id, n_gates, output);

        self.broadcast_share(output.clone(), n_gates)
            .into_iter()
            .enumerate()
            .for_each(|(party, share)| {
                self.shares[party].insert(n_gates, share);
            });

        debug!("Party{}: interpolating {:?}", self.id, (0..n_parties).map(|p| self.shares[p][&n_gates].clone()).collect::<Vec<_>>());

        polynomial::Polynomial::interpolate(
            (0..n_parties).map(|party| self.shares[party][&n_gates].clone()).collect(),
            &self.field, n_parties, |x| DataType::from(x as u16)
        )
    }

    fn broadcast_poly(&mut self, poly: polynomial::Polynomial<DataType>, gate_id: usize) -> Vec<DataType> {
        let n_parties = self.circuit.get_n_parties();
        let mut shares = vec![DataType::from(0); n_parties as usize];

        (0..n_parties)
            .map(|i| (i, poly.eval(DataType::from(i + 1))))
            .for_each(|(party, share)| {
                let party = party as usize;
                if party == self.id {
                    shares[party] = share;
                } else {
                    debug!("Party{}: gate({}) send share {} to Party{}",
                            self.id, gate_id, share, party);
                    self.txs[party].send(message::Message::new(self.id, party, gate_id, share));
                }
            });
        (1..n_parties)
            .for_each(|_| {
                let msg = self.safe_recv(gate_id);
                debug!("Party{}: gate({}) recv share {} from Party{}",
                            self.id, gate_id, msg.get_share(), msg.get_from());
                shares[msg.get_from()] = msg.get_share();
            });

        shares
    }

    fn broadcast_share(&mut self, share: DataType, gate_id: usize) -> Vec<DataType> {
        let n_parties = self.circuit.get_n_parties();
        let mut shares = vec![DataType::from(0); n_parties as usize];

        (0..n_parties)
            .map(|i| (i, share.clone()))
            .for_each(|(party, share)| {
                let party = party as usize;
                if party == self.id {
                    shares[party] = share;
                } else {
                    debug!("Party{}: gate({}) send share {} to Party{}",
                            self.id, gate_id, share, party);
                    self.txs[party].send(message::Message::new(self.id, party, gate_id, share));
                }
            });
        (1..n_parties)
            .for_each(|_| {
                let msg = self.safe_recv(gate_id);
                debug!("Party{}: gate({}) recv share {} from Party{}",
                            self.id, gate_id, msg.get_share(), msg.get_from());
                shares[msg.get_from()] = msg.get_share();
            });

        shares
    }
}
