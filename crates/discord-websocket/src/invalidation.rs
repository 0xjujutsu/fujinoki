use std::fmt::{Display, Formatter};

use discord_api::gateway::{OpCode, OpCodeName};
use indexmap::IndexSet;
use turbopack_binding::turbo::tasks::{
    util::StaticOrArc, InvalidationReason, InvalidationReasonKind,
};

/// Computation was caused by a event from the gateway.
#[derive(PartialEq, Eq, Hash)]
pub struct WebsocketMessage {
    pub opcode: OpCode,
    pub event: Option<String>,
    pub hide: bool,
}

impl InvalidationReason for WebsocketMessage {
    fn kind(&self) -> Option<StaticOrArc<dyn InvalidationReasonKind>> {
        Some(StaticOrArc::Static(&WEBSOCKET_MESSAGE_KIND))
    }
}

impl Display for WebsocketMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.hide {
            write!(f, "[hide] ").unwrap();
        }

        write!(
            f,
            "{}",
            self.event
                .clone()
                .unwrap_or(self.opcode.name().to_uppercase())
        )
    }
}

/// Invalidation kind for [WebsocketMessage]
#[derive(PartialEq, Eq, Hash)]
struct WebsocketMessageKind;

static WEBSOCKET_MESSAGE_KIND: WebsocketMessageKind = WebsocketMessageKind;

impl InvalidationReasonKind for WebsocketMessageKind {
    fn fmt(
        &self,
        reasons: &IndexSet<StaticOrArc<dyn InvalidationReason>>,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        let example = reasons
            .into_iter()
            .filter(|reason| reason.to_string().chars().count() > 0)
            .map(|reason| reason.as_any().downcast_ref::<WebsocketMessage>().unwrap())
            .min_by_key(|reason| {
                reason
                    .event
                    .clone()
                    .unwrap_or(reason.opcode.name().to_string())
                    .len()
            })
            .unwrap();
        write!(
            f,
            "{} events ({}, ...)",
            reasons.len(),
            example
                .event
                .clone()
                .unwrap_or(example.opcode.name().to_string().to_uppercase())
        )
    }
}

/// Side effect that was caused by a request to the server.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct WebsocketMessageSideEffects {
    pub opcode: OpCode,
    pub event: Option<String>,
}

impl InvalidationReason for WebsocketMessageSideEffects {
    fn kind(&self) -> Option<StaticOrArc<dyn InvalidationReasonKind>> {
        Some(StaticOrArc::Static(&WEBSOCKET_MESSAGE_SIDE_EFFECTS_KIND))
    }
}

impl Display for WebsocketMessageSideEffects {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "side effects of {}",
            self.event
                .clone()
                .unwrap_or(self.opcode.name().to_uppercase())
        )
    }
}

/// Invalidation kind for [WebsocketMessageSideEffects]
#[derive(PartialEq, Eq, Hash)]
struct WebsocketMessageSideEffectsKind;

static WEBSOCKET_MESSAGE_SIDE_EFFECTS_KIND: WebsocketMessageSideEffectsKind =
    WebsocketMessageSideEffectsKind;

impl InvalidationReasonKind for WebsocketMessageSideEffectsKind {
    fn fmt(
        &self,
        reasons: &IndexSet<StaticOrArc<dyn InvalidationReason>>,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        let example = reasons
            .into_iter()
            .map(|reason| {
                reason
                    .as_any()
                    .downcast_ref::<WebsocketMessageSideEffects>()
                    .unwrap()
            })
            .min_by_key(|reason| {
                reason
                    .event
                    .clone()
                    .unwrap_or(reason.opcode.name().to_string())
                    .len()
            })
            .unwrap();
        write!(
            f,
            "side effects of {} events ({}, ...)",
            reasons.len(),
            example
                .event
                .clone()
                .unwrap_or(example.opcode.name().to_string().to_uppercase())
        )
    }
}
