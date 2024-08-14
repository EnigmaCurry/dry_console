use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize, Serializer};
use std::time::Duration;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PingReport {
    #[serde(serialize_with = "serialize_duration_as_milliseconds")]
    pub duration_ms: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: Ulid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    pub id: Ulid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessComplete {
    pub id: Ulid,
    pub code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub id: Ulid,
    pub line: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Enum of message types that the server may send to the client.
pub enum ServerMsg {
    Ping,
    Pong,
    PingReport(PingReport),
    Process(Process),
    ProcessOutput(ProcessOutput),
    ProcessComplete(ProcessComplete),
}

#[derive(Debug, Deserialize, Serialize)]
/// Enum of message types that the client may send to the server.
pub enum ClientMsg {
    Command(Command),
    Pong,
}

pub fn serialize_duration_as_milliseconds<S>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u128(duration.as_millis())
}

/// Exit codes for websocket connections
/// https://www.rfc-editor.org/rfc/rfc6455.html#section-7.4.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u16)]
pub enum CloseCode {
    NormalClosure = 1000,           // 1000: Normal closure
    GoingAway = 1001,               // 1001: Going away
    ProtocolError = 1002,           // 1002: Protocol error
    UnsupportedData = 1003,         // 1003: Received data it cannot accept
    InvalidFramePayloadData = 1007, // 1007: Received data inconsistent with message type
    PolicyViolation = 1008,         // 1008: Received message that violates policy
    MessageTooBig = 1009,           // 1009: Received message too big to process
    MissingExtension = 1010,        // 1010: Expected extension not returned in handshake
    InternalServerError = 1011,     // 1011: Encountered unexpected condition
}
