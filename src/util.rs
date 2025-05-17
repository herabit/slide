#![allow(dead_code)]
use core::ops::{Bound, Range};

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

/// Convert some bounds into a [`Range<usize>`].
///
/// This does not do any bounds checking (or overflow checking in debug).
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn into_range(
    len: usize,
    (start, end): (Bound<usize>, Bound<usize>),
) -> Range<usize> {
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => start + 1,
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => end + 1,
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    Range { start, end }
}

/// Convert some bounds into a [`Range<usize>`].
///
/// Returns `None` if the indices overflow.
///
/// This does not do any bounds checking.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn into_range_checked(
    len: usize,
    (start, end): (Bound<usize>, Bound<usize>),
) -> Option<Range<usize>> {
    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => match start.checked_add(1) {
            Some(start) => start,
            None => return None,
        },
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => match end.checked_add(1) {
            Some(end) => end,
            None => return None,
        },
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    Some(Range { start, end })
}
