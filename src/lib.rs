#![cfg_attr(all(doc, docsrs), feature(doc_auto_cfg))]

mod core;
pub use core::{channel, pay, resolve, withdraw, Resolved, Response};

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use server::Server;
