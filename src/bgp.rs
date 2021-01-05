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

struct BgpOpenMessage;
struct BgpUpdateMessage;
struct BgpNotificationMessage;
struct BgpKeepaliveMessage;

enum BgpMessage {
    Open(BgpOpenMessage),
    Update(BgpUpdateMessage),
    Notification(BgpNotificationMessage),
    Keepalive(BgpKeepaliveMessage),
}