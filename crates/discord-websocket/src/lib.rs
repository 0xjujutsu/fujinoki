#![feature(arbitrary_self_types)]

pub mod commands;
pub mod context;
pub mod dispatch;
mod external;
pub mod heartbeat;
pub mod identity;
mod invalidation;
pub mod issue;
mod util;

pub use dispatch::dispatch;
pub use heartbeat::heartbeat;
pub use identity::identify;

pub fn register() {
    discord_api::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
