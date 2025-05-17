#![allow(dead_code)]

/// Helper function that marks something as needing to be unsafe.
#[inline(always)]
pub(crate) const unsafe fn needs_unsafe<T>(x: T) -> T {
    x
}

/// Marks a given code path as cold.
#[inline(always)]
#[cold]
pub(crate) const fn cold() {}

/// Marks a condition as unlikely.
#[inline(always)]
#[must_use]
pub(crate) const fn unlikely(cond: bool) -> bool {
    if cond {
        cold();
    }

    cond
}

/// Marks a condition as likely.
#[inline(always)]
#[must_use]
pub(crate) const fn likely(cond: bool) -> bool {
    if !cond {
        cold();
    }

    cond
}
