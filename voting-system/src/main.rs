use druid::{
	widget::{prelude::*, Button, Flex, Label, Either},
	AppLauncher, Widget, WidgetExt, WindowDesc, Data, Lens, Env,
};
use std::net::{TcpStream};
use std::io::{Read};
use std::str::from_utf8;
use std::env;

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

            let mut data = [0 as u8; 500];
            let text = match stream.read(&mut data) {
                Ok(size) => {
                	from_utf8(&data[0..size]).unwrap().to_string()
                },
                Err(e) => {
                    panic!("Failed to receive data: {}", e);
                }
            };
		    println!("Options received.");

		    let options_num = text.split(",").collect::<Vec<&str>>().len();
		    let main_window = WindowDesc::new(move || ui_builder(options_num, stream, id))
		        .title("Take a vote!")
		        .window_size((300.0, 500.0));

		    println!("{}", text);
		    let params = Params {
		        is_picked: false,
		        options: text,
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

fn ui_builder(options_num: usize, stream: TcpStream, id: usize) -> impl Widget<Params> {

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
		.controller(controller::VoteChoiceController::new(stream, id))
}
