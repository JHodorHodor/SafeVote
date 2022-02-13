use druid::{
	widget::{prelude::*, Button, Flex, Label, Either},
	AppLauncher, Widget, WidgetExt, WindowDesc, Data, Lens, Env,
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
    is_picked: bool,
    options: String,
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
            let mut data = [0 as u8; 4];
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

            let vote_options = controller::VoteOptions {
                id: id,
                number_of_voters: number_of_voters,
                vote_threshold: vote_threshold,
                number_of_options: voting_options.split(",").collect::<Vec<&str>>().len(),
            };

		    println!("Options received: number of voters: {};  vote threshold: {}; voting options: {}.", number_of_voters, vote_threshold, voting_options);

            // Init UI
		    let main_window = WindowDesc::new(move || ui_builder(stream, vote_options))
		        .title("Take a vote!")
		        .window_size((300.0, 500.0));

		    println!("{}", voting_options);
		    let params = Params {
		        is_picked: false,
		        options: voting_options,
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
    println!("UI");
    let buttons_group = (0..vote_options.number_of_options).fold(
    	Flex::column(),
    	|column, i| column.with_child(
    		Button::new(
    			move |data: &Params, _env: &Env| data.options.split(",").collect::<Vec<&str>>()[i].to_string()
    		).on_click(
    			move |ctx: &mut EventCtx, data: &mut Params, _env| {
    				if !data.is_picked {
	    				data.is_picked = true;
	    				println!("Voted for {}", i);
						ctx.submit_command(command::VOTE.with(i as u16));
    				}
    			}
    		)
    	)
    );

    Flex::column()
        .with_child(Label::new(|data: &Params, _env: &Env| {
        	if data.is_picked {
        		"Voted, wait to compute the result!".to_string()
        	} else {
        		"Options:".to_string()
        	}
        }))
        .with_child(Either::new(
            |data: &Params, _env: &Env| data.is_picked,
            Flex::column(),
            buttons_group,
        ))
		.controller(controller::VoteChoiceController::new(stream, vote_options))
}
