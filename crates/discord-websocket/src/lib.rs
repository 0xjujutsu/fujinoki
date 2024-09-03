#![feature(arbitrary_self_types)]

pub mod commands;
pub mod context;
pub mod dispatch;
mod external;
mod invalidation;
pub mod issue;
pub mod send;
mod util;

pub use dispatch::dispatch;

pub fn register() {
    discord_api::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
