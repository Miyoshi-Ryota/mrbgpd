use std::{convert::TryInto, fmt, io::Read, net::{Ipv4Addr, TcpStream}};

enum BGPVersion{
    V1,
    V2,
    V3,
    V4,
}

impl BGPVersion {
    fn decode_to_u8(&self) -> u8 {
        match self {
            &BGPVersion::V1 => 1,
            &BGPVersion::V2 => 2,
            &BGPVersion::V3 => 3,
            &BGPVersion::V4 => 4,
        }
    }
}

struct BgpMessageHeader {
    length: u16,
    type_: BgpMessageType,
}

impl BgpMessageHeader {
    fn decode_to_u8(&self) -> Vec<u8> {
        let mut raw_data = vec![0u8; 19];
        for i in 0..16 {
            raw_data[i] = 255;
        }
        let bytes = self.length.to_be_bytes();
        raw_data[16] = bytes[0];
        raw_data[17] = bytes[1];
        raw_data[18] = match self.type_ {
            BgpMessageType::Open => 1,
            BgpMessageType::Update => 2,
            BgpMessageType::Notification => 3,
            BgpMessageType::Keepalive => 4,
        };
        raw_data
    }
}

#[derive(Debug)]
enum BgpMessageType {
    Open,
    Update,
    Notification,
    Keepalive,
}

pub struct BgpOpenMessage {
    header: BgpMessageHeader,
    version: BGPVersion,
    my_autonomous_system: AutonomousSystemNumber,
    hold_time: HoldTime,
    bgp_identifier: Ipv4Addr,
    optional_parameter_length: u8, // すでにパース後であるこのデータストラクチャには不要かも
    optional_parameters: Vec<OptionalParameter>,
}

impl BgpOpenMessage {
    pub fn new(my_as_number: AutonomousSystemNumber,
               my_ip_address: Ipv4Addr) -> Self {
        let header = BgpMessageHeader {
            length: 29,
            type_: BgpMessageType::Open,
        };
        let version = BGPVersion::V4;
        let my_autonomous_system = my_as_number;
        let hold_time = HoldTime(60 * 4);
        let bgp_identifier = my_ip_address;
        let optional_parameter_length = 0;
        let optional_parameters = vec![];

        BgpOpenMessage {
            header,
            version,
            my_autonomous_system,
            hold_time,
            bgp_identifier,
            optional_parameter_length,
            optional_parameters,
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        let mut header_bytes = self.header.decode_to_u8();
        let mut buf = [0u8; 10];
        buf[0] = self.version.decode_to_u8();
        let as_bytes = self.my_autonomous_system.0.to_be_bytes();
        buf[1] = as_bytes[0];
        buf[2] = as_bytes[1];

        let hold_time_bytes = self.hold_time.0.to_be_bytes();
        buf[3] = hold_time_bytes[0];
        buf[4] = hold_time_bytes[1];

        let ip_bytes = self.bgp_identifier.octets();
        buf[5] = ip_bytes[0];
        buf[6] = ip_bytes[1];
        buf[7] = ip_bytes[2];
        buf[8] = ip_bytes[3];

        buf[9] = self.optional_parameter_length;

        // ToDo: Optional Parameters ni taiou
        header_bytes.append(&mut buf.to_vec());
        header_bytes
    }
}

struct BgpUpdateMessage {
    withdrawn_routes_length: u16,
    withdrawn_routes: Vec<IpPrefix>,
    total_path_attribute_length: u16,
    path_attributes: Vec<PathAttribute>,
}

struct IpPrefix;

struct PathAttribute;

struct BgpKeepaliveMessage {
    header: BgpMessageHeader,
}

struct BgpNotificationMessage{
    header: BgpMessageHeader,
    error_code: BgpErrorCode,
    data: Vec<u8>, // とりあえず
}

enum BgpErrorCode {
    MessageHeaderError(MessageHeaderErrorSubcode),
    OpenMessageError(OpenMessageErrorSubCode),
    UpdateMessageError(UpdateMessageErrorSubcode),
    HoldTimerExpired,
    FaniteStateMachineError,
    Cease,
}

enum MessageHeaderErrorSubcode {
    ConnectionNotSynchronized,
    BadMessageLength,
    BadMessageType,
}

enum OpenMessageErrorSubCode {
    UnsupportedVersionNumber,
    BadPeerAs,
    BadBgpIdentifier,
    UnsupportedOptionalParameter,
    UnacceptableHoldTime,
}

enum UpdateMessageErrorSubcode {
    MalformedAttributeList,
    UnrecognizedWellKnownAttribute,
    MissingWellKnownAttribute,
    AttributeFlagsError,
    AttributeLengthError,
    InvalidOriginAttribute,
    InvalidNextHopAttribute,
    OptinalAttributeError,
    InvalidNetworkField,
    MalformedAsPath,
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
#[derive(Debug, Copy, Clone)]
pub struct AutonomousSystemNumber(u16);

impl AutonomousSystemNumber {
    pub fn new(as_number: u16) -> Self {
        AutonomousSystemNumber(as_number)
    }
}

enum BgpMessage {
    Open(BgpOpenMessage),
    Update(BgpUpdateMessage),
    Notification(BgpNotificationMessage),
    Keepalive(BgpKeepaliveMessage),
}

pub fn bgp_packet_handler(raw_data: &Vec<u8>) {
    let bgp_message_type = identify_what_kind_of_bgp_packet_is(raw_data);
    match bgp_message_type {
        Ok(t) => {
            match t {
                BgpMessageType::Open => {
                    println!("Open Message!");
                    println!("Raw Data: {:?}", raw_data);
                },
                BgpMessageType::Update => (),
                BgpMessageType::Notification => (),
                BgpMessageType::Keepalive => (),
            }
        },
        Err(_) => (),
    }
}

#[derive(Debug)]
struct CannotIdentifyTheRawDataAsBgpPacketError;
impl fmt::Display for CannotIdentifyTheRawDataAsBgpPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cannot identify the raw data as bgp packet")
    }
}


fn identify_what_kind_of_bgp_packet_is(raw_data: &Vec<u8>) -> Result<BgpMessageType, CannotIdentifyTheRawDataAsBgpPacketError> {
    match raw_data[18] {
        1 => Ok(BgpMessageType::Open),
        2 => Ok(BgpMessageType::Update),
        3 => Ok(BgpMessageType::Notification),
        4 => Ok(BgpMessageType::Keepalive),
        _ => Err(CannotIdentifyTheRawDataAsBgpPacketError),
    }
}
