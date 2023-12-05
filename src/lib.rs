#![cfg_attr(all(doc, docsrs), feature(doc_auto_cfg))]

mod core;
mod serde;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub use core::{channel, pay, resolve, withdraw, Query};

#[cfg(feature = "server")]
pub use server::Server;

#[cfg(feature = "client")]
pub use client::Client;
