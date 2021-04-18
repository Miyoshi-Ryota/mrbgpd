use std::{convert::TryInto, fmt, fs::create_dir_all, io::Read, net::{Ipv4Addr, TcpStream}, option, str::FromStr};

use crate::finite_state_machine::{Event, EventQueue};
use crate::routing::IpPrefix;

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

    fn encode_from_u8(v: u8) -> Result<Self, CannotEncodeU8AsBGPVersion> {
        match v {
            1 => Ok(BGPVersion::V1),
            2 => Ok(BGPVersion::V2),
            3 => Ok(BGPVersion::V3),
            4 => Ok(BGPVersion::V4),
            _ => Err(CannotEncodeU8AsBGPVersion),
        }
    }
}

#[derive(Debug)]
struct CannotEncodeU8AsBGPVersion;
impl fmt::Display for CannotEncodeU8AsBGPVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cannot encode u8 as bgp version")
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

    fn encode_from_u8(raw_data: &Vec<u8>) -> Self {
        let length = u16::from_be_bytes(raw_data[16..18].try_into().unwrap());
        let type_ = identify_what_kind_of_bgp_packet_is(raw_data).unwrap();
        Self { length, type_ }
    }

    fn new(length: u16, type_: BgpMessageType) -> Self {
        Self { length, type_ }
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
    pub fn encode(raw_data: &Vec<u8>) -> Self {
        let header = BgpMessageHeader::encode_from_u8(&raw_data);
        let version = BGPVersion::encode_from_u8(raw_data[19]).unwrap();
        let my_autonomous_system = AutonomousSystemNumber(
            u16::from_be_bytes(raw_data[20..22].try_into().unwrap()));
        let hold_time = HoldTime(
            u16::from_be_bytes(raw_data[22..24].try_into().unwrap()));
        let bgp_identifier = Ipv4Addr::new(raw_data[24], raw_data[25], raw_data[26], raw_data[27]);
        let optional_parameter_length = raw_data[28];

        // ToDo: optional parameter ni taiou suru
        let optional_parameters = if optional_parameter_length != 0 {
            panic!()
        } else {
            vec![]
        };

        Self {
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

impl BgpUpdateMessage {
    pub fn is_created_from_adj_rib_out() -> Self {
        // ToDo: 実装する
    }
}

struct RoutingInformationEntry {
    prefix: IpPrefix,
    destination_address: Ipv4Addr,
    output_interface: Interface
}

struct LocRib(Vec<RoutingInformationEntry>);
struct AdjRibsOut(Vec<RoutingInformationEntry>);
struct Interface;

fn disseminate_routes() -> AdjRibsOut {
    AdjRibsOut(vec![])
    // Phase 3
}

fn put_route_of_network_config_on_loc_lib(loc_lib: &mut LocRib, network: IpPrefix) -> () {
    // 最初の送信するルートはhttps://www.infraexpert.com/study/bgpz06.htmlによると
    // networkコマンドで送ったりするっぽいな、fsmのコンフィグをいじるようにしよう
    // loc_libはグローバルかな
    let (destination_address, output_interface) = lookup_routing_table(&network);
    let routing_information_entry = RoutingInformationEntry {
        prefix: network,
        destination_address,
        output_interface};
    loc_lib.0.push(routing_information_entry);
}

fn lookup_routing_table(network: &IpPrefix) -> (Ipv4Addr, Interface) {
    (Ipv4Addr::from_str("192.168.2.5").unwrap(), Interface)
}

enum Origin {
    Igp,
    Egp,
    Incompleted,
}

impl Origin {
    pub fn value(&self) -> u8 {
        match self {
            &Origin::Igp => 0b0,
            &Origin::Egp => 0b1,
            &Origin::Incompleted => 0b10,
        }
    }
}

enum PathAttribute {
    // PathAttributeのバイト列の表現は以下の通り
    // (<PathAttribute Type>, <attribute length>, <attribute value>)
    // <PathAttribute Type>: (<attr flags>: u8, <attr type code>: u8)
    //  - attr flags: 110[ifattribute length is one octet then 0 two octets then 1]0000
    //    - 0bit: optional(1) or well-known(0)
    //    - 1bit: transitive(1) or non-transitive(0) // well-knownは絶対transitive
    //    - 2bit: optional transitive(1), complete(0) // well-knownとoptional-non-transitiveは0
    //            (optionだけどとりあえずわからなくても転送させるようなやつは1)
    //    - 3bit: if attribute length is one octet then 0 two octets then 1
    //    - 4-7 bit: 0
    //  - attr type code: type code u8
    // <attribute length>: type内の4bit目に応じてu8 or u16 (1 byte or 2 bytes)でattribute valueのオクテット数を表す
    // <attribute value>: ものによる。
    Origin(Origin),
    AsPath,
    NextHop,
    LocalPref, // EBGPではつかわない
    AtomicAggregate, // 実装は後でで良い
    Aggregator, // 実装は後でで良い
}

impl PathAttribute {
    pub fn decode(&self) -> Vec<u8> {
        match self {
            &PathAttribute::Origin(origin) => {
                let attribute_flag: u8 = 0b01000000;
                let attribute_type_code = 0b1;
                let path_attribute_length = 1;
                vec![attribute_flag, attribute_type_code, path_attribute_length, origin.value()]
            },
            &PathAttribute::AsPath => {
                let path_segment_type = 0;
                let path_segment_length = 0;
                let path_segment_value = 0;
                vec![]
            },
            &PathAttribute::NextHop => vec![],
            _ => vec![],
        }
    }
}

pub struct BgpKeepaliveMessage {
    header: BgpMessageHeader,
}

impl BgpKeepaliveMessage {
    pub fn new() -> Self {
        let header = BgpMessageHeader::new(19, BgpMessageType::Keepalive);
        Self { header }
    }

    pub fn decode_to_u8(&self) -> Vec<u8> {
        self.header.decode_to_u8()
    }
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

pub fn bgp_packet_handler(raw_data: &Vec<u8>, event_queue: &mut EventQueue) {
    let bgp_message_type = identify_what_kind_of_bgp_packet_is(raw_data);
    match bgp_message_type {
        Ok(t) => {
            match t {
                BgpMessageType::Open => {
                    // if valid open message
                    let bgp_message = BgpOpenMessage::encode(raw_data);
                    event_queue.push(Event::BgpOpen);
                    // ToDo: else error open message ni taiou
                    // event_queue.push(Event::BgpOpenMsgErr);
                },
                BgpMessageType::Update => (),
                BgpMessageType::Notification => (),
                BgpMessageType::Keepalive => {
                    event_queue.push(Event::KeepAliveMsg);
                },
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
