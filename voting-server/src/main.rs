use std::thread;
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_VOTERS_COUNT: AtomicUsize = AtomicUsize::new(0);
static EXPECTED_VOTERS: usize = 3;


fn initialize_client(mut stream: TcpStream) {

    let options = b"Ala,Kot,Kasia,Someone";
    stream.write(options).unwrap();

    let mut data = [0 as u8; 500];
    match stream.read(&mut data) {
        Ok(size) => {
            println!("{}", from_utf8(&data[0..size]).unwrap());
            if from_utf8(&data[0..size]).unwrap() == "READY" {
                GLOBAL_VOTERS_COUNT.fetch_add(1, Ordering::SeqCst);
                if GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst) >= EXPECTED_VOTERS {       
                    match TcpStream::connect("localhost:3333") {
                        Ok(_) => {
                            println!("All ready!");
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

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("Server listening on port 3333");

    let mut voters_streams: Vec<(TcpStream, SocketAddr)> = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                println!("New voter: {}", stream.peer_addr().unwrap());
                println!("{}", GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst));
                if GLOBAL_VOTERS_COUNT.load(Ordering::SeqCst) >= EXPECTED_VOTERS {
                    stream.shutdown(Shutdown::Both);
                    println!("All have voted!");

                    let mut voters_streams_tmp: Vec<(TcpStream, SocketAddr)> = Vec::new();
                    for (next_stream, next_addr) in &voters_streams {
                        voters_streams_tmp.push((next_stream.try_clone().unwrap(), next_addr.clone()));
                    }
                    for (next_stream, next_addr) in &voters_streams {
                        voters_streams_tmp.iter();
                        let streams_clones: Vec<TcpStream> = voters_streams_tmp.iter()
                            .filter(|(_, other_addr)| next_addr != other_addr)
                            .map(|(other_stream, _)| other_stream.try_clone().unwrap()).collect();
                        let next_stream_clone = next_stream.try_clone().unwrap();
                        thread::spawn(move || proxy_data(next_stream_clone, streams_clones));
                    }
                } else {
                    voters_streams.push((stream.try_clone().unwrap(), addr));
                    thread::spawn(move || initialize_client(stream));
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
