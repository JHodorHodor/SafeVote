use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};
use std::net::{TcpStream};
use std::io::{Read, Write};

use mpc::{
    party::Party,
    field::Field,
    message::Message,
    share_receiver::ShareReceiver,
    share_sender::ShareSender,
};

use crate::{
    command, Params,
    util::generate_circuit,
};

pub(crate) static GROUP_ORDER: u16 = 251;


#[derive(Clone)]
pub(crate) struct OptionsToggle(pub Vec<bool>);

impl Data for OptionsToggle {
    fn same(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0.iter().zip(other.0.iter()).all(|opt| opt.0 == opt.1)
    }
}

#[derive(Clone)]
pub(crate) struct VoteOptions {
    id: usize,
    number_of_voters: usize,
    vote_threshold: usize,
    number_of_options: usize,
}

impl VoteOptions {
    pub(crate) fn new(id: usize, number_of_voters: usize, vote_threshold: usize, number_of_options: usize) -> Self {
        VoteOptions {
            id, number_of_voters, vote_threshold, number_of_options
        }
    }

    pub(crate) fn get_number_of_options(&self) -> usize {
        self.number_of_options
    }
}

pub(crate) struct VoteChoiceController {
    stream: TcpStream,
    vote_options: VoteOptions
}

impl VoteChoiceController {
    pub(crate) fn new(stream: TcpStream, vote_options: VoteOptions) -> Self {
        VoteChoiceController {
            stream,
            vote_options,
        }
    }
}

impl<W> Controller<Params, W> for VoteChoiceController
where
    W: Widget<Params>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Params,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }
}

impl VoteChoiceController {
    fn command(
        &mut self,
        _ctx: &mut EventCtx,
        command: &Command,
        data: &mut Params,
    ) -> Handled {
        if command.is(command::VOTE) {
            let results = self.vote(command.get_unchecked(command::VOTE).clone()).into_iter().map(|result| result != 0).collect();
            data.options_result = OptionsToggle(results);
            data.is_computed = true;
            Handled::Yes
        } else {
            Handled::No
        }
    }

    fn vote(&mut self, input: Vec<bool>) -> Vec<u16> {
        self.stream.write(b"VOTED").unwrap();

        let mut data = [0 as u8; 500];

        match self.stream.read(&mut data) {
            Ok(_) => println!("Protocol started!"),
            Err(_) => println!("Error"),
        };

        let rx = Box::new(ShareStream(self.stream.try_clone().unwrap(), self.vote_options.id));
        let txs = (0..self.vote_options.number_of_voters).map(
            |id| Box::new(ShareStream(self.stream.try_clone().unwrap(), id)) as _
        ).collect();

        Party::new(
            self.vote_options.id,
            input.into_iter().map(u16::from).collect(),
            rx,
            txs,
            Field::new(GROUP_ORDER),
            generate_circuit(self.vote_options.number_of_voters, self.vote_options.vote_threshold, self.vote_options.number_of_options, GROUP_ORDER),
            self.vote_options.vote_threshold
        ).run()
    }
}

struct ShareStream(TcpStream, usize);

impl ShareReceiver<Message<u16>> for ShareStream {
    fn recv(&mut self) -> Message<u16> {
        let mut data = [0u8; std::mem::size_of::<Message<u16>>()];

        self.0.read_exact(&mut data).unwrap_or_else(|_e| println!("Error recv"));
        unsafe { std::mem::transmute(data) }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

impl ShareSender<Message<u16>> for ShareStream {
    fn send(&mut self, msg: Message<u16>) {
        let data = [&(self.1 as u32).to_be_bytes()[..], unsafe { any_as_u8_slice(&msg) }].concat();
        self.0.write(&data).unwrap_or_else(|_e| { println!("Error send"); 0 });
    }
}