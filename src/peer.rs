use std::net::TcpListener;
use std::net::SocketAddr;
use crate::{Config, finite_state_machine::fsm};
use crate::rib::{LocRib, AdjRibIn, AdjRibOut};
pub struct BgpPeers {
    pub peers: Vec<fsm>,
    pub loc_rib: LocRib,
}

impl BgpPeers {
    pub fn new(configs: Vec<Config>) -> Self {
        let mut peers = vec![];
        for config in configs {
            let tcp_listener = TcpListener::bind((config.my_ip_addr, 179)).expect("port 179が使用できません。");
            let fsm = fsm::new(config, tcp_listener);
            peers.push(fsm);
        }
        let loc_rib = LocRib::new(vec![]);
        Self { peers, loc_rib }
    }
}
