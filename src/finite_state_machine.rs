use std::{time::{Duration, SystemTime}};
use std::net;

use net::TcpListener;

pub struct SessionAttribute {
    state: State,
    connect_retry_counter: usize,
    connect_retry_timer: Duration,
    connect_retry_time: SystemTime,
    hold_timer: Duration,
    hold_time: SystemTime,
    keepalive_timer: Duration,
    keepalive_time: SystemTime,
}

pub struct fsm {
    session_attribute: SessionAttribute,
    tcp_listener: Option<net::TcpListener>,
    packet_buffer: [u8; 1024],
}

impl fsm {
    pub fn new() -> Self {
        let session_attribute = SessionAttribute::new();
        let tcp_listener = None;
        let packet_buffer = [0u8; 1024];
        Self { 
            session_attribute,
            tcp_listener,
            packet_buffer,
        }
    }

    fn get_state(&self) -> &State {
        self.session_attribute.get_state()
    }

    pub fn handle_event(&mut self, event: &Event) {
        match self.get_state() {
            &State::Idle => {
                match event {
                    Event::ManualStart => {
                        // - initializes all BGP resources for the peer connection,
                        // - sets ConnectRetryCounter to zero,
                        // - starts the ConnectRetryTimer with the initial value,
                        // - initiates a TCP connection to the other BGP peer,
                        // - listens for a connection that may be initiated by the remote
                        //   BGP peer, and
                        // - changes its state to Connect.
                        self.packet_buffer = [0u8; 1024];
                        self.session_attribute.connect_retry_counter = 0;
                        self.session_attribute.connect_retry_timer = std::time::Duration::from_secs(0);
                        self.tcp_listener = Some(TcpListener::bind("0.0.0.0:179").expect("port 179が使用できません。"));
                        self.session_attribute.state = State::Connect;
                    },
                    _ => (),
                };
                // Event を Matchでハンドルさせる。
            },
            &State::Connect => (),
            &State::Active => (),
            &State::OpenConfirm => (),
            &State::Established => (),
        };
    }
}

impl SessionAttribute {
    pub fn new() -> Self {
        SessionAttribute {
            state: State::Idle,
            connect_retry_counter: 0,
            connect_retry_timer: Duration::new(0, 0),
            connect_retry_time: SystemTime::now(),
            hold_timer: Duration::new(0, 0),
            hold_time: SystemTime::now(),
            keepalive_timer: Duration::new(0, 0),
            keepalive_time: SystemTime::now(),
        }
    }

    fn get_state(&self) -> &State {
        &self.state
    }
}

pub enum Event {
    // Administrative Event
    ManualStart, // Event 1
    ManualStop, // Event 2
    // TimerEvent
    ConnectRetryTimerExpires, // Event 9
    HoldTimerExpires, // Event 10
    KeepaliveTimerExpires, // Event 11
    //TcpConnectionBasedEvent
    TcpCrAcked, // Event 16
    TcpConnectionConfirmed, // Event 17
    TcpConnectionFails, // Event 18
    // BgpMessageBasedEvent
    BgpOpen, // Event 19
    BgpHeaderErr, // Event 21
    BgpOpenMsgErr, // Event 22
    NotifMsgVerErr, // Event 24
    NotifMsg, // Event 25
    KeepAliveMsg, // Event 26
    UpdateMsg, // Event 27
    UpdateMsgErr, // Event 28
}

enum State {
    Idle,
    Connect,
    Active,
    OpenConfirm,
    Established,
}