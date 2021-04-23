use rtnetlink::packet::RouteMessage;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use crate::routing::{self, IpPrefix};

#[derive(Clone)]
pub struct Rib(pub Vec<RoutingInformationEntry>);

impl Rib {
    pub fn new(routing_table: Vec<RoutingInformationEntry>) -> LocRib {
        Rib(routing_table)
    }

    pub fn add_from_route_message(&mut self, routing_information: &mut Vec<RouteMessage>) {
        for rm in routing_information {
            if let Some(IpAddr::V4(gateway)) = rm.gateway() {

                let destnation_address = match rm.destination_prefix() {
                    Some((ip, prefix)) => {
                        if let IpAddr::V4(ip) = ip {
                            IpPrefix::new(ip, prefix)
                        } else {
                            panic!();
                        }
                    },
                    _ => panic!(),
                };

                let routing_information_entry = RoutingInformationEntry::new(
                    gateway, destnation_address,
                );

                self.0.push(routing_information_entry)
            }
        }
    }

    pub fn add(&mut self, routing_information: &mut Vec<RoutingInformationEntry>) {
        self.0.append(routing_information)
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
        self.add_from_route_message(&mut filtered_routing_information)
    }

    pub fn get_new_route(&self) -> Vec<RoutingInformationEntry> {
        self.0.clone()
    }

    pub fn lookup(&self, destnation: &Ipv4Addr) {
        ()
    }
}
#[derive(Clone)]
pub struct RoutingInformationEntry {
    pub nexthop: Ipv4Addr,
    pub destnation_address: IpPrefix,
}

impl RoutingInformationEntry {

    pub fn new(nexthop: Ipv4Addr, destnation_address: IpPrefix) -> Self {
        Self {nexthop, destnation_address}
    }
}

#[derive(Clone)]
enum Protocol {
    Bgp,
    Static,
}

pub type LocRib = Rib;
pub type AdjRibOut = Rib;
pub type AdjRibIn = Rib;
