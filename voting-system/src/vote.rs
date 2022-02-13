use std::net::TcpStream;
use std::io::{Read, Write};

use mpc::{
    party::Party,
    field::Field,
    message::Message,
    share_receiver::ShareReceiver,
    share_sender::ShareSender,
};

use crate::{
    vote_options,
    util::generate_circuit,
};

pub(crate) static GROUP_ORDER: u16 = 251;

pub(crate) fn vote(input: Vec<bool>, vote_options: vote_options::VoteOptions, mut stream: TcpStream) -> Vec<u16> {
    stream.write(b"VOTED").unwrap();

    let mut data = [0 as u8; 500];

    match stream.read(&mut data) {
        Ok(_) => println!("Protocol started!"),
        Err(_) => println!("Error when starting protocol"),
    };

    let rx = Box::new(ShareStream(stream.try_clone().unwrap(), vote_options.get_id()));
    let txs = (0..vote_options.get_number_of_voters()).map(
        |id| Box::new(ShareStream(stream.try_clone().unwrap(), id)) as _
    ).collect();

    Party::new(
        vote_options.get_id(),
        input.into_iter().map(u16::from).collect(),
        rx,
        txs,
        Field::new(GROUP_ORDER),
        generate_circuit(vote_options.get_number_of_voters(), vote_options.get_vote_threshold(), vote_options.get_number_of_options(), GROUP_ORDER),
        (vote_options.get_number_of_voters() - 1) / 2
    ).setup().run()
}

struct ShareStream(TcpStream, usize);

type Msg = Message<u16>;

impl ShareReceiver<Msg> for ShareStream {
    fn recv(&mut self) -> Msg {
        let mut data = [0u8; std::mem::size_of::<Msg>()];

        self.0.read_exact(&mut data).unwrap_or_else(|e| println!("Error recv: {}", e));
        unsafe { std::mem::transmute(data) }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

impl ShareSender<Msg> for ShareStream {
    fn send(&mut self, msg: Msg) {
        let data = [
            &(self.1 as u64).to_be_bytes()[..],
            &(std::mem::size_of::<Msg>() as u64).to_be_bytes(),
            unsafe { any_as_u8_slice(&msg) }
        ].concat();
        self.0.write(&data).unwrap_or_else(|e| { println!("Error send: {}", e); 0 });
    }
}