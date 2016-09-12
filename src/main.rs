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
        .version("0.1.0")
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("all")
            .short("a")
            .long("all")
            .help("Lists all servers, even those with no players"))
        .arg(Arg::with_name("reverse")
            .short("r")
            .long("reverse")
            .help("Reverses result order"))
        .arg(Arg::with_name("no_stats")
            .short("s")
            .long("no-stats")
            .help("Don't display overall stats in the end of the list"))
        .arg(Arg::with_name("length")
            .short("l")
            .long("length")
            .value_name("N")
            .help("The length of the server name"))
        .arg(Arg::with_name("SEARCH")
            .index(1)
            .help("An address to search for"))
        .get_matches();
    let show_all = args.is_present("all");
    let search = args.value_of("SEARCH");

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

    let cols:usize = match args.value_of("length") {
        None => match Command::new("stty").arg("size").arg("-F").arg("/dev/stderr").stderr(Stdio::inherit()).output() {
            Ok(out) => String::from_utf8(out.stdout).unwrap().split_whitespace().last().unwrap().parse().unwrap(),
            Err(_) => 80,
        },
        Some(l) => l.parse().expect("Please enter a valid length value")
    };

    let mut playing_count: u16 = 0;
    let mut player_count: u32 = 0;

    for server in servers {
        if server.players == 0 && !show_all {
            continue;
        }

        if search.is_some() {
            // We are searching for a string
            let searching = search.unwrap();

            let last_char = searching.chars().rev().next().unwrap();
            if last_char == ':' && !server.address.starts_with(searching) {
                // If the last character is a :, make sure our string starts
                // with the search query, without having any other characters
                // before
                continue;
            } else if last_char != ':' && !server.address.contains(searching) {
                continue;
            }
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

    if !args.is_present("no_stats") {
        println!("\n{} active server{}, {} server{} found, {} player{} total",
            playing_count, if playing_count != 1 { "s" } else { "" },
            server_count, if server_count != 1 { "s" } else { "" },
            player_count, if player_count != 1 { "s" } else { "" }
        );
    }
}
