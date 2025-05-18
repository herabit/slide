#![allow(dead_code)]
use core::{
    ops::{Bound, Range, RangeInclusive},
    ptr,
};

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

#[inline(always)]
#[must_use]
#[track_caller]
pub const fn bound_ref<T>(bound: &Bound<T>) -> Bound<&T> {
    match bound {
        Bound::Included(bound) => Bound::Included(bound),
        Bound::Excluded(bound) => Bound::Excluded(bound),
        Bound::Unbounded => Bound::Unbounded,
    }
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const fn bound_copied<T: Copy>(bound: Bound<&T>) -> Bound<T> {
    match bound {
        Bound::Included(&bound) => Bound::Included(bound),
        Bound::Excluded(&bound) => Bound::Excluded(bound),
        Bound::Unbounded => Bound::Unbounded,
    }
}
