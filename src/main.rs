extern crate hyper;
extern crate ansi_term;
extern crate clap;

use hyper::client::Client;
use std::io::Read;
use std::string::String;
use ansi_term::Colour::{Red, Green, Yellow, Cyan};
use std::process::Command;
use std::process::Stdio;
use std::cmp::Ordering;
use clap::{Arg, App, AppSettings, SubCommand};

struct Server {
    protocol: String,
    players: u8,
    observers: u8,
    address: String,
    name: String,
}

fn main() {
    let args = App::new("bzls-rust")
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("all")
            .short("a")
            .help("List all servers, even those with no players"))
        .arg(Arg::with_name("reverse")
            .short("r")
            .help("Reverse result order"))
        .get_matches();
    let show_all = args.is_present("all");

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

    let server_count = servers.len();

    servers.sort_by(|a, b| match b.players.cmp(&a.players) {
        Ordering::Equal => b.observers.cmp(&a.observers),
        c => c
    });

    if args.is_present("reverse") {
        servers.reverse();
    }

    let cols:usize = match Command::new("stty").arg("size").arg("-F").arg("/dev/stderr").stderr(Stdio::inherit()).output() {
        Ok(out) => String::from_utf8(out.stdout).unwrap().split_whitespace().last().unwrap().parse().unwrap(),
        Err(_) => 80,
    };

    let mut playing_count: u16 = 0;
    let mut player_count: u32 = 0;

    for server in servers {
        if server.players == 0 && !show_all {
            break;
        }

        if server.players != 0 {
            playing_count += 1;
            player_count += server.players as u32;
        }

        println!("[{}, {}]  {}  {}",
            Yellow.paint(format!("{:2}", server.players)),
            Cyan.paint(server.observers.to_string()),
            Green.paint(format!("{:1$}", server.name, cols/3)),
            Red.paint(server.address)
        );
    }

    println!("\n{} active server{}, {} server{} found, {} player{} total",
        playing_count, if playing_count != 1 { "s" } else { "" },
        server_count, if server_count != 1 { "s" } else { "" },
        player_count, if player_count != 1 { "s" } else { "" }
    );
}
