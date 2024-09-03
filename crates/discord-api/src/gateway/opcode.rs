use std::convert::{TryFrom, TryInto};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use turbopack_binding::turbo::{tasks as turbo_tasks, tasks::TaskInput};

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, TaskInput)]
#[non_exhaustive]
pub enum OpCode {
    /// An event was dispatches.
    Dispatch = 0,
    /// Fired periodically by the client to keep the connection alive.
    Heartbeat = 1,
    /// Starts a new session during the initial handshake.
    Identify = 2,
    /// Update the client's presence.
    PresenceUpdate = 3,
    /// Used to join/leave or move between voice channels.
    VoiceStateUpdate = 4,
    /// Resume a previous session that was disconnected.
    Resume = 6,
    /// You should attempt to reconnect and resume immediately.
    Reconnect = 7,
    /// Request information about offline guild members in a large guild.
    RequestGuildMembers = 8,
    /// The session has been invalidated. You should reconnect and
    /// identify/resume accordingly.
    InvalidSession = 9,
    /// Sent immediately after connecting, contains the heartbeat_interval to
    /// use.
    Hello = 10,
    /// Sent in response to receiving a heartbeat to acknowledge that it has
    /// been received.
    HeartbeatAck = 11,
    /// Unknown opcode.
    Unknown = !0,
}

pub trait OpCodeName {
    fn name(&self) -> &'static str;
}

impl OpCodeName for OpCode {
    fn name(&self) -> &'static str {
        match self {
            OpCode::Dispatch => "Dispatch",
            OpCode::Heartbeat => "Heartbeat",
            OpCode::Identify => "Identify",
            OpCode::PresenceUpdate => "PresenceUpdate",
            OpCode::VoiceStateUpdate => "VoiceStateUpdate",
            OpCode::Resume => "Resume",
            OpCode::Reconnect => "Reconnect",
            OpCode::RequestGuildMembers => "RequestGuildMembers",
            OpCode::InvalidSession => "InvalidSession",
            OpCode::Hello => "Hello",
            OpCode::HeartbeatAck => "HeartbeatAck",
            OpCode::Unknown => "Unknown",
        }
    }
}

impl<'de> Deserialize<'de> for OpCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(OpCode::try_from(value).unwrap())
    }
}

impl<'se> Serialize for OpCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = *self as u8;
        value.serialize(serializer)
    }
}

impl TryInto<u8> for OpCode {
    type Error = ();

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            OpCode::Dispatch => Ok(0),
            OpCode::Hello => Ok(1),
            OpCode::Identify => Ok(2),
            OpCode::PresenceUpdate => Ok(3),
            OpCode::VoiceStateUpdate => Ok(4),
            OpCode::Resume => Ok(6),
            OpCode::Reconnect => Ok(7),
            OpCode::RequestGuildMembers => Ok(8),
            OpCode::InvalidSession => Ok(9),
            OpCode::HeartbeatAck => Ok(11),
            OpCode::Unknown => Ok(!0),
            _ => Ok(!0),
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == OpCode::Hello as u8 => Ok(OpCode::Hello),
            x if x == OpCode::Dispatch as u8 => Ok(OpCode::Dispatch),
            x if x == OpCode::RequestGuildMembers as u8 => Ok(OpCode::RequestGuildMembers),
            x if x == OpCode::Heartbeat as u8 => Ok(OpCode::Heartbeat),
            x if x == OpCode::HeartbeatAck as u8 => Ok(OpCode::HeartbeatAck),
            x if x == OpCode::Identify as u8 => Ok(OpCode::Identify),
            x if x == OpCode::InvalidSession as u8 => Ok(OpCode::InvalidSession),
            x if x == OpCode::Reconnect as u8 => Ok(OpCode::Reconnect),
            x if x == OpCode::Resume as u8 => Ok(OpCode::Resume),
            x if x == OpCode::PresenceUpdate as u8 => Ok(OpCode::PresenceUpdate),
            x if x == OpCode::Unknown as u8 => Ok(OpCode::Unknown),
            x if x == OpCode::VoiceStateUpdate as u8 => Ok(OpCode::VoiceStateUpdate),
            _ => Err(()),
        }
    }
}
