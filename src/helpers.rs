#![allow(dead_code)]

use core::ptr::{self, NonNull};

use crate::macros::assert_unchecked;

/// Returns `lhs.unchecked_sub(rhs)` with some better diagnostics in debug builds.
///
/// # Safety
///
/// - `lhs - rhs` must not overflow (`lhs >= rhs`).
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn unchecked_sub(lhs: usize, rhs: usize) -> usize {
    unsafe { assert_unchecked!(lhs >= rhs, "`lhs < rhs`") };

    unsafe { lhs.unchecked_sub(rhs) }
}

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
/// - Likely others. See [`pointer::offset_from_unsigned`] for any potentially
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

/// Creates a [`NonNull`] slice.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn nonnull_slice<T>(ptr: NonNull<T>, len: usize) -> NonNull<[T]> {
    unsafe { NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len)) }
}

/// Creates a [`NonNull`] from a reference.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn nonnull_ref<'a, T: ?Sized>(x: &'a T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(x as *const T as *mut T) }
}

/// Creates a [`NonNull`] from a mutable reference.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn nonnull_mut<'a, T: ?Sized>(x: &'a mut T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(x) }
}
