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
    Noop,
    Unimpl,
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.trim_end() {
            "create" => Command::Create,
            "update" => Command::Update,
            "login" => Command::Login,
            "quit" => Command::Quit,
            "" => Command::Noop,
            _ => Command::Unimpl,
        }
    }
}

fn get_command_response(cmd: Command) -> String {
    let s = match cmd {
        Command::Create => "give me a <user>:<password>\n",
        Command::Login => "give me a <user>:<password>\n",
        Command::Update => "todo",
        Command::Quit => "terminating...\n",
        Command::Noop => "continuing...\n",
        Command::Unimpl => "unimplemented!\n",
    };
    s.to_string()
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
        &self,
        mut session: Box<Session>,
        user: &str,
        password: &str,
    ) -> (Box<Session>, Response) {
        if session.user.is_some() {
            return (
                session,
                Response {
                    status: Status::Failure,
                    msg: "already logged in\n".to_string(),
                    acct: None,
                },
            );
        }

        let account = self
            .accounts
            .lock()
            .unwrap()
            .get_account(user.to_string())
            .clone();
        match account {
            Some(acct) => {
                if acct.is_correct_password(password) {
                    session.authed = true;
                    session.user = Some(user.to_string());
                    return (
                        session,
                        Response {
                            status: Status::Success,
                            msg: "login successful!\n".to_string(),
                            acct: Some(acct),
                        },
                    );
                } else {
                    return (
                        session,
                        Response {
                            status: Status::Success,
                            msg: "incorrect password\n".to_string(),
                            acct: None,
                        },
                    );
                }
            }
            None => (
                session,
                Response {
                    status: Status::Failure,
                    msg: "no such account\n".to_string(),
                    acct: None,
                },
            ),
        }
    }

    fn handle_create(
        &self,
        mut session: Box<Session>,
        user: &str,
        password: &str,
    ) -> (Box<Session>, Response) {
        let mut accounts = self.accounts.lock().unwrap();

        let account = accounts.get_account(user.to_string()).clone();
        match account {
            Some(acct) => {
                return (
                    session,
                    Response {
                        status: Status::Failure,
                        msg: "account exists\n".to_string(),
                        acct: None,
                    },
                )
            }
            None => {
                let acct = Account::new(password);
                accounts.add_account(user.to_string(), acct);
                session.user = Some(user.to_string());
                (
                    session,
                    Response {
                        status: Status::Success,
                        msg: "account created\n".to_string(),
                        acct: Some(acct),
                    },
                )
            }
        }
    }

    fn handle_update(mut self, amount: f32) -> Response {
        Response {
            status: Status::Failure,
            msg: "unimplemented\n".to_string(),
            acct: None,
        }
    }

    pub fn handle_stream(&mut self, mut stream: TcpStream) {
        let mut session = Box::new(Session::default());
        let mut buf = [0; 1024];

        let open_msg = "Welcome! Supported methods: create, login, update, quit.\n".to_string();
        stream.write(open_msg.as_bytes()).unwrap();

        while match stream.read(&mut buf) {
            Ok(n) => {
                let data = match str::from_utf8(&buf[0..n]) {
                    Ok(s) => s,
                    Err(_) => "nope",
                };
                
                match session.command {
                    Some(cmd) => {
                        match cmd {
                            Command::Login => {
                                let (user, password) = parse_up_string(data);
                                let (s, res) = self.handle_login(session, user, password);
                                session = s;
                                stream.write(res.msg.as_bytes()).unwrap();
                            }
                            Command::Create => {
                                let (user, password) = parse_up_string(data);
                                let (s, res) = self.handle_create(session, user, password);
                                session = s;
                                stream.write(res.msg.as_bytes()).unwrap();
                            }
                            Command::Update => {}
                            Command::Quit => {}
                            Command::Unimpl => {}
                            Command::Noop => {}
                        }
                        session.command = None;
                        for el in buf.iter_mut() {
                            *el = 0;
                        }
                    }
                    None => {
                        session.command = Some(Command::from(data));
                        let msg = get_command_response(session.command.unwrap());
                        stream.write(msg.as_bytes()).unwrap();
                    }
                }
                true
            }
            Err(e) => false,
        } {}
    }
}
