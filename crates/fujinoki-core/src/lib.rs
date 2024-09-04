#![feature(arbitrary_self_types)]
#![feature(hash_extract_if)]

use const_format::formatcp;

pub mod config;
pub mod structures;

// The main reason for doing this is to make sure everything is correctly
// spelled and formatted.
pub const GITHUB_REPO: &'static str = "0xjujutsu/fujinoki";
/// Does not include the protocol
pub const WEBSITE: &'static str = "jujutsu.studio";
/// Or the name of the binary
pub const NPM_PACKAGE: &'static str = "fujinoki";
pub const DISPLAY_NAME: &'static str = "Fujinoki";

pub fn get_version() -> &'static str {
    let package_json = include_str!("../../../packages/fujinoki/package.json");

    if let Some(version) = package_json.split("\"version\": \"").nth(1) {
        if let Some(version) = version.split('\"').next() {
            return version;
        }
    }

    unreachable!()
}

pub const fn platform_name() -> &'static str {
    const ARCH: &str = {
        #[cfg(target_arch = "x86_64")]
        {
            "64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            "arm64"
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            "unknown"
        }
    };

    const OS: &str = {
        #[cfg(target_os = "macos")]
        {
            "darwin"
        }
        #[cfg(target_os = "windows")]
        {
            "windows"
        }
        #[cfg(target_os = "linux")]
        {
            "linux"
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            "unknown"
        }
    };

    formatcp!("{OS}-{ARCH}")
}

pub fn register() {
    fujinoki_node::register();
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbo::tasks_env::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
