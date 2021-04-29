use rtnetlink::packet::RouteMessage;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use crate::{bgp::{BgpUpdateMessage, PathAttribute}, routing::{self, IpPrefix}};
use std::cmp::PartialEq;

#[derive(Clone, Debug)]
pub struct Rib(pub Vec<RoutingInformationEntry>);

impl Rib {
    pub fn new(routing_table: Vec<RoutingInformationEntry>) -> LocRib {
        Rib(routing_table)
    }

    pub fn add_from_route_message(&mut self, routing_information: &mut Vec<RouteMessage>) {
        println!("now in Rib.add_from_route_message {:?}", routing_information);
        for rm in routing_information {
            println!("one route message: {:?}", rm);
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
                    gateway, destnation_address, RoutingInformationStatus::Updated,
                );
                println!("Add from route message. Try to add route: {:?}", routing_information_entry);
                self.add_if_needed(routing_information_entry)
            }
        }
    }

    pub fn add(&mut self, routing_information: Vec<RoutingInformationEntry>) {
        for routing in routing_information {
            self.add_if_needed(routing);
        }
    }

    fn add_if_needed(&mut self, one_route: RoutingInformationEntry) {
        if !self.0.contains(&one_route) {
            self.0.push(one_route);
        } else {
            println!("the rib already have had the route, {:?}.", one_route);
            println!("and now rib is {:?}", self.0);
        }
    }

    pub fn does_have_new_route(&self) -> bool {
        for route in &self.0 {
            if !(route.status == RoutingInformationStatus::UnChanged) {
                return true
            }
        };
        return false
    }

    pub fn change_state_of_all_routing_information_to_unchanged(&mut self) {
        for entry in &mut self.0 {
            entry.status = RoutingInformationStatus::UnChanged;
        }
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

    pub fn add_from_update_message(&mut self, update_message: BgpUpdateMessage) {
        let mut nexthop = Ipv4Addr::new(0, 0, 0, 0);
        for path_attribute in &update_message.path_attributes {
            match path_attribute {
                &PathAttribute::NextHop(ip_addr) => {
                    nexthop = ip_addr;
                },
                _ => (),
            }
        }
        let routing_information: Vec<RoutingInformationEntry> = update_message.network_layer_reachability_information.into_iter().map(
            |dest| RoutingInformationEntry::new(nexthop, dest, RoutingInformationStatus::Updated)).collect();
        self.add(routing_information);
    }


    pub fn get_new_route(&self) -> Vec<RoutingInformationEntry> {
        self.0.clone()
    }

    pub fn lookup(&self, destnation: &Ipv4Addr) {
        ()
    }
}
#[derive(Clone, Debug)]
pub struct RoutingInformationEntry {
    pub nexthop: Ipv4Addr,
    pub destnation_address: IpPrefix,
    pub status: RoutingInformationStatus,
}

impl PartialEq for RoutingInformationEntry {
    fn eq(&self, other: &RoutingInformationEntry) -> bool {
        self.nexthop == other.nexthop
        && self.destnation_address == other.destnation_address
    }
}

#[derive(Clone, Copy, std::cmp::PartialEq, Debug)]
pub enum RoutingInformationStatus {
    Withdrawn,
    Updated,
    UnChanged,
}

impl RoutingInformationEntry {

    pub fn new(nexthop: Ipv4Addr, destnation_address: IpPrefix, status: RoutingInformationStatus) -> Self {
        Self {nexthop, destnation_address, status}
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
