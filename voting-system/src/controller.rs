use druid::{
    widget::{prelude::*, Controller},
    Command, Handled, ExtEventSink, Target,
};
use std::net::{TcpStream};

use crate::{
    command, Params,
    vote_options::VoteOptions,
    vote,
};


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
        ctx: &mut EventCtx,
        command: &Command,
        data: &mut Params,
    ) -> Handled {
        if command.is(command::VOTE) {
            data.is_computed = false;
            self.vote_wrapper(ctx.get_external_handle(), command.get_unchecked(command::VOTE).clone());
            Handled::Yes
        } else if command.is(command::VOTE_OUTPUT) {
            data.is_computed = true;
            data.options_result = OptionsToggle(command.get_unchecked(command::VOTE_OUTPUT).clone());
            Handled::Yes
        } else {
            Handled::No
        }
    }

    fn vote_wrapper(&mut self, sink: ExtEventSink, input: Vec<bool>) {
        let vote_options = self.vote_options.clone();
        let stream = self.stream.try_clone().unwrap();
        std::thread::spawn(move || {
            let results: Vec<bool> = vote::vote(input, vote_options, stream)
                .into_iter().map(|result| result != 0).collect();
            sink.submit_command(command::VOTE_OUTPUT, results, Target::Auto).unwrap();
        });
    }
}
