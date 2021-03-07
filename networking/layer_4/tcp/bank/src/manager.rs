use std::convert::From;
use std::net::TcpStream;
use std::str;
use std::sync::{Mutex, Arc};
use std::io::{Read, Write};

use crate::account::{self, Account, AccountStore, Result};

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

enum Status {
    Success,
    Failure,
}

struct Response {
    status: Status,
    msg: String,
    acct: Option<Account>
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

    fn parse_up_string(self, up: &str) -> (&str, &str) {
        let up: Vec<&str> = up.split(":").collect();
        (up[0], up[1])
    }

    fn handle_login(mut self, user: &str, password: &str) -> Response {
        let accts = self.accounts.lock().unwrap();
        if self.user.is_some() {
            return Response { status: Status::Failure, msg: "already logged in".to_string(), acct: None }
        }

        match accts.get_account(user.to_string()) {
            Ok(acct) => {
                if acct.is_correct_password(password) {
                    self.authed = true;
                    self.user = Some(user.to_string());
                    return Response { status: Status::Success, msg: "login successful!".to_string(), acct: Some(acct) }
                } else {
                    return Response { status: Status::Success, msg: "incorrect password".to_string(), acct: None }
                }
                
            }
            Err(_) => Response { status: Status::Failure, msg: "no such account".to_string(), acct: None }
        }
    }

    fn handle_create(mut self, user: String, password: String) -> Response {
        Response { status: Status::Failure, msg: "unimplemented".to_string(), acct: None }
    }

    fn handle_update(mut self, amount: f32) -> Response {
        Response { status: Status::Failure, msg: "unimplemented".to_string(), acct: None }
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
                                let (user, password) = self.parse_up_string(data);
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
