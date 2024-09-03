#![feature(arbitrary_self_types)]

pub mod commands;
pub mod context;
mod external;
mod invalidation;
pub mod issue;
pub mod send;
pub mod receive;
mod util;

pub fn register() {
    discord_api::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
