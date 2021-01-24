use mrbgpd::finite_state_machine::{fsm, Event};
use mrbgpd::Config;
use std::net::TcpListener;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::parse_args(args);
    println!("{:?}", &config);
    let tcp_listener = TcpListener::bind("0.0.0.0:179").expect("port 179が使用できません。");
    let mut fsm = fsm::new(config);
    fsm.event_queue.push(Event::ManualStart);
    loop {
        println!("{:?}", fsm.get_state());
        match fsm.event_queue.pop() {
            Some(event) => fsm.handle_event(&event),
            None => (),
        }
    }
}
