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

impl From<JsonValue> for PayloadData {
    fn from(value: JsonValue) -> Self {
        PayloadData::Json(value)
    }
}

impl From<ReceiveEvents> for PayloadData {
    fn from(value: ReceiveEvents) -> Self {
        PayloadData::Receive(value)
    }
}

impl From<SendEvents> for PayloadData {
    fn from(value: SendEvents) -> Self {
        PayloadData::Send(value)
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReceiveEvents {
    Hello(HelloPayloadData),
    Ready(ReadyPayloadData),
}

impl From<HelloPayloadData> for ReceiveEvents {
    fn from(value: HelloPayloadData) -> Self {
        ReceiveEvents::Hello(value)
    }
}

impl From<ReadyPayloadData> for ReceiveEvents {
    fn from(value: ReadyPayloadData) -> Self {
        ReceiveEvents::Ready(value)
    }
}
#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum SendEvents {
    Identify(IdentifyPayloadData),
}

impl From<IdentifyPayloadData> for SendEvents {
    fn from(value: IdentifyPayloadData) -> Self {
        SendEvents::Identify(value)
    }
}
