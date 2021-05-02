use std::net::TcpListener;
use std::net::SocketAddr;
use crate::{Config, finite_state_machine::fsm};

pub struct BgpPeers {
    pub peers: Vec<fsm>
}

impl BgpPeers {
    pub fn new(configs: Vec<Config>) -> Self {
        let mut peers = vec![];
        for config in configs {
            let tcp_listener = TcpListener::bind((config.my_ip_addr, 179)).expect("port 179が使用できません。");
            let fsm = fsm::new(config, tcp_listener);
            peers.push(fsm);
        }
        Self { peers }
    }
}
