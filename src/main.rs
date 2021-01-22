use mrbgpd::finite_state_machine::{fsm, Event};

fn get_event() -> Event {
    // Todo: Eventをいい感じに返すように
    Event::ManualStart
}

fn main() {
    let mut fsm = fsm::new();
    loop {
        println!("{:?}", fsm.get_state());
        let event = get_event();
        fsm.handle_event(&event);
    }
}
