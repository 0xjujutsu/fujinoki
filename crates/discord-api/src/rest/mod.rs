// TODO(kijv) make this into discord-rest
use const_format::formatcp;
use lazy_static::lazy_static;
use turbopack_binding::turbo::tasks::Vc;

use crate::VERSION;

pub mod application;
pub mod gateway;
pub mod http;
pub mod interactions;
pub mod routes;

lazy_static! {
    pub static ref HTTP: Vc<http::Http> = http::Http::new(get_api_url().into());
}

pub const fn get_api_url() -> String {
    formatcp!("https://discord.com/api/v{VERSION}").to_string()
}
