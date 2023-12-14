#![cfg_attr(all(doc, docsrs), feature(doc_auto_cfg))]

mod core;
pub use core::{auth, channel, pay, resolve, withdraw, CallbackResponse, Entrypoint, Resolved};

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use server::Server;
