use rtnetlink::packet::RouteMessage;
use std::net::Ipv4Addr;
use std::net::IpAddr;
use crate::{bgp::{AutonomousSystemNumber, BgpUpdateMessage, PathAttribute}, routing::{self, IpPrefix}};
use std::cmp::PartialEq;
use crate::bgp::AsPath;

#[derive(Clone, Debug)]
pub struct Rib(pub Vec<RoutingInformationEntry>);

impl Rib {
    pub fn new(routing_table: Vec<RoutingInformationEntry>) -> LocRib {
        Rib(routing_table)
    }

    pub fn add_from_route_message(&mut self, routing_information: &mut Vec<RouteMessage>, path_attributes: Vec<PathAttribute>) {
        println!("now in Rib.add_from_route_message {:?}", routing_information);
        for rm in routing_information {
            println!("the route gateway: {:?}", rm.gateway());
            if let Some((ip, prefix)) = rm.destination_prefix() {
                let destination_address = if let IpAddr::V4(ip) = ip {
                    IpPrefix::new(ip, prefix)
                } else {
                    panic!();
                };
                let gateway =  match rm.gateway() {
                    Some(IpAddr::V4(gateway)) => gateway,
                    _ => Ipv4Addr::new(0, 0, 0, 0),
                };
                let routing_information_entry = RoutingInformationEntry::new(
                    gateway,
                    destination_address,
                    RoutingInformationStatus::Updated,
                    path_attributes.clone(),
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

    pub fn add_from_update_message(&mut self, update_message: BgpUpdateMessage, my_as_number: &AutonomousSystemNumber) {
        let mut nexthop = Ipv4Addr::new(0, 0, 0, 0);
        for path_attribute in &update_message.path_attributes {
            match &path_attribute {
                &PathAttribute::NextHop(ip_addr) => {
                    nexthop = *ip_addr;
                },
                _ => (),
            }
        }
        let routing_information: Vec<RoutingInformationEntry> = update_message.network_layer_reachability_information.iter().map(
            |dest| RoutingInformationEntry::new(nexthop, *dest, RoutingInformationStatus::Updated, update_message.path_attributes.clone())).collect();
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
    pub path_attributes: Vec<PathAttribute>,
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

    pub fn new(nexthop: Ipv4Addr, destnation_address: IpPrefix, status: RoutingInformationStatus, path_attributes: Vec<PathAttribute>) -> Self {
        Self {nexthop, destnation_address, status, path_attributes}
    }

    pub fn get_as_path(&self) -> &AsPath {
        for path in &self.path_attributes {
            match &path {
                &PathAttribute::AsPath(as_path) => return as_path,
                _ => (),
            }
        }
        panic!();
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
