use crate::{macros::assert_unchecked, util};
use core::{ptr::NonNull, slice};

pub(crate) mod pos;
pub(crate) mod raw;

/// Returns the length of a represented by the pointer range `start..end`.
///
/// # Safety
///
/// - `start` and `end` must either:
///    - Point to the same address.
///    - Be derived from the same allocated object.
///
/// - The distance between the pointers, in bytes, must be
///   an exact multiple of the size of `T`.
///
/// - The distance between the pointers must be nonnegative (`start <= end`).
///
/// - Likely others. See [`NonNull::offset_from_unsigned`] for any potentially
///   missed details.
///
/// # Panics
///
/// - Panics if `T` is zero-sized.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn slice_len<T>(start: NonNull<T>, end: NonNull<T>) -> usize {
    assert!(size_of::<T>() != 0, "size must be nonzero");
    unsafe { assert_unchecked!(end.offset_from(start) >= 0, "`start > end`") };

    unsafe { end.offset_from_unsigned(start) }
}

/// Turns a slice into a slice of `N`-element arrays.
///
/// # Safety
///
/// - `slice` must divide exactly into `N`-element arrays (`slice.len() % N == 0`).
///
/// # Panics
///
/// - Panics at compile time if `N` is zero.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn as_chunks_unchecked<const N: usize, T>(slice: &[T]) -> &[[T; N]] {
    const { assert!(N != 0, "chunk size must be nonzero") };

    // SAFETY: The caller ensures that the slice's length is exactly divisible by `N`.
    unsafe {
        assert_unchecked!(
            slice.len() % N == 0,
            "slice must be exactly divisible by the chunk size (`slice.len() % N == 0`)"
        )
    };

    // SAFETY: The caller ensures that the slice's length is exactly divisible by `N`.
    let new_len = unsafe { util::exact_div_unchecked(slice.len(), N) };

    // SAFETY: We're converting a slice of `new_len * N` elements into
    //         a slice of `new_len` many `N`-many chunks.
    unsafe { slice::from_raw_parts(slice.as_ptr().cast(), new_len) }
}

/// Splits the slice into the leading chunks of `N`-element arrays, and a slice of the
/// remaining elements.
///
/// # Panics
///
/// - Panics at compile time if `N` is zero.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn as_chunks<const N: usize, T>(slice: &[T]) -> (&[[T; N]], &[T]) {
    const { assert!(N != 0, "chunk size must be nonzero") };

    let rounded_down_len = (slice.len() / N) * N;

    // SAFETY: The rounded down length is always in bounds for `slice`.
    let (left, right) = unsafe { slice.split_at_unchecked(rounded_down_len) };

    // SAFETY: The left slice is guaranteed to have a length that is a multiple of `N`.
    let left = unsafe { as_chunks_unchecked(left) };

    (left, right)
}

/// [`core::slice::Chunks::next`] in const.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn next_chunk<'a, T>(slice: &mut &'a [T], chunk_size: usize) -> Option<&'a [T]> {
    if slice.is_empty() {
        None
    } else {
        let len = if chunk_size < slice.len() {
            chunk_size
        } else {
            slice.len()
        };

        // SAFETY: `len <= slice.len()`.
        let (chunk, rest) = unsafe { slice.split_at_unchecked(len) };
        *slice = rest;

        Some(chunk)
    }
}

/// [`core::slice::Iter::next`] in const.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn next<'a, T>(slice: &mut &'a [T]) -> Option<&'a T> {
    match slice.split_first() {
        Some((first, rest)) => {
            *slice = rest;

            Some(first)
        }
        None => None,
    }
}
