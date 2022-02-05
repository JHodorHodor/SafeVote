use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};
use std::net::{TcpStream};
use std::io::{Read, Write};

use crate::command;
use crate::Params;

pub struct VoteChoiceController {
    stream: TcpStream,
}

impl VoteChoiceController {
    pub fn new(stream: TcpStream) -> Self {
        VoteChoiceController {
            stream
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
        let input = b"READY";
        self.stream.write(input).unwrap();
        0
    }
}