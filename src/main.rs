use mrbgpd::{bgp, finite_state_machine::{fsm, Event}};
use mrbgpd::Config;
use std::{convert::TryInto, io, net::{TcpListener, TcpStream}};
use std::{thread, time};
use std::io::Read;
use std::env;
use bgp::bgp_packet_handler;

struct DataBuffer {
    pub buf: Vec<u8>,
}

impl DataBuffer {
    pub fn new() -> Self {
        let buf = vec![];
        Self { buf }
    }

    fn retrieve_bgp_header_data(&mut self) -> Vec<u8>{
        let bgp_header_length = 19;
        let (bgp_header, buf) = self.buf.split_at(bgp_header_length);
        let bgp_header = bgp_header.to_vec();
        self.buf = buf.to_vec();
        bgp_header
    }

    pub fn retrive_one_bgp_message(&mut self) -> Vec<u8> {
        let mut bgp_message = self.retrieve_bgp_header_data();
        let bgp_header_length = 19;
        let next_bgp_message_length: u16 = u16::from_be_bytes(bgp_message[16..18].try_into().unwrap());
        let (bgp_data, buf )= self.buf.split_at((next_bgp_message_length - bgp_header_length) as usize);
        let mut bgp_data = bgp_data.to_vec();
        self.buf = buf.to_vec();
        bgp_message.append(&mut bgp_data);
        bgp_message
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::parse_args(args);
    println!("{:?}", &config);
    let mut data_buffer = DataBuffer::new();
    let tcp_listener = TcpListener::bind("0.0.0.0:179").expect("port 179が使用できません。");
    // tcp_listener.set_nonblocking(true).unwrap();
    let mut fsm = fsm::new(config, tcp_listener.try_clone().unwrap());
    fsm.event_queue.push(Event::ManualStart);
    loop {
        println!("{:?}", fsm.get_state());
        match fsm.event_queue.pop() {
            Some(event) => fsm.handle_event(&event),
            None => (),
        }
        let mut buf = vec![];
        match fsm.tcp_connection.as_ref().unwrap().read_to_end(&mut buf) {
             Ok(_) => {
                 // Tcp connection is closed.
             },
             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                 data_buffer.buf.append(&mut buf);
                 // Tcp connection is still open and there no data in socket.
             }    
             Err(e) => {
                 println!("other error happen: {:?}, : {:?}", e, buf);
             }
        }
        if data_buffer.buf.len() > 0 {
            bgp_packet_handler(&data_buffer.retrive_one_bgp_message(), &mut fsm.event_queue);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
