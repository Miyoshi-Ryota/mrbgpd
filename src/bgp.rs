struct PacketBuffer;

enum BGPVersion{
    V1,
    V2,
    V3,
    V4,
}

impl PacketBuffer {
    fn get_version(&self) -> BGPVersion {
        BGPVersion::V4
    }
}
