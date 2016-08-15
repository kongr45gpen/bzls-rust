extern crate hyper;
extern crate ansi_term;

use hyper::client::Client;
use std::io::Read;
use std::string::String;
use ansi_term::Colour::{Red, Green, Yellow, Cyan};

struct Server {
    protocol: String,
    players: u8,
    observers: u8,
    address: String,
    name: String,
}

fn main() {
    let mut servers: Vec<Server> = Vec::new();

    let client = Client::new();

    let mut res = client.get("https://my.bzflag.org/db/?action=LIST&listformat=plain").send().unwrap();
    assert_eq!(res.status, hyper::Ok);

    let mut data = String::new();
    res.read_to_string(&mut data).expect("Failed to read data");

    for line in data.lines() {       
        let server: Vec<&str> = line.split(' ').collect();

        let data = server[2];
        let players = [
            u8::from_str_radix(&data[34..36], 16).unwrap(), // rogue
            u8::from_str_radix(&data[38..40], 16).unwrap(), // red
            u8::from_str_radix(&data[42..44], 16).unwrap(), // green
            u8::from_str_radix(&data[46..48], 16).unwrap(), // blue
            u8::from_str_radix(&data[50..52], 16).unwrap(), // purple
        ];
        let players: u8 = players.iter().fold(0u8, |a, &b| a + b);
        let observers = u8::from_str_radix(&data[54..56], 16).unwrap();

        let server = Server {
            protocol: server[1].to_string(),
            players: players,
            observers: observers,
            address: server[0].to_string(),
            name: (&server[4..]).join(" ").to_string()
        };

        servers.push(server);
    }

    servers.sort_by(|a, b| b.players.cmp(&a.players));

    for server in servers {
        if server.players == 0 {
            break;
        }

        println!("[{}, {}]  {}  {}",
            Yellow.paint(format!("{:2}", server.players)),
            Cyan.paint(server.observers.to_string()),
            Green.paint(format!("{:50}", server.name)),
            Red.paint(server.address)
        );
    }
}
