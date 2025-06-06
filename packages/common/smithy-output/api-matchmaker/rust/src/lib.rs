#![allow(clippy::module_inception)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::disallowed_names)]
#![allow(clippy::vec_init_then_push)]
#![allow(rustdoc::bare_urls)]
#![warn(missing_docs)]
//! rivet-matchmaker
//!
//! # Crate Organization
//!
//! The entry point for most customers will be [`Client`]. [`Client`] exposes one method for each API offered
//! by the service.
//!
//! Some APIs require complex or nested arguments. These exist in [`model`](crate::model).
//!
//! Lastly, errors that can be returned by the service are contained within [`error`]. [`Error`] defines a meta
//! error encompassing all possible errors that can be returned by the service.
//!
//! The other modules within this crate are not required for normal usage.

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use error_meta::Error;

#[doc(inline)]
pub use config::Config;

/// Client and fluent builders for calling the service.
pub mod client;
/// Configuration for the service.
pub mod config;
/// Errors that can occur when calling the service.
pub mod error;
mod error_meta;
mod http_serde;
/// Input structures for operations.
pub mod input;
mod json_deser;
mod json_errors;
mod json_ser;
/// Data structures used by operation inputs/outputs.
pub mod model;
/// All operations that this crate can perform.
pub mod operation;
mod operation_deser;
mod operation_ser;
/// Output structures for operations.
pub mod output;
pub use client::{Builder, Client, ClientWrapper};
/// Crate version number.
pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Re-exported types from supporting crates.
pub mod types {
	pub use aws_smithy_http::result::SdkError;
	pub use aws_smithy_types::DateTime;
}
