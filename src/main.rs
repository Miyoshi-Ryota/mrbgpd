use mrbgpd::finite_state_machine::{fsm, Event};
use mrbgpd::Config;
use std::{net::{TcpListener, TcpStream}};
use std::{thread, time};
use std::io::Read;
use std::env;

// * mai loop goto ni fsm.tcp_stream kara read suru.

fn handle_packets(buf: Vec<u8>) {
    println!("{:?}", buf);
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::parse_args(args);
    println!("{:?}", &config);
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
            Ok(_) => handle_packets(buf),
            Err(e) => println!("error: {}, no packets...: {:?}", e, buf),
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
