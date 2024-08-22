// TODO(kijv) update this to newest impl of turbopack-node (or try to make
// NodeJsPool public)
#![feature(async_closure)]
#![feature(min_specialization)]
#![feature(arbitrary_self_types)]
#![feature(extract_if)]

pub mod transforms;
pub mod embed_js;

pub fn register() {
    turbopack_binding::turbo::tasks::register();
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbo::tasks_bytes::register();
    turbopack_binding::turbopack::ecmascript::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
