use druid::{
	widget::{prelude::*, Button, Flex, Label, Either},
	AppLauncher, Widget, WidgetExt, WindowDesc, Data, Lens, Env, Color
};
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::env;
use std::convert::TryInto;

mod command;
mod controller;
mod util;

#[derive(Clone, Data, Lens)]
struct Params {
    is_confirmed: bool,
    options: String,
    options_toggle: controller::OptionsToggle,
    is_computed: bool,
    options_result: controller::OptionsToggle,
}

fn main() {
	env_logger::init();

    let id: usize = match env::args().collect::<Vec<String>>().get(1) {
        Some(id) => match id.parse::<usize>() {
            Ok(id) => id,
            _ => panic!("Client id should be a non-negative integer!")
        },
        None => panic!("Specify client id!"),
    };

    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");

            // Send party id to server
            let id_bytes = (id as u32).to_be_bytes();
            stream.write(&id_bytes).unwrap();

            // Receive voting options
            let mut data = [0 as u8; std::mem::size_of::<u32>()];
            let number_of_voters = match stream.read_exact(&mut data) {
                Ok(_) => {
                    u32::from_be_bytes(data.try_into().unwrap()) as usize
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };
            let vote_threshold = match stream.read_exact(&mut data) {
                Ok(_) => {
                    u32::from_be_bytes(data.try_into().unwrap()) as usize
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };

            let mut data = [0 as u8; 500];
            let voting_options = match stream.read(&mut data) {
                Ok(size) => {
                	from_utf8(&data[0..size]).unwrap().to_string()
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };

            let number_of_options = voting_options.split(",").collect::<Vec<&str>>().len();
            let vote_options = controller::VoteOptions::new(
                id, number_of_voters, vote_threshold, number_of_options
            );

		    println!("Options received: number of voters: {};  vote threshold: {}; voting options: {}.", number_of_voters, vote_threshold, voting_options);

            // Init UI
		    let main_window = WindowDesc::new(move || ui_builder(stream, vote_options))
		        .title("Voting App")
		        .window_size((300.0, 500.0));

		    let params = Params {
		        is_confirmed: false,
		        options: voting_options,
                options_toggle: controller::OptionsToggle(vec![false; number_of_options]),
                is_computed: false,
                options_result: controller::OptionsToggle(vec![false; number_of_options]),
		    };

		    AppLauncher::with_window(main_window)
		        .launch(params)
		        .expect("Failed to launch application");
        },
        Err(e) => {
            panic!("Failed to connect: {}", e);
        }
    }
}

fn ui_builder(stream: TcpStream, vote_options: controller::VoteOptions) -> impl Widget<Params> {
    let buttons_group = (0..vote_options.get_number_of_options()).fold(
    	Flex::column(),
    	|column, i| column.with_child(
            Either::new(
                move |data: &Params, _env: &Env| data.options_toggle.0[i],
                Button::new(
                    move |data: &Params, _env: &Env| {
                        let mut label = "-> ".to_owned();
                        label.push_str(data.options.split(",").collect::<Vec<&str>>()[i]);
                        label
                    }
                ).on_click(
                    move |_ctx: &mut EventCtx, data: &mut Params, _env| data.options_toggle.0[i] = false
                ).border(Color::rgb(0.0, 0.0, 0.3), 2.0).padding(10.0).expand_width(),
                Button::new(
                    move |data: &Params, _env: &Env| data.options.split(",").collect::<Vec<&str>>()[i].to_string()
                ).on_click(
                    move |_ctx: &mut EventCtx, data: &mut Params, _env| data.options_toggle.0[i] = true
                ).border(Color::rgb(0.0, 0.0, 0.3), 2.0).padding(10.0).expand_width(),
            )
    	)
    );

    let label_group = (0..vote_options.get_number_of_options()).fold(
        Flex::column(),
        |column, i| column.with_child(
            Either::new(
                move |data: &Params, _env: &Env| data.options_toggle.0[i],
                Either::new(
                    move |data: &Params, _env: &Env| data.options_result.0[i],
                    Label::new(
                        move |data: &Params, _env: &Env| {
                            let mut label = "-> ".to_owned();
                            label.push_str(data.options.split(",").collect::<Vec<&str>>()[i]);
                            label
                        }
                    ).center().expand_width().border(Color::rgb(0.0, 0.5, 0.0), 2.0).rounded(5.0).padding(10.0),
                    Label::new(
                        move |data: &Params, _env: &Env| {
                            let mut label = "-> ".to_owned();
                            label.push_str(data.options.split(",").collect::<Vec<&str>>()[i]);
                            label
                        }
                    ).center().expand_width().border(Color::rgb(0.5, 0.0, 0.0), 2.0).rounded(5.0).padding(10.0),
                ),
                Either::new(
                    move |data: &Params, _env: &Env| data.options_result.0[i],
                    Label::new(
                        move |data: &Params, _env: &Env| data.options.split(",").collect::<Vec<&str>>()[i].to_string()
                    ).center().expand_width().center().border(Color::rgb(0.0, 0.5, 0.0), 2.0).rounded(5.0).padding(10.0),
                    Label::new(
                        move |data: &Params, _env: &Env| data.options.split(",").collect::<Vec<&str>>()[i].to_string()
                    ).center().expand_width().center().border(Color::rgb(0.5, 0.0, 0.0), 2.0).rounded(5.0).padding(10.0),

                ),
            )
        )
    );

    let to_vote_section = Flex::column()
        .with_child(Label::new(|data: &Params, _env: &Env| {
            if data.is_confirmed {
                "Voted, wait to compute the result!".to_string()
            } else {
                "Options:".to_string()
            }
        }).padding(10.0))
        .with_child(buttons_group)
        .with_child(Button::new("Confirm votes").on_click(
            move |ctx: &mut EventCtx, data: &mut Params, _env: &Env| {
                data.is_confirmed = true;
                ctx.submit_command(command::VOTE.with(data.options_toggle.0.clone()))
            }
        ));

    let results_section = Flex::column()
        .with_child(Label::new("Results:").padding(10.0))
        .with_child(label_group);

    Flex::column()
        .with_child(Either::new(
            move |data: &Params, _env: &Env| data.is_computed,
            results_section,
            to_vote_section
        ))
		.controller(controller::VoteChoiceController::new(stream, vote_options))
}
