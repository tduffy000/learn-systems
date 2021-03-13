use std::io::Result;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

mod account;
mod manager;

use account::AccountStore;
use manager::SessionManager;

fn main() -> Result<()> {
    let addr = "127.0.0.1:34245";
    println!("listening at {}", addr);
    let listener = TcpListener::bind(&addr).unwrap();
    let accounts = Arc::new(Mutex::new(AccountStore::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection: {}", stream.peer_addr().unwrap());
                let accounts_clone = accounts.clone();
                thread::spawn(move || {
                    let mut session_mgr = SessionManager::new(accounts_clone);
                    session_mgr.handle_stream(stream);
                });
            }
            Err(e) => {
                println!("Got error: {}", e);
            }
        }
    }

    drop(listener);
    Ok(())
}
