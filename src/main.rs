use mrbgpd::finite_state_machine::{SessionAttribute, Event};

fn get_event() -> Event {
    // Todo: Eventをいい感じに返すように
    Event::ManualStart
}

fn main() {
    let mut fsm = SessionAttribute::new();
    loop {
        let event = get_event();
        fsm.handle_event(&event);
    }
}
