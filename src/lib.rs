#![cfg_attr(not(test), no_std)]

mod helpers;
mod macros;
mod pos;
mod raw;

/// Helper function that marks something as needing to be unsafe.
#[inline(always)]
pub(crate) const unsafe fn needs_unsafe<T>(x: T) -> T {
    x
}
