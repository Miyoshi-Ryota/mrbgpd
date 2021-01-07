use std::net::Ipv4Addr;

enum BGPVersion{
    V1,
    V2,
    V3,
    V4,
}

struct BgpMessageHeader {
    length: u16,
    type_: BgpMessageType,
}

enum BgpMessageType {
    Open,
    Update,
    Notification,
    Keepalive,
}

struct BgpOpenMessage {
    header: BgpMessageHeader,
    version: BGPVersion,
    my_autonomous_system: AutonomousSystemNumber,
    hold_time: HoldTime,
    bgp_identifier: Ipv4Addr,
    optional_parameter_length: u8, // すでにパース後であるこのデータストラクチャには不要かも
    optional_parameters: Vec<OptionalParameter>,
}

struct OptionalParameter {
    type_: BgpOpenMessageOptionalParameterType,
    length: u8, // すでにパース後であるこのデータストラクチャには不要かも
    value: Vec<u8>,
}

enum BgpOpenMessageOptionalParameterType {
    i_dont_know_now, // あとでRFC3392をみておく。
}

struct HoldTime(u16);
struct AutonomousSystemNumber(u16);

struct BgpUpdateMessage;
struct BgpNotificationMessage;
struct BgpKeepaliveMessage;

enum BgpMessage {
    Open(BgpOpenMessage),
    Update(BgpUpdateMessage),
    Notification(BgpNotificationMessage),
    Keepalive(BgpKeepaliveMessage),
}