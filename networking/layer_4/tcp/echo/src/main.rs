use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Result, Write};

fn echo_handle(mut stream: TcpStream) {
    
    let mut buf = [0; 1024];
    while match stream.read(&mut buf) {
        Ok(n) => {
            stream.write(&buf[0..n]).unwrap();
            true
        },
        Err(e) => {
            println!("Got error, terminating connection to {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } 
    {}
}

fn main() -> Result<()> {

    let addr = "127.0.0.1:34245";
    println!("listening at {}", addr);
    let listener = TcpListener::bind(&addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    echo_handle(stream)
                });
            },
            Err(e) => {
                println!("Got error: {}", e);
            }
        }
    }
    
    drop(listener);
    Ok(())  
}