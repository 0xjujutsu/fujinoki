#![feature(async_closure)]
#![feature(min_specialization)]
#![feature(round_char_boundary)]
#![feature(thread_id_value)]
#![feature(arbitrary_self_types)]

pub mod issue;
pub mod runtime_entry;
pub mod source_context;

pub fn register() {
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbopack::core::register();
    turbopack_binding::turbopack::resolve::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
