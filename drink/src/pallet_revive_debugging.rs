//! This module provides all the necessary elements for supporting contract debugging directly in
//! the contracts pallet.
//!
//! # Smart-contract developer <-> pallet-contracts interaction flow
//!
//! The interaction between end-user and runtime is as follows:
//! 1. At some points during execution, the pallet invokes some callback through its configuration
//!    parameter `Debug`.
//! 2. In order to forward the callback outside the runtime, `Debug` will call a runtime interface,
//!    that will then forward the call further to the proper runtime extension.
//! 3. The runtime extension can be fully controlled by the end-user. It just has to be registered
//!    in the runtime.
//!
//! So, in brief: pallet-contracts -> runtime interface -> runtime extension
//!              |<-----------runtime side-------------->|<---user side--->|
//!
//! # Passing objects between runtime and runtime extension
//!
//! Unfortunately, runtime interface that lies between runtime, and the end-user accepts only
//! simple argument types, and those that implement some specific traits. This means that usually,
//! complex objects will be passed in their encoded form (`Vec<u8>` obtained with scale encoding).

mod intercepting;
mod runtime;
mod tracing;

pub use runtime::{InterceptingExt, InterceptingExtT, NoopExt, TracingExt, TracingExtT};

/// Main configuration parameter for the contracts pallet debugging. Provides all the necessary
/// trait implementations.
pub enum DrinkDebug {}
