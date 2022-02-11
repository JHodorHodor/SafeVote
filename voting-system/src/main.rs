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


#[derive(Clone, Data, Lens)]
struct Params {
    is_picked: bool,
    options: String,
}

fn main() {

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
            let input = (id as u32).to_be_bytes();
            stream.write(&input).unwrap();


            // Receive votig options
            let mut data = [0 as u8; 500];
            let number_of_voters = match stream.read(&mut data) {
                Ok(_) => {
                    let (id_bytes, _rest) = data.split_at(std::mem::size_of::<u32>());
                    u32::from_be_bytes(id_bytes.try_into().unwrap()) as usize
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };
            let vote_treshold = match stream.read(&mut data) {
                Ok(_) => {
                    let (id_bytes, _rest) = data.split_at(std::mem::size_of::<u32>());
                    u32::from_be_bytes(id_bytes.try_into().unwrap()) as usize
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };
            let voting_options = match stream.read(&mut data) {
                Ok(size) => {
                	from_utf8(&data[0..size]).unwrap().to_string()
                },
                Err(e) => panic!("Failed to receive data: {}", e)
            };

		    println!("Options received: number of voters: {};  vote treshold: {}; voting options: {}.", number_of_voters, vote_treshold, voting_options);

            // Init UI
		    let options_num = voting_options.split(",").collect::<Vec<&str>>().len();
		    let main_window = WindowDesc::new(move || ui_builder(options_num, stream, id, number_of_voters, vote_treshold))
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

fn ui_builder(options_num: usize, stream: TcpStream, id: usize, number_of_voters: usize, vote_treshold: usize) -> impl Widget<Params> {

    let buttons_group = (0..options_num).fold(
    	Flex::column(),
    	|column, i| column.with_child(
    		Button::new(
    			move |data: &Params, _env: &Env| data.options.split(",").collect::<Vec<&str>>()[i].to_string()
    		).on_click(
    			move |ctx: &mut EventCtx, data: &mut Params, _env| {
    				if !data.is_picked {
	    				data.is_picked = true;
	    				println!("Voted for {}", i);
						ctx.submit_command(command::VOTE.with(i as u8));
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
		.controller(controller::VoteChoiceController::new(stream, id, number_of_voters, vote_treshold))
}
