#![allow(non_snake_case)]

use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::env;
use std::convert::TryInto;


static GLOBAL_VOTERS_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct VoteOptions {
    expected_voters: usize,
    vote_treshold: usize,
    options: String,
}

fn initialize_client(mut stream: TcpStream, VOTE_OPTIONS: VoteOptions) {

    stream.write(&(VOTE_OPTIONS.expected_voters as u32).to_be_bytes()).unwrap();

    stream.write(&(VOTE_OPTIONS.vote_treshold as u32).to_be_bytes()).unwrap();

    stream.write(VOTE_OPTIONS.options.as_bytes()).unwrap();

    let mut data = [0 as u8; 500];
    match stream.read(&mut data) {
        Ok(size) => {
            println!("{}", from_utf8(&data[0..size]).unwrap());
            if from_utf8(&data[0..size]).unwrap() == "VOTED" {
                GLOBAL_VOTERS_COUNT.fetch_add(1, Ordering::SeqCst);
                if GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst) >= VOTE_OPTIONS.expected_voters {       
                    match TcpStream::connect("localhost:3333") {
                        Ok(_) => {
                            println!("All ready - from now on all data will be proxied between voters");
                        },
                        Err(e) => {
                            panic!("Failed to connect: {}", e);
                        }
                    }
                }
            }
        },
        Err(_) => println!("Error"),
    }
}

fn proxy_data(mut read_stram: TcpStream, write_streams: Vec<TcpStream>) {
    let mut data = [0 as u8; 500];

    let options = b"Proxy Opened!";
    read_stram.write(options).unwrap();

    while match read_stram.read(&mut data) {
        Ok(size) => {
            println!("{}", from_utf8(&data[0..size]).unwrap());
            for mut w_stm in write_streams.iter() {
                println!("{}", from_utf8(&data[0..size]).unwrap());
                w_stm.write(&data[0..size]).unwrap();
            }
            true
        },
        Err(_) => {
            println!("Error");
            false
        },
    } {}
}

fn main() {

    let EXPECTED_VOTERS: usize = match env::args().collect::<Vec<String>>().get(1) {
        Some(expected_voters) => match expected_voters.parse::<usize>() {
            Ok(expected_voters) => expected_voters,
            _ => panic!("EXPECTED_VOTERS should be a non-negative integer!")
        },
        None => panic!("Specify program arguments: <expected_voters> <vote_treshold> <vote_options>"),
    };

    let VOTE_TRESHOLD: usize = match env::args().collect::<Vec<String>>().get(2) {
        Some(vote_treshold) => match vote_treshold.parse::<usize>() {
            Ok(vote_treshold) => vote_treshold,
            _ => panic!("VOTE_TRESHOLD should be a non-negative integer!")
        },
        None => panic!("Specify program arguments: <expected_voters> <vote_treshold> <vote_options>"),
    };

    let OPTIONS: String = match env::args().collect::<Vec<String>>().get(3) {
        Some(options) => options,
        None => panic!("Specify program arguments: <expected_voters> <vote_treshold> <vote_options>"),
    }.to_string();

    let VOTE_OPTIONS = VoteOptions {
        expected_voters: EXPECTED_VOTERS,
        vote_treshold: VOTE_TRESHOLD,
        options: OPTIONS,
    };

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server starting with options: number of voters: {};  vote treshold: {}; voting options: {}.", VOTE_OPTIONS.expected_voters, VOTE_OPTIONS.vote_treshold, VOTE_OPTIONS.options);

    let mut voters_streams: Vec<(TcpStream, usize)> = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New voter: {}", stream.peer_addr().unwrap());
                println!("{}", GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst));
                if GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst) >= EXPECTED_VOTERS {
                    println!("All voters have voted!");

                    let mut voters_streams_tmp: Vec<(TcpStream, usize)> = Vec::new();
                    for (next_stream, next_id) in &voters_streams {
                        voters_streams_tmp.push((next_stream.try_clone().unwrap(), next_id.clone()));
                    }
                    for (next_stream, next_id) in &voters_streams {
                        voters_streams_tmp.iter();
                        let streams_clones: Vec<TcpStream> = voters_streams_tmp.iter()
                            .filter(|(_, other_id)| next_id != other_id)
                            .map(|(other_stream, _)| other_stream.try_clone().unwrap()).collect();
                        let next_stream_clone = next_stream.try_clone().unwrap();
                        thread::spawn(move || proxy_data(next_stream_clone, streams_clones));
                    }
                } else {
                    // Receive party id
                    let mut data = [0 as u8; 4];
                    let id = match stream.try_clone().unwrap().read(&mut data) {
                        Ok(_) => {
                            let (id_bytes, _rest) = data.split_at(std::mem::size_of::<u32>());
                            u32::from_be_bytes(id_bytes.try_into().unwrap()) as usize
                        },
                        Err(_) => panic!("Error reciving id"),
                    };
                    voters_streams.push((stream.try_clone().unwrap(), id));
                    let cloned_vote_options = VOTE_OPTIONS.clone();
                    thread::spawn(move || initialize_client(stream, cloned_vote_options));
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
