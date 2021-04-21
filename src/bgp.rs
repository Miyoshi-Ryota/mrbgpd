use std::{convert::TryInto, fmt, fs::create_dir_all, io::Read, net::{Ipv4Addr, IpAddr, TcpStream}, option, path::Path, str::FromStr};
use crate::rib::{AdjRibOut, Rib};
use crate::Config;
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

pub struct BgpUpdateMessage {
    header: BgpMessageHeader,
    withdrawn_routes_length: u16,
    withdrawn_routes: Vec<IpPrefix>,
    total_path_attribute_length: u16,
    path_attributes: Vec<PathAttribute>,
    network_layer_reachability_information: Vec<IpPrefix>,
}

impl BgpUpdateMessage {
    pub fn is_created_from_adj_rib_out(adj_rib_out: &AdjRibOut, config: &Config) -> Self {
        // ToDo: 実装する
        let advertise_route = adj_rib_out.get_new_route();
        let mut advertise_route_ip_prefixes = vec![];
        for entry in advertise_route {
            if let Some(ip_prefix) = entry.destination_prefix() {
                if let IpAddr::V4(ipaddr) = ip_prefix.0 {
                    let ip_prefix = IpPrefix::new(ipaddr, ip_prefix.1);
                    advertise_route_ip_prefixes.push(ip_prefix);
                }
            }
        }
        let origin = PathAttribute::Origin(Origin::Igp);
        let as_path = PathAttribute::AsPath(AsPath::AsSequence(vec![config.as_number.0]));
        let next_hop = PathAttribute::NextHop(config.my_ip_addr);
        let path_attributes = vec![origin, as_path, next_hop];
        let total_path_attributes_length: usize = path_attributes.iter().map(|p|p.decode().len()).sum();
        let total_path_attributes_length = total_path_attributes_length.try_into().unwrap();

        let withdrawn_routes_length = 0;

        let header_length = 19;
        let mut nlri_length = 0;
        for i in &advertise_route_ip_prefixes {
            nlri_length += i.decode().len();
        }
        let nlri_length: u16 = nlri_length.try_into().unwrap();
        let update_message_length = total_path_attributes_length
            + withdrawn_routes_length
            + 4
            + nlri_length;
        let header = BgpMessageHeader::new(
            header_length + update_message_length,
             BgpMessageType::Update);
        BgpUpdateMessage {
            header,
            withdrawn_routes_length,
            withdrawn_routes: vec![],
            total_path_attribute_length: total_path_attributes_length,
            path_attributes: path_attributes,
            network_layer_reachability_information: advertise_route_ip_prefixes,
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        let mut header_bytes = self.header.decode_to_u8();
        let withdrawn_length = self.withdrawn_routes_length.to_be_bytes();
        let mut withdrawn_routes: Vec<u8> = vec![]; // ToDo: Withdrawn routesに対応しておく。
        let total_path_attribute_length = self.total_path_attribute_length.to_be_bytes();
        let mut path_attributes = vec![];
        for p in &self.path_attributes {
            let mut path_attribute_byte = p.decode().clone();
            path_attributes.append(&mut path_attribute_byte);
        }
        let mut ip_prefix = vec![];
        for i in &self.network_layer_reachability_information {
            let mut ip_prefix_byte = i.decode();
            ip_prefix.append(&mut ip_prefix_byte);
        }
        let mut result = vec![];
        result.append(&mut header_bytes);
        result.append(&mut withdrawn_length.to_vec());
        result.append(&mut withdrawn_routes);
        result.append(&mut total_path_attribute_length.to_vec());
        result.append(&mut path_attributes);
        result.append(&mut ip_prefix);
        println!("{:?}", result);
        result
    }

    pub fn encode(raw_data: &Vec<u8>) -> Self {
        let header = BgpMessageHeader::encode_from_u8(raw_data);
        let withdrawn_routes_length = u16::from_be_bytes(raw_data[19..21].try_into().unwrap());
        let end_of_withdrawn_routes = 21 + withdrawn_routes_length;
        let withdrawn_routes = Self::encode_routes(&raw_data[21..end_of_withdrawn_routes.into()].to_vec());
        let end_of_withdrawn_routes_usize = end_of_withdrawn_routes.try_into().unwrap();
        let total_path_attribute_length = u16::from_be_bytes(
            raw_data[end_of_withdrawn_routes_usize..end_of_withdrawn_routes_usize+2].try_into().unwrap());
        let start_of_path_attributes = end_of_withdrawn_routes_usize + 2;
        let total_path_attribute_length_usize :usize = total_path_attribute_length.into();
        let end_of_path_attributes :usize  = start_of_path_attributes + total_path_attribute_length_usize;
        let path_attributes = Self::encode_path_attributes(&raw_data[start_of_path_attributes..end_of_path_attributes].to_vec());
        let start_of_nlri = end_of_path_attributes;
        let network_layer_reachability_information = Self::encode_routes(&raw_data[start_of_nlri.into()..].to_vec());
        Self {
            header,
            withdrawn_routes_length,
            withdrawn_routes,
            total_path_attribute_length,
            path_attributes,
            network_layer_reachability_information
        }
    }

    fn encode_path_attributes(raw_data: &Vec<u8>) -> Vec<PathAttribute> {
        // path attributeのところだけを渡す
        let mut result = vec![];
        let mut i = 0;
        while i < raw_data.len() {
            let path_attribute_flag = raw_data[i];
            let path_attribute_type = raw_data[i+1];
            let number_of_octates_path_attribute_length = if 0b00010000 & path_attribute_flag == 16 {
                2
            } else {
                1
            };
            let path_attribute_length: u16 = if 0b00010000 & path_attribute_flag == 16 {
                u16::from_be_bytes(raw_data[i+2..i+4].try_into().unwrap())
            } else {
                u16::from_be_bytes([0, raw_data[i+2]])
            };
            let start_of_path_attrtibute_value = i + 2 + number_of_octates_path_attribute_length;
            let path_attribute_length_usize :usize = path_attribute_length.into();
            let end_of_path_attribute_value = start_of_path_attrtibute_value + path_attribute_length_usize;
            let path_attribute_value = &raw_data[start_of_path_attrtibute_value..end_of_path_attribute_value];
            i = end_of_path_attribute_value;

            let path_attribute = PathAttribute::encode(path_attribute_flag, path_attribute_type, path_attribute_length, path_attribute_value);
            result.push(path_attribute);
        }
        result
    }

    fn encode_routes(raw_data: &Vec<u8>) -> Vec<IpPrefix> {
        // withdrawn_routesやnetwork_layer_reachability_informationだけを渡す
        let mut result = vec![];
        let mut i = 0;
        while i < raw_data.len() {
            let prefix_length = raw_data[i];
            // number_of_octatesはprefix_lengthが
            // 0 -> 0
            // 1-8 -> 1
            // 9-16 -> 2
            // 17-24 -> 3
            // 25-32 -> 4
            let number_of_octates = match prefix_length {
                0 => 0,
                1..9 => 1,
                9..17 => 2,
                17..25 => 3,
                25..33 => 4,
                _ => panic!("prefix_length is wrong!"),
            };
            let ipaddr = &raw_data[i+1..i+number_of_octates+1];
            let ip_prefix = IpPrefix::encode(&ipaddr.to_vec());
            result.push(ip_prefix);
            i += 1 + number_of_octates;
        }
        result
    }
}

struct RoutingInformationEntry {
    prefix: IpPrefix,
    destination_address: Ipv4Addr,
    output_interface: Interface
}

struct Interface;


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

enum AsPath {
    AsSet(Vec<u16>),
    AsSequence(Vec<u16>),
}

impl AsPath {
    pub fn value(&self) -> Vec<u8> {
        match &self {
            &AsPath::AsSet(v) => {
                let path_segment_type: u8 = 1;
                let path_segment_length: u8 = v.len().try_into().unwrap();
                let mut result = vec![path_segment_type, path_segment_length];
                for i in v.iter() {
                    let bytes = i.to_be_bytes();
                    for j in bytes.iter() {
                        result.push(*j);
                    }
                }
                result
            },
            &AsPath::AsSequence(v) => {
                let path_segment_type: u8 = 1;
                let path_segment_length: u8 = v.len().try_into().unwrap();
                let mut result = vec![path_segment_type, path_segment_length];
                for i in v.iter() {
                    let bytes = i.to_be_bytes();
                    for j in bytes.iter() {
                        result.push(*j);
                    }
                }
                result
            },
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
    AsPath(AsPath),
    NextHop(Ipv4Addr),
    LocalPref, // EBGPではつかわない
    AtomicAggregate, // 実装は後でで良い
    Aggregator, // 実装は後でで良い
    DontKnow(Vec<u8>), // 不明なやつ
}

impl PathAttribute {
    pub fn decode(&self) -> Vec<u8> {
        match &self {
            &PathAttribute::Origin(origin) => {
                let attribute_flag: u8 = 0b01000000;
                let attribute_type_code = 0b1;
                let path_attribute_length = 1;
                vec![attribute_flag, attribute_type_code, path_attribute_length, origin.value()]
            },
            &PathAttribute::AsPath(as_path) => {
                let attribute_flag: u8 = 0b01000000;
                let attribute_type_code = 2;
                let mut attribute_value = as_path.value();
                let attribute_length: u8 = attribute_value.len().try_into().unwrap();
                let mut result = vec![attribute_flag, attribute_type_code, attribute_length];
                result.append(&mut attribute_value);
                result
            },
            &PathAttribute::NextHop(next_hop) => {
                let attribute_flag: u8 = 0b01000000;
                let attribute_type_code :u8 = 3;
                let attribute_length :u8 = 4;
                let mut attribute_value = next_hop.octets().to_vec();
                let mut result = vec![attribute_flag, attribute_type_code, attribute_length];
                result.append(&mut attribute_value);
                result
            },
            _ => vec![],
        }
    }
    pub fn encode(attribute_flag: u8, attribute_type: u8, attribute_length: u16, attribute_value: Vec<u8>) -> Self {
        match attribute_type {
            1 => {
                let origin = match attribute_type {
                    0 => Origin::Igp,
                    1 => Origin::Egp,
                    2 => Origin::Incompleted,
                    _ => panic!(),
                };
                PathAttribute::Origin(origin)
            },
            2 => {
                let mut as_sequence = vec![];
                let mut i = 0;
                while i < attribute_value.len() {
                    let as_number = u16::from_be_bytes(attribute_value[i..i+2].try_into().unwrap());
                    i += 2;
                    as_sequence.push(as_number);
                };
                PathAttribute::AsPath(as_sequence)
            },
            3 => {
                let ip_addr = Ipv4Addr::new(attribute_value[0], attribute_value[1], attribute_value[2], attribute_value[3]);
                PathAttribute::NextHop(ip_addr)
            },
            _ => PathAttribute::DontKnow(attribute_value)
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
                BgpMessageType::Update => {
                    let bgp_message = BgpUpdateMessage::encode(raw_data);
                    // packet_bufferに積むかも？
                    event_queue.push(Event::UpdateMsg);
                },
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
