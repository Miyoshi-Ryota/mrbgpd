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

enum AdministrativeEvent {
    ManualStart,
    ManualStop,
}

enum TimerEvent {
    ConnectRetryTimerExpires,
    HoldTimerExpires,
    KeepaliveTimerExpires,
}

enum TcpConnectionBasedEvent {
    TcpCrAcked,
    TcpConnectionConfirmed,
    TcpConnectionFails,
}

enum BgpMessageBasedEvent {
    BgpOpen,
    BgpHeaderErr,
    BgpOpenMsgErr,
    NotifMsgVerErr,
    NotifMsg,
    KeepAliveMsg,
    UpdateMsg,
    UpdateMsgErr,
}