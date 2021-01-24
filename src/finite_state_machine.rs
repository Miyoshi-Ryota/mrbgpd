use std::{time::{Duration, SystemTime}};
use std::net;

use net::{TcpListener, TcpStream};

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
    tcp_connection: Option<net::TcpStream>,
    packet_buffer: [u8; 1024],
    pub event_queue: EventQueue,
}

pub struct EventQueue {
    data: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            data: vec![]
        }
    }

    pub fn push(&mut self, event: Event) {
        self.data.push(event);
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.data.pop()
    }
}

impl fsm {
    pub fn new() -> Self {
        let session_attribute = SessionAttribute::new();
        let tcp_listener = None;
        let tcp_connection = None;
        let event_queue = EventQueue::new();
        let packet_buffer = [0u8; 1024];
        Self { 
            session_attribute,
            tcp_listener,
            tcp_connection,
            packet_buffer,
            event_queue,
        }
    }

    pub fn get_state(&self) -> &State {
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
                        self.tcp_listener = None;
                        self.tcp_connection = net::TcpStream::connect("192.168.2.14:179").ok();
                        self.session_attribute.state = State::Connect;
                    },
                    _ => (),
                };
            },
            &State::Connect => {
                match event {
                    &Event::ManualStop => {
                        // - drops the TCP connection,
                        self.tcp_connection.as_ref().unwrap().shutdown(std::net::Shutdown::Both).unwrap();
                        self.tcp_connection = None;
                        // - releases all BGP resources,
                        self.packet_buffer = [0u8; 1024];
                        // - sets ConnectRetryCounter to zero,
                        self.session_attribute.connect_retry_counter = 0;
                        // - stops the ConnectRetryTimer and sets ConnectRetryTimer to
                        //   zero, and
                        self.session_attribute.connect_retry_timer = Duration::from_secs(0);
                        // - changes its state to Idle.
                        self.session_attribute.state = State::Idle;
                    },
                    &Event::ConnectRetryTimerExpires => {
                        // - drops the TCP connection,
                        // - restarts the ConnectRetryTimer,
                        // - stops the DelayOpenTimer and resets the timer to zero,
                        // - initiates a TCP connection to the other BGP peer,
                        // - continues to listen for a connection that may be initiated by
                        //   the remote BGP peer, and
                        // - stays in the Connect state.
                    },
                    &Event::TcpCrAcked | &Event::TcpConnectionConfirmed => {
                        // If the TCP connection succeeds (Event 16 or Event 17), the local
                        // system checks the DelayOpen attribute prior to processing.  If the
                        // DelayOpen attribute is set to TRUE, the local system:
                        // - DelayOpenAttributeは実装しておらず常にFALSEなので省略
 
                        // If the DelayOpen attribute is set to FALSE, the local system:
                        // - stops the ConnectRetryTimer (if running) and sets the
                        //   ConnectRetryTimer to zero,
                        // - completes BGP initialization
                        // - sends an OPEN message to its peer,
                        // - sets the HoldTimer to a large value, and
                        // - changes its state to OpenSent. 
                        // A HoldTimer value of 4 minutes is suggested.
                    },
                    &Event::TcpConnectionFails => {
                        // If the TCP connection fails (Event 18), the local system checks
                        // the DelayOpenTimer.  If the DelayOpenTimer is running, the local
                        // If the DelayOpenTimer is not running, the local system:
                        // - stops the ConnectRetryTimer to zero,
                        // - drops the TCP connection,
                        // - releases all BGP resources, and
                        // - changes its state to Idle.
                    },
                    &Event::BgpHeaderErr | &Event::BgpOpenMsgErr => {
                        // If BGP message header checking (Event 21) or OPEN message checking 
                        // detects an error (Event 22) (see Section 6.2), the local system:
                        // - (optionally) If the SendNOTIFICATIONwithoutOPEN attribute is
                        //   set to TRUE, then the local system first sends a NOTIFICATION
                        //   message with the appropriate error code, and then
                        // - stops the ConnectRetryTimer (if running) and sets the
                        //   ConnectRetryTimer to zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - increments the ConnectRetryCounter by 1,
                        // - (optionally) performs peer oscillation damping if the
                        //   DampPeerOscillations attribute is set to TRUE, and
                        // - changes its state to Idle.
                    },
                    &Event::NotifMsgVerErr => {
                        // If a NOTIFICATION message is received with a version error (Event
                        // 24), the local system checks the DelayOpenTimer.  If the
                        // DelayOpenTimer is running, the local system:
                        // If the DelayOpenTimer is not running, the local system:
                        // - stops the ConnectRetryTimer and sets the ConnectRetryTimer to
                        //   zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - increments the ConnectRetryCounter by 1,
                        // - performs peer oscillation damping if the DampPeerOscillations
                        //   attribute is set to True, and
                        // - changes its state to Idle.
                    },
                    _ => {
                        // If the DelayOpenTimer is not running, the local system:
                        // - stops the ConnectRetryTimer and sets the ConnectRetryTimer to
                        //   zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - increments the ConnectRetryCounter by 1,
                        // - performs peer oscillation damping if the DampPeerOscillations
                        //   attribute is set to True, and
                        // - changes its state to Idle.
                    }
                }
            },
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

#[derive(Debug)]
pub enum State {
    Idle,
    Connect,
    Active,
    OpenConfirm,
    Established,
}