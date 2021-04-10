use rtnetlink::packet::RouteMessage;

#[derive(Clone)]
pub struct Rib(pub Vec<RouteMessage>);

impl Rib {
    pub fn new(routing_table: Vec<RouteMessage>) -> LocRib {
        Rib(routing_table)
    }

    pub fn add(&mut self, routing_information: &mut Vec<RouteMessage>) {
        self.0.append(routing_information);
    }
}

pub type LocRib = Rib;
pub type AdjRibOut = Rib;
