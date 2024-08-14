use std::time::Duration;

use serde::{Deserialize, Serialize, Serializer};
use ulid::Ulid;

#[derive(Debug, Serialize)]
pub struct PingReport {
    #[serde(serialize_with = "serialize_duration_as_milliseconds")]
    pub duration_ms: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub id: Ulid,
}

#[derive(Debug, Serialize)]
pub struct Process {
    pub id: Ulid,
}

#[derive(Debug, Serialize)]
pub struct ProcessOutput {
    pub process_id: Ulid,
    pub line: String,
}

#[derive(Debug, Serialize)]
/// Enum of message types that the server may send to the client.
pub enum ServerMsg {
    Ping,
    Pong,
    PingReport(PingReport),
    Process(Process),
    ProcessOutput(ProcessOutput),
    Goodbye(String),
}

#[derive(Debug, Deserialize)]
/// Enum of message types that the client may send to the server.
pub enum ClientMsg {
    Command(Command),
    Pong,
    Goodbye(String),
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
