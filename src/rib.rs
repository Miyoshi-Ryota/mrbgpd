use rtnetlink::packet::RouteMessage;
use std::str::FromStr;

pub struct LocRib(Vec<RouteMessage>);

impl LocRib {
    pub fn new(routing_table: Vec<RouteMessage>) -> LocRib {
        LocRib(routing_table)
    }

    pub fn add(&mut self, routing_information: &mut Vec<RouteMessage>) {
        self.0.append(routing_information);
    }
}
