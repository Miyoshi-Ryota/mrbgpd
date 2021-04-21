#![feature(str_split_as_str)]
#![feature(exclusive_range_pattern)]

pub mod bgp;
pub mod finite_state_machine;
pub mod routing;
pub mod rib;
use std::{net::Ipv4Addr, str::FromStr, string::ParseError};
use crate::bgp::AutonomousSystemNumber;
use crate::routing::IpPrefix;

#[derive(Debug)]
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
#[derive(Debug)]
enum Mode {
    Active,
    Passive,
}

impl Config {
    pub fn parse_args(args: Vec<String>) -> Config {
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
}
