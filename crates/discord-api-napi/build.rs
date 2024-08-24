use std::{env, path::PathBuf};

use turbopack_binding::turbo::tasks_build::generate_register;

pub fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap();
    let cargo_pkg_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");

    println!("cargo:rerun-if-changed={}", out_dir.display());
    println!("cargo:rerun-if-changed={}", target_dir.display());
    println!("cargo:rerun-if-changed={}", cargo_pkg_name.clone());
    println!("cargo:rerun-if-changed={}", profile.clone());

    let typedef_path =
        PathBuf::from(target_dir).join(format!("lib{}.typedef", cargo_pkg_name.replace("-", "_")));

    println!(
        "cargo:rustc-env=TYPE_DEF_TMP_PATH={}",
        typedef_path.display()
    );

    napi_build::setup();
    generate_register()
}
