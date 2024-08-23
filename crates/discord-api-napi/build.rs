use turbopack_binding::turbo::tasks_build::generate_register;

pub fn main() {
    napi_build::setup();
    generate_register()
}
