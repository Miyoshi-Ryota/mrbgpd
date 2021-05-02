#![feature(str_split_as_str)]
#![feature(exclusive_range_pattern)]

pub mod bgp;
pub mod finite_state_machine;
pub mod routing;
pub mod rib;
pub mod peer;

use std::{net::Ipv4Addr, str::FromStr, string::ParseError};
use crate::bgp::AutonomousSystemNumber;
use crate::routing::IpPrefix;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    as_number: AutonomousSystemNumber,
    my_ip_addr: Ipv4Addr,
    remote_as_number: AutonomousSystemNumber,
    remote_ip_addr: Ipv4Addr,
    mode: Mode,
    advertisement_network: IpPrefix,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "active" {
            Ok(Mode::Active)
        } else if s == "passive" {
            Ok(Mode::Passive)
        } else {
            Err(String::from("ParseError"))
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum Mode {
    Active,
    Passive,
}

impl Config {
    pub fn parse_args(args: Vec<&str>) -> Config {
        let as_number = AutonomousSystemNumber::new(
            args[1].parse().expect("cannot parse arg 1"));
        let my_ip_addr: Ipv4Addr = args[2].parse().expect("cannot parse arg 2");
        let remote_as_number = AutonomousSystemNumber::new(
            args[3].parse().expect("cannot parse arg 3"));
        let remote_ip_addr: Ipv4Addr = args[4].parse().expect("cannot parse arg 4");
        let mode: Mode = args[5].parse().expect("cannot parse arg 5");
        let advertisement_network = args[6].parse().expect("cannot parse arg6");

        Config {
            as_number,
            my_ip_addr,
            remote_as_number,
            remote_ip_addr,
            mode,
            advertisement_network,
        }
    }

    pub fn parse_from_file(filename: &str) -> Vec<Config> {
        let mut result = vec![];
        if let Ok(lines) = read_lines(filename) {
            for line in lines {
                if let Ok(ip) = line {
                    let config_args: Vec<&str> = ip.split(" ").collect();
                    result.push(Config::parse_args(config_args));
                }
            }
        }
        result
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
