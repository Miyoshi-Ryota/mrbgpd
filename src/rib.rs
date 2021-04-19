use rtnetlink::packet::RouteMessage;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use crate::routing::IpPrefix;

#[derive(Clone)]
pub struct Rib(pub Vec<RouteMessage>);

impl Rib {
    pub fn new(routing_table: Vec<RouteMessage>) -> LocRib {
        Rib(routing_table)
    }

    pub fn add(&mut self, routing_information: &mut Vec<RouteMessage>) {
        self.0.append(routing_information);
    }

    pub fn add_route_filtered_by_network_command
    (&mut self, routing_information: &Vec<RouteMessage>, network_command: &IpPrefix) {
        let mut filtered_routing_information = vec![];
        for entry in routing_information {
            if let Some(dest) = entry.destination_prefix() {
                if let IpAddr::V4(v) = dest.0 {
                    let dest = IpPrefix::new(v, dest.1);
                    if network_command.does_include(&dest) {
                        filtered_routing_information.push(entry.clone());
                    }
                }
            }
        }
        self.add(&mut filtered_routing_information)
    }

    pub fn get_new_route(&self) -> Vec<RouteMessage> {
        self.0.clone()
    }

    pub fn lookup(&self, destnation: &Ipv4Addr) {
        ()
    }
}

pub type LocRib = Rib;
pub type AdjRibOut = Rib;
