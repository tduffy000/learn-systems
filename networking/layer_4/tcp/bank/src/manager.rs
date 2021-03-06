use std::convert::From;
use std::net::TcpStream;
use std::str;
use std::sync::{Mutex, Arc};
use std::io::{Read, Write};

use crate::account::{Account, AccountStore, Result};

#[derive(Debug, Clone, Copy)]
enum Command {
    Create,
    Login,
    Update,
    Quit,
    Unimpl,
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "create" => Command::Create,
            "update" => Command::Update,
            "login" => Command::Login,
            "quit" => Command::Quit,
            _ => Command::Unimpl,
        }
    }
}

pub struct SessionManager {
    accounts: Arc<Mutex<AccountStore>>,
    authed: bool,
    user: Option<String>, // the key in AccountStore
    command_mode: Option<Command>,
}

impl SessionManager {

    pub fn new(accounts: Arc<Mutex<AccountStore>>) -> SessionManager {
        SessionManager { accounts, authed: false, user: None , command_mode: None}
    }

    fn handle_login(mut self, user: String, password: String) -> Result<Account> {
        
    }

    fn handle_create(mut self, user: String, password: String) -> Result<Account> {

    }

    fn handle_update(mut self, amount: f32) -> Result<Account> {

    }

    pub fn handle_stream(mut self, mut stream: TcpStream) {
        let mut buf = [0; 1024];
        while match stream.read(&mut buf) {
            Ok(n) => {
                
                // parse the response as a string from some bytes
                stream.write(&buf[0..n]).unwrap();
                let data = match str::from_utf8(&buf[0..n]) {
                    Ok(s) => s,
                    Err(_) => "nope",
                };

                match self.command_mode {
                    Some(cmd) => { // awaiting data for a command
                        match cmd {
                            Command::Login => {
                                let up: Vec<&str> = data.split(":").collect();
                                let user = up[0];
                                let password = up[1];
                                self.handle_login(user, password);
                            },
                            Command::Create => {

                            }, 
                            Command::Update => {

                            }, 
                            Command::Quit => {
                                
                            }, 
                            Command::Unimpl => {

                            },
                        }
                    },
                    None => { // accepting commands
                        self.command_mode = Some(Command::from(data))
                    },
                }
                true
            },
            Err(e) => {

                false
            },
        } {}
    }
}
