use mrbgpd::finite_state_machine::{fsm, Event};
use std::net::TcpListener;

fn main() {
    let tcp_listener = TcpListener::bind("0.0.0.0:179").expect("port 179が使用できません。");
    let mut fsm = fsm::new();
    fsm.event_queue.push(Event::ManualStart);
    loop {
        println!("{:?}", fsm.get_state());
        match fsm.event_queue.pop() {
            Some(event) => fsm.handle_event(&event),
            None => (),
        }
    }
}
