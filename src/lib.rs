#![cfg_attr(all(doc, docsrs), feature(doc_auto_cfg))]

pub mod core;
mod serde;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;
