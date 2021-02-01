use crate::{Config, Mode, bgp::BgpOpenMessage, bgp::BgpKeepaliveMessage};
use std::{alloc::System, time::{Duration, SystemTime}};
use std::net;
use std::{thread, time};
use net::{TcpListener, TcpStream};
use std::io::Write;
pub struct SessionAttribute {
    state: State,
    connect_retry_counter: usize,
    connect_retry_timer: SystemTime,
    connect_retry_time: Duration,
    hold_timer: SystemTime,
    hold_time: Duration,
    keepalive_timer: SystemTime,
    keepalive_time: Duration,
}

pub struct fsm {
    config: Config,
    session_attribute: SessionAttribute,
    tcp_listener: net::TcpListener,
    pub tcp_connection: Option<net::TcpStream>,
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
    pub fn new(config: Config, tcp_listener: TcpListener) -> Self {
        let session_attribute = SessionAttribute::new();
        let tcp_listener = tcp_listener;
        let tcp_connection = None;
        let event_queue = EventQueue::new();
        let packet_buffer = [0u8; 1024];
        Self { 
            config,
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
                        self.session_attribute.connect_retry_timer = SystemTime::now();
                        self.session_attribute.connect_retry_time = std::time::Duration::from_secs(120);
                        self.tcp_connection = match &self.config.mode {
                            &Mode::Active =>
                                net::TcpStream::connect(format!("{}:{}", &self.config.remote_ip_addr, "179")).ok(),
                            &Mode::Passive =>
                                Some(self.tcp_listener.accept().unwrap().0),
                        };
                        if self.tcp_connection.is_some() {
                            self.event_queue.push(Event::TcpConnectionConfirmed);
                        } else {
                            self.event_queue.push(Event::TcpConnectionFails);
                        }
                        self.tcp_connection.as_ref().unwrap().set_nonblocking(true).unwrap();
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
                        self.session_attribute.connect_retry_timer = SystemTime::now();
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
                        self.session_attribute.connect_retry_timer = SystemTime::now();
                        let open_message = BgpOpenMessage::new(
                            self.config.as_number,
                            self.config.my_ip_addr,
                        );
                        let open_message = open_message.decode();
                        self.tcp_connection.as_ref().unwrap().write(&open_message[..]).expect("cannot send open message");
                        self.session_attribute.hold_time = time::Duration::from_secs(4 * 60);
                        self.session_attribute.hold_timer = SystemTime::now();
                        self.session_attribute.state = State::OpenSent;
                    },
                    &Event::TcpConnectionFails => {
                        // If the TCP connection fails (Event 18), the local system checks
                        // the DelayOpenTimer.  If the DelayOpenTimer is running, the local
                        // If the DelayOpenTimer is not running, the local system:
                        // - stops the ConnectRetryTimer to zero,
                        // - drops the TCP connection,
                        // - releases all BGP resources, and
                        // - changes its state to Idle.
                        thread::sleep(time::Duration::from_secs(1));
                        self.session_attribute.state = State::Idle;
                        self.event_queue.push(Event::ManualStart);
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
            &State::OpenSent => {
                match event {
                    &Event::ManualStop => {
                        // - sends the NOTIFICATION with a Cease,
                        // - sets the ConnectRetryTimer to zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - sets the ConnectRetryCounter to zero, and
                        // - changes its state to Idle.
                    },
                    &Event::HoldTimerExpires => {
                        // - sends a NOTIFICATION message with the error code Hold Timer
                        //   Expired,
                        // - sets the ConnectRetryTimer to zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - increments the ConnectRetryCounter,
                        // - (optionally) performs peer oscillation damping if the
                        //   DampPeerOscillations attribute is set to TRUE, and
                        // - changes its state to Idle.
                    },
                    &Event::TcpCrAcked | &Event::TcpConnectionConfirmed => {
                        // If a TcpConnection_Valid (Event 14), Tcp_CR_Acked (Event 16), or a
                        // TcpConnectionConfirmed event (Event 17) is received, a second TCP
                        // connection may be in progress.  This second TCP connection is
                        // tracked per Connection Collision processing (Section 6.8) until an
                        // OPEN message is received.
                    },
                    &Event::TcpConnectionFails => {
                        // If a TcpConnectionFails event (Event 18) is received, the local
                        // system:
                        // - closes the BGP connection,
                        // - restarts the ConnectRetryTimer,
                        // - continues to listen for a connection that may be initiated by
                        //   the remote BGP peer, and
                        // - changes its state to Active.
                    },
                    &Event::BgpOpen => {
                        // When an OPEN message is received, all fields are checked for
                        // correctness.  If there are no errors in the OPEN message (Event
                        // 19), the local system:
                        //   - resets the DelayOpenTimer to zero,
                        //   - sets the BGP ConnectRetryTimer to zero,
                        //   - sends a KEEPALIVE message, and
                        //   - sets a KeepaliveTimer (via the text below)
                        //   - sets the HoldTimer according to the negotiated value (see
                        //     Section 4.2),
                        //   - changes its state to OpenConfirm.
                        self.session_attribute.connect_retry_timer = SystemTime::now();
                        let keepalive_message = BgpKeepaliveMessage::new();
                        let raw_data = keepalive_message.decode_to_u8();
                        self.tcp_connection.as_ref().unwrap().write(&raw_data[..]).unwrap();

                        self.session_attribute.hold_timer = SystemTime::now();
                        // ToDo: Nego wo RFC doori ni yaru (mijikai hou wo saiyou suru)
                        self.session_attribute.hold_time = time::Duration::from_secs(90);

                        self.session_attribute.keepalive_timer = SystemTime::now();
                        self.session_attribute.keepalive_time = self.session_attribute.hold_time / 3;

                        self.session_attribute.state = State::OpenConfirm;
                        // If the negotiated hold time value is zero, then the HoldTimer and
                        // KeepaliveTimer are not started.  If the value of the Autonomous
                        // System field is the same as the local Autonomous System number,
                        // then the connection is an "internal" connection; otherwise, it is
                        // an "external" connection.  (This will impact UPDATE processing as
                        // described below.)
                        // Collision detection mechanisms (Section 6.8) need to be applied
                        // when a valid BGP OPEN message is received (Event 19 or Event 20).
                        // Please refer to Section 6.8 for the details of the comparison.
                    }
                    &Event::BgpHeaderErr | &Event::BgpOpenMsgErr => {
                        // - sends a NOTIFICATION message with the appropriate error code,
                        // - sets the ConnectRetryTimer to zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection,
                        // - increments the ConnectRetryCounter by 1,
                        // - (optionally) performs peer oscillation damping if the
                        //   DampPeerOscillations attribute is TRUE, and
                        // - changes its state to Idle.
                    },
                    &Event::NotifMsgVerErr => {
                        // If a NOTIFICATION message is received with a version error
                        // (Event24), the local system:
                        // - sets the ConnectRetryTimer to zero,
                        // - releases all BGP resources,
                        // - drops the TCP connection, and
                        // - changes its state to Idle.
                    },
                    &Event::ConnectRetryTimerExpires | &Event::KeepaliveTimerExpires | &Event::NotifMsg | &Event::KeepAliveMsg | &Event::UpdateMsg | &Event::UpdateMsgErr => {
                        // In response to any other event (Events 9, 11-13, 20, 25-28), the
                        // local system:
                        //   - sends the NOTIFICATION with the Error Code Finite State
                        //     Machine Error,
                        //   - sets the ConnectRetryTimer to zero,
                        //   - releases all BGP resources,
                        //   - drops the TCP connection,
                        //   - increments the ConnectRetryCounter by 1,
                        //   - (optionally) performs peer oscillation damping if the DampPeerOscillations attribute is set to TRUE, and
                        //   - changes its state to Idle.
                    },
                    _ => {
                        //
                    }
                }
            },
            &State::OpenConfirm => {
                match event {
                    &Event::KeepAliveMsg => {
                        // If the local system receives a KEEPALIVE message (KeepAliveMsg
                        //    (Event 26)), the local system:
                        //      - restarts the HoldTimer and
                        //      - changes its state to Established.
                        self.session_attribute.hold_timer = SystemTime::now();
                        self.session_attribute.state = State::Established;
                    },
                    _ => ()
                }
            },
            &State::Established => (),
        };
    }
}

impl SessionAttribute {
    pub fn new() -> Self {
        SessionAttribute {
            state: State::Idle,
            connect_retry_counter: 0,
            connect_retry_timer: SystemTime::now(),
            connect_retry_time: Duration::from_secs(120),
            hold_timer: SystemTime::now(),
            hold_time: Duration::from_secs(90),
            keepalive_timer: SystemTime::now(),
            keepalive_time: Duration::from_secs(30),
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
    OpenSent,
    Established,
}
