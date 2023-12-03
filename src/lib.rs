#![allow(clippy::multiple_crate_versions)]
// socket2 from hyper and tokio

pub mod core;
mod serde;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;
