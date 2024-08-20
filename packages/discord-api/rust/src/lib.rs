#[macro_use]
extern crate napi_derive;

// TODO(kijv) finish implementation
pub mod rest;

#[global_allocator]
static ALLOC: turbopack_binding::turbo::malloc::TurboMalloc =
    turbopack_binding::turbo::malloc::TurboMalloc;

pub fn register() {
    discord_api::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
