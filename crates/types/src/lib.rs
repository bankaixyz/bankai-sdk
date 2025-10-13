#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Shared proof types (always available, works in no_std)
pub mod proofs;

// API module requires std and serde
#[cfg(any(feature = "default", feature = "api"))]
pub mod api;

// Utils module - needs to work in both std and no_std
pub mod utils;

// block is needed for both api and verifier-types
#[cfg(any(feature = "default", feature = "api", feature = "verifier-types"))]
pub mod block;

// Verifier-specific modules (can work in no_std)
#[cfg(feature = "verifier-types")]
pub mod fetch;

#[cfg(feature = "verifier-types")]
pub mod verify;
