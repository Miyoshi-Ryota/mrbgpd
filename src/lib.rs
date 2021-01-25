pub mod bgp;
pub mod finite_state_machine;

use std::net::Ipv4Addr;
use crate::bgp::AutonomousSystemNumber;

#[derive(Debug)]
pub struct Config {
    as_number: AutonomousSystemNumber,
    my_ip_addr: Ipv4Addr,
    remote_as_number: AutonomousSystemNumber,
    remote_ip_addr: Ipv4Addr,
}

impl Config {
    pub fn parse_args(args: Vec<String>) -> Config {
        let as_number = AutonomousSystemNumber::new(
            args[1].parse().expect("cannot parse arg 1"));
        let my_ip_addr: Ipv4Addr = args[2].parse().expect("cannot parse arg 2");
        let remote_as_number = AutonomousSystemNumber::new(
            args[3].parse().expect("cannot parse arg 3"));
        let remote_ip_addr: Ipv4Addr = args[4].parse().expect("cannot parse arg 4");
        Config {
            as_number,
            my_ip_addr,
            remote_as_number,
            remote_ip_addr,
        }
    }
}
