#![cfg_attr(not(test), no_std)]

/// Macros that are used internally.
mod macros;

/// Utilities that are used internally for handling memory.
mod mem;

/// Utilities that are used internally that don't really belong
/// anywhere else.
mod util;

/// Marker types and traits that are used internally.
mod marker;

/// What is a slice? This module seeks to answer that question.
pub mod slice;

#[doc(inline)]
pub use slice::Slice;
