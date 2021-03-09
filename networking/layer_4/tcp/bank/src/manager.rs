use std::convert::From;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};

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
    acct: Option<Account>,
}

#[derive(Debug, Clone)]
pub struct Session {
    authed: bool,
    user: Option<String>,
    command: Option<Command>,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            authed: false,
            user: None,
            command: None,
        }
    }
}

pub struct SessionManager {
    accounts: Arc<Mutex<AccountStore>>,
}

fn parse_up_string(up: &str) -> (&str, &str) {
    let up: Vec<&str> = up.split(":").collect();
    (up[0], up[1])
}

impl SessionManager {
    pub fn new(accounts: Arc<Mutex<AccountStore>>) -> SessionManager {
        SessionManager { accounts }
    }

    fn handle_login(
        self,
        accounts: &Arc<Mutex<AccountStore>>,
        mut session: Box<Session>,
        user: &str,
        password: &str,
    ) -> (Box<Session>, Response) {
        let accts = accounts.lock().unwrap();
        if session.user.is_some() {
            return (
                session,
                Response {
                    status: Status::Failure,
                    msg: "already logged in".to_string(),
                    acct: None,
                },
            );
        }

        match accts.get_account(user.to_string()) {
            Ok(acct) => {
                if acct.is_correct_password(password) {
                    session.authed = true;
                    session.user = Some(user.to_string());
                    return (
                        session,
                        Response {
                            status: Status::Success,
                            msg: "login successful!".to_string(),
                            acct: Some(acct),
                        },
                    );
                } else {
                    return (
                        session,
                        Response {
                            status: Status::Success,
                            msg: "incorrect password".to_string(),
                            acct: None,
                        },
                    );
                }
            }
            Err(_) => (
                session,
                Response {
                    status: Status::Failure,
                    msg: "no such account".to_string(),
                    acct: None,
                },
            ),
        }
    }

    fn handle_create(mut self, user: String, password: String) -> Response {
        Response {
            status: Status::Failure,
            msg: "unimplemented".to_string(),
            acct: None,
        }
    }

    fn handle_update(mut self, amount: f32) -> Response {
        Response {
            status: Status::Failure,
            msg: "unimplemented".to_string(),
            acct: None,
        }
    }

    pub fn handle_stream(&mut self, mut stream: TcpStream) {
        let mut session = Box::new(Session::default());
        let mut buf = [0; 1024];
        while match stream.read(&mut buf) {
            Ok(n) => {
                // parse the response as a string from some bytes
                stream.write(&buf[0..n]).unwrap();
                let data = match str::from_utf8(&buf[0..n]) {
                    Ok(s) => s,
                    Err(_) => "nope",
                };

                match session.command {
                    Some(cmd) => {
                        // awaiting data for a command
                        match cmd {
                            Command::Login => {
                                let (user, password) = parse_up_string(data);
                                let (s, res) =
                                    self.handle_login(&self.accounts, session, user, password);
                                session = s; // can you do this in one line?
                            }
                            Command::Create => {}
                            Command::Update => {}
                            Command::Quit => {}
                            Command::Unimpl => {}
                        }
                    }
                    None => {
                        // accepting commands
                        session.command = Some(Command::from(data));
                    }
                }
                true
            }
            Err(e) => false,
        } {}
    }
}
