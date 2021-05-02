use mrbgpd::{bgp, finite_state_machine::{fsm, Event}};
use mrbgpd::Config;
use std::{convert::TryInto, io, net::{TcpListener, TcpStream}};
use std::{thread, time};
use std::io::Read;
use std::env;
use mrbgpd::peer::BgpPeers;
use bgp::bgp_packet_handler;
use tokio;


#[tokio::main]
async fn main() {
    let filename: Vec<String> = env::args().collect();
    println!("{:?}", filename[1]);
    let configs = Config::parse_from_file(&filename[1]);
    println!("{:?}", &configs);
    // ToDo: Data BufferをFSMに持たせる
    let mut bgp_peers = BgpPeers::new(configs);
    for fsm in &mut bgp_peers.peers {
        fsm.event_queue.push(Event::ManualStart);
    }
    loop {
        for fsm in &mut bgp_peers.peers {
            println!("{:?}", fsm.get_state());
            match fsm.event_queue.pop() {
                Some(event) => fsm.handle_event(&event).await,
                None => (),
            }
        }
        for fsm in &mut bgp_peers.peers {
            let mut buf = vec![];
            match fsm.tcp_connection.as_ref().unwrap().read_to_end(&mut buf) {
                Ok(_) => {
                 // Tcp connection is closed.
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    fsm.data_buffer.buf.append(&mut buf);
                    // Tcp connection is still open and there no data in socket.
                },
                Err(e) => {
                    println!("other error happen: {:?}, : {:?}", e, buf);
                }
            }
            if fsm.data_buffer.buf.len() > 0 {
                bgp_packet_handler(&fsm.data_buffer.retrive_one_bgp_message(), &mut fsm.event_queue, &mut fsm.packet_queue);
            }
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
