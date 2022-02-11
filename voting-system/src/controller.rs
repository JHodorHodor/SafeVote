use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

use mpc::{
    party::Party,
    field::Field,
    circuit::Circuit,
    gate::Gate,
    message::Message,
    share_receiver::ShareReceiver,
    share_sender::ShareSender,
};

use crate::command;
use crate::Params;

pub struct VoteChoiceController {
    stream: TcpStream,
    id: usize,
    number_of_voters: usize,
    vote_treshold: usize,
}

impl VoteChoiceController {
    pub fn new(stream: TcpStream, id: usize, number_of_voters: usize, vote_treshold: usize) -> Self {
        VoteChoiceController {
            stream,
            id,
            number_of_voters,
            vote_treshold,
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
        _data: &mut Params,
    ) -> Handled {
        //tracing::debug!("Voting tab received command: {:?}", command);

        if command.is(command::VOTE) {
            self.vote(*command.get_unchecked(command::VOTE));
            Handled::Yes
        } else {
            Handled::No
        }
    }

    fn vote(&mut self, _input: u8) -> u8 {
        let input = b"VOTED";
        self.stream.write(input).unwrap();

        let mut data = [0 as u8; 500];

        match self.stream.read(&mut data) {
            Ok(_) => println!("Protocol started!"),
            Err(_) => println!("Error"),
        };

        let rx = ShareStream(self.stream.try_clone().unwrap(), self.number_of_voters);
        let tx0 = ShareStream(self.stream.try_clone().unwrap(), self.number_of_voters);
        let tx1 = ShareStream(self.stream.try_clone().unwrap(), self.number_of_voters);
        let tx2 = ShareStream(self.stream.try_clone().unwrap(), self.number_of_voters);

        Party::new(self.id, _input, Box::new(rx), vec![Box::new(tx0), Box::new(tx1), Box::new(tx2)],
            Field::new(97),
            Circuit::new(Gate::<u8>::new_input(0), 3),
            2).run()
    }
}

struct ShareStream(TcpStream, usize);

// TODO: use e.g. serde for (de)serialization of Message<>


impl ShareReceiver<Message<u8>> for ShareStream {
    fn recv(&mut self) -> Message<u8> {
        let mut data = [0u8; std::mem::size_of::<Message<u8>>()];

        match self.0.read(&mut data) {
            Ok(size) => {
                println!("{}", from_utf8(&data[0..size]).unwrap());
            },
            Err(_) => println!("Error"),
        }
        unsafe { std::mem::transmute(data) }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

impl ShareSender<Message<u8>> for ShareStream {
    fn send(&mut self, msg: Message<u8>) {
        self.0.write(&(msg.to as u32).to_be_bytes()).unwrap();
        let data = unsafe { any_as_u8_slice(&msg) };
        println!("send: {:?} to {}", data, msg.to);
        self.0.write(data).unwrap();
    }
}