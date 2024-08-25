#![feature(backtrace_frames)]
#![feature(future_join)]
#![feature(min_specialization)]
#![feature(arbitrary_self_types)]
#![feature(panic_payload_as_str)]
#![feature(panic_can_unwind)]
#![feature(pattern)]

pub mod arguments;
pub mod build;
pub(crate) mod contexts;
pub mod dev;
pub(crate) mod embed_js;
pub mod panic_handler;
pub mod tracing_presets;
pub mod upgrade;
pub(crate) mod util;

pub fn register() {
    turbopack_binding::turbopack::turbopack::register();
    fujinoki_ecmascript_plugins::register();
    fujinoki_core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
