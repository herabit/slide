#![cfg_attr(not(test), no_std)]
#![allow(clippy::needless_lifetimes, clippy::wrong_self_convention)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Macros that are used internally.
mod macros;

/// Utilities that are used internally for handling memory.
mod mem;

/// Utilities that are used internally that don't really belong
/// anywhere else.
mod util;

/// Marker types and traits that are used internally.
mod marker;

/// Just some helpers for [`prim@str`]s.
pub mod str;

/// What is a slice? This module seeks to answer that question.
pub mod slice;

/// Module for handling slice bounds.
pub mod bounds;
