#![allow(non_snake_case)]

use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::env;
use std::convert::TryInto;
use std::collections::HashMap;


static GLOBAL_VOTERS_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct VoteOptions {
    expected_voters: usize,
    vote_threshold: usize,
    options: String,
}

fn initialize_client(mut stream: TcpStream, VOTE_OPTIONS: VoteOptions) {

    stream.write(&(VOTE_OPTIONS.expected_voters as u32).to_be_bytes()).unwrap();

    stream.write(&(VOTE_OPTIONS.vote_threshold as u32).to_be_bytes()).unwrap();

    stream.write(VOTE_OPTIONS.options.as_bytes()).unwrap();

    let mut data = [0 as u8; 500];
    match stream.read(&mut data) {
        Ok(size) => {
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

fn process_packet(data: &[u8], _id: usize, write_streams: &HashMap<usize, TcpStream>) -> bool {
    let (id_bytes, data) = data.split_at(std::mem::size_of::<u32>());
    let rcvr_id = u32::from_be_bytes(id_bytes.try_into().unwrap()) as usize;

    match write_streams.get(&rcvr_id) {
        Some(mut stream) => {
            stream.write(&data[0..32]).unwrap();
            true
        },
        None => {
            println!("Error");
            false
        }
    }
}

fn proxy_data(mut read_stream: TcpStream, id: usize, write_streams: HashMap<usize, TcpStream>) {
    let start_protocol_info = b"Proxy Opened!";
    read_stream.write(start_protocol_info).unwrap();

    let mut data = [0 as u8; std::mem::size_of::<u32>() + 32];

    while match read_stream.read_exact(&mut data) {
        Ok(_) => {
            process_packet(&data[..], id, &write_streams)
        },
        Err(_) => {
            println!("Channel closed");
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
        None => panic!("Specify program arguments: <expected_voters> <vote_threshold> <vote_options>"),
    };

    let VOTE_THRESHOLD: usize = match env::args().collect::<Vec<String>>().get(2) {
        Some(vote_threshold) => match vote_threshold.parse::<usize>() {
            Ok(vote_threshold) => vote_threshold,
            _ => panic!("VOTE_THRESHOLD should be a non-negative integer!")
        },
        None => panic!("Specify program arguments: <expected_voters> <vote_threshold> <vote_options>"),
    };

    let OPTIONS: String = match env::args().collect::<Vec<String>>().get(3) {
        Some(options) => options,
        None => panic!("Specify program arguments: <expected_voters> <vote_threshold> <vote_options>"),
    }.to_string();

    let VOTE_OPTIONS = VoteOptions {
        expected_voters: EXPECTED_VOTERS,
        vote_threshold: VOTE_THRESHOLD,
        options: OPTIONS,
    };

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server starting with options: number of voters: {};  vote threshold: {}; voting options: {}.", VOTE_OPTIONS.expected_voters, VOTE_OPTIONS.vote_threshold, VOTE_OPTIONS.options);

    let mut voters_streams: Vec<(TcpStream, usize)> = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst) >= EXPECTED_VOTERS {
                    println!("All voters have voted!");

                    let mut voters_streams_tmp: Vec<(TcpStream, usize)> = Vec::new();
                    for (next_stream, next_id) in &voters_streams {
                        voters_streams_tmp.push((next_stream.try_clone().unwrap(), next_id.clone()));
                    }
                    for (next_stream, next_id) in &voters_streams {

                        let mut other_streams_map = HashMap::new();
                        voters_streams_tmp.iter()
                            .filter(|(_, other_id)| next_id != other_id)
                            .for_each(|(other_stream, other_id)| {
                                other_streams_map.insert(other_id.clone(), other_stream.try_clone().unwrap());
                            }
                        );
                        let next_stream_clone = next_stream.try_clone().unwrap();
                        let next_id_clone = next_id.clone();
                        thread::spawn(move || proxy_data(next_stream_clone, next_id_clone, other_streams_map));
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
                    println!("Connected {}", id);
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
