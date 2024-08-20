#![feature(arbitrary_self_types)]
#![feature(iter_intersperse)]

pub mod after_resolve;

pub fn register() {
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
