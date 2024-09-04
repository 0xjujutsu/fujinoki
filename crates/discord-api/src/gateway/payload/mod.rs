use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use turbopack_binding::turbo::{tasks as turbo_tasks, tasks::RcStr};

pub mod receive;
pub mod send;

use self::{receive::*, send::*};
use super::opcode::OpCode;

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug)]
pub struct Payload {
    pub op: OpCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<PayloadData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<RcStr>,
}

pub trait PayloadToString {
    fn to_string(&self) -> String;
}

impl PayloadToString for Payload {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum PayloadData {
    Json(JsonValue),
    Receive(ReceiveEvents),
    Send(SendEvents),
}

impl TryFrom<PayloadData> for JsonValue {
    type Error = ();

    fn try_from(value: PayloadData) -> Result<Self, Self::Error> {
        match value {
            PayloadData::Json(value) => Ok(value),
            _ => Err(()),
        }
    }
}

macro_rules! impl_from {
    ($enum_type:ident, $($variant:ident, $payload:ident),+) => {
        $(
            impl From<$payload> for $enum_type {
                fn from(value: $payload) -> Self {
                    $enum_type::$variant(value)
                }
            }
        )+
    }
}

impl_from!(
    PayloadData,
    Json, JsonValue,
    Receive, ReceiveEvents,
    Send, SendEvents
);

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReceiveEvents {
    Hello(HelloPayloadData),
    Ready(ReadyPayloadData),
}

impl TryFrom<PayloadData> for ReceiveEvents {
    type Error = ();

    fn try_from(value: PayloadData) -> Result<Self, Self::Error> {
        match value {
            PayloadData::Receive(event) => Ok(event),
            _ => Err(()),
        }
    }
}

impl_from!(
    ReceiveEvents,
    Hello, HelloPayloadData,
    Ready, ReadyPayloadData
);

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum SendEvents {
    Identify(IdentifyPayloadData),
    UpdatePresence(UpdatePresencePayloadData),
}

impl TryFrom<PayloadData> for SendEvents {
    type Error = ();

    fn try_from(value: PayloadData) -> Result<Self, Self::Error> {
        match value {
            PayloadData::Send(event) => Ok(event),
            _ => Err(()),
        }
    }
}

impl_from!(
    SendEvents,
    Identify, IdentifyPayloadData,
    UpdatePresence, UpdatePresencePayloadData
);
