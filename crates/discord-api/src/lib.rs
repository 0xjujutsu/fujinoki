#![feature(arbitrary_self_types)]
#![feature(const_trait_impl)]
#![feature(trivial_bounds)]
#![feature(min_specialization)]
#![feature(try_trait_v2)]
#![feature(hash_extract_if)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(result_flattening)]
#![feature(error_generic_member_access)]
#![feature(new_uninit)]
#![feature(type_alias_impl_trait)]
#![feature(never_type)]
#![allow(deprecated)]

pub mod application;
pub mod channel;
pub mod emoji;
pub mod gateway;
pub mod guild;
pub mod id;
pub mod interactions;
pub mod issue;
pub mod locales;
pub mod permissions;
pub mod rest;
pub mod team;
pub mod timestamp;
pub mod user;
mod utils;

pub use rest::routes::Routes;

pub const VERSION: i8 = 10;

pub fn register() {
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbo::tasks_fetch::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
