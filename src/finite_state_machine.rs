use std::time::{Duration, SystemTime};

struct SessionAttribute {
    state: State,
    connect_retry_counter: usize,
    connect_retry_timer: Duration,
    connect_retry_time: SystemTime,
    hold_timer: Duration,
    hold_time: SystemTime,
    keepalive_timer: Duration,
    keepalive_time: SystemTime,
}

enum Event {
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