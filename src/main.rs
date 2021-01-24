use mrbgpd::finite_state_machine::{fsm, Event};

fn main() {
    let mut fsm = fsm::new();
    loop {
        println!("{:?}", fsm.get_state());
        match fsm.event_queue.pop() {
            Some(event) => fsm.handle_event(&event),
            None => (),
        }
    }
}
