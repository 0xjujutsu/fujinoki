fn main() {
    napi_build::setup();
    turbopack_binding::turbo::tasks_build::generate_register();
}
