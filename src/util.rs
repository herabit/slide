#![allow(dead_code)]

use core::{
    mem::{self},
    ptr::{self, NonNull},
};

use crate::macros::assert_unchecked;

/// Helper function that marks something as needing to be unsafe.
#[inline(always)]
pub(crate) const unsafe fn needs_unsafe<T>(x: T) -> T {
    x
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

/// Perform an exact division without checks, ensuring:
///
/// - That `rhs` is nonzero (`rhs != 0`).
///
/// - That `lhs` is a multiple of `rhs` (`lhs % rhs == 0`).
///
/// - This has the added bonus of proving to the compiler
///   that `(lhs / rhs) * rhs == lhs` is always true.
///
/// # Safety
///
/// - It is undefined behavior for `rhs` to be zero (`rhs == 0`).
///
/// - It is undefined behavior for `lhs` to not be a multiple of `rhs`
///   (`lhs % rhs != 0`).
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn exact_div_unchecked(lhs: usize, rhs: usize) -> usize {
    unsafe {
        // SAFETY: The caller ensures that `rhs != 0`.
        assert_unchecked!(rhs != 0, "division by zero (`rhs == 0`)");

        // SAFETY: The caller ensures that `lhs` is a multiple of `rhs`.
        assert_unchecked!(
            lhs % rhs == 0,
            "`lhs` is not a multiple of `rhs` (`lhs % rhs != 0`)"
        );

        // SAFETY: Since the caller ensures `lhs` is a multiple of `rhs`,
        //         it can be assumed `(lhs / rhs) * rhs` does not overflow,
        //         and that it is equal to `lhs`.
        //
        //         This mainly serves as a workaround that LLVM sometimes will
        //         emit branching code for computing divisions, which seems to
        //         fuck up LLVM identifying what would be an otherwise obvious
        //         identity.
        //
        //         On x86_64 it seems to do the branch to allow the computer to
        //         perform 32-bit divisions instead of 64-bit ones... I'd be
        //         interested to learn the rational behind this optimization.
        //
        //         Or, perhaps the branching is entirely unrelated, in fact it likely
        //         is, and the branch is just the thing I got focused on.
        //
        //         But yeah, this is just to further give the compiler hints about
        //         the invariants of this function.
        assert_unchecked!(
            (lhs / rhs).unchecked_mul(rhs) == lhs,
            "`(lhs / rhs) * rhs != lhs`, something has gone horribly wrong"
        );
    }

    lhs / rhs
}

/// Tries to perform an exact division, ensuring:
///
/// - That `rhs` is nonzero (`rhs != 0`).
///
/// - That `lhs` is a multiple of `rhs` (`lhs % rhs == 0`). This has
///   the added bonus of proving to the compiler that `(lhs / rhs) * rhs == lhs`
///   is always true.
///
/// # Returns
///
/// - Returns `None` if `rhs` is zero (`rhs == 0`).
/// - Returns `None` if `lhs` is not a multiple of `rhs` (`lhs % rhs != 0`).
#[inline(always)]
#[must_use]
pub(crate) const fn exact_div_checked(lhs: usize, rhs: usize) -> Option<usize> {
    if rhs != 0 && lhs % rhs == 0 {
        // SAFETY: We checked that `rhs` is not zero and that `lhs`
        //         is a multiple of `rhs`.
        Some(unsafe { exact_div_unchecked(lhs, rhs) })
    } else {
        None
    }
}

/// Perform an exact division, ensuring:
///
/// - That `rhs` is nonzero (`rhs != 0`).
///
/// - That `lhs` is a multiple of `rhs` (`lhs % rhs == 0`). This has
///   the added bonus of proving to the compiler that `(lhs / rhs) * rhs == lhs`
///   is always true.
///
/// # Panics
///
/// - Panics if `rhs` is zero (`rhs == 0`).
/// - Panics if `lhs` is not a multiple of `rhs` (`lhs % rhs != 0`).
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn exact_div(lhs: usize, rhs: usize) -> usize {
    assert!(rhs != 0, "division by zero (`rhs == 0`)");
    assert!(
        lhs % rhs == 0,
        "`lhs` is not a multiple of `rhs` (`lhs % rhs != 0`)"
    );

    // SAFETY: We checked that `rhs` is not zero and that `lhs`
    //         is a multiple of `rhs`.
    unsafe { exact_div_unchecked(lhs, rhs) }
}

/// Returns a [`usize`] whose every byte is equal to `x`.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn repeat_u8(x: u8) -> usize {
    usize::from_ne_bytes([x; size_of::<usize>()])
}

/// Return a [`usize`] whose every byte pair is equal to `x`.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn repeat_u16(x: u16) -> usize {
    const LEN: usize = size_of::<usize>() / size_of::<u16>();

    // SAFETY: Integers and arrays of integers are plain-old-datatypes,
    //         so transmuting between them is always safe.
    unsafe { mem::transmute([x; LEN]) }
}

/// Sum all bytes within a given [`usize`].
///
/// I kinda figured out how the algorithm works, but
/// if you were to ask me in the future it's a thorough
/// "I stole this from the rust standard library, and forget
/// why it works".
///
/// SWAR is confusing.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn sum_bytes_usize(values: usize) -> usize {
    const LSB_SHORTS: usize = repeat_u16(0x0001);
    const SKIP_BYTES: usize = repeat_u16(0x00FF);

    const TRUNC_SHIFT: usize = (size_of::<usize>() - 2) * 8;
    const MAX_VALUE: usize = size_of::<usize>().checked_mul(0xFF).unwrap();

    let a = values & SKIP_BYTES;
    let b = (values >> 8) & SKIP_BYTES;

    // SAFETY: `a` and `b` are both vectors of `u16`s that
    //         are all representable as a byte.
    //
    //         As such, the lanewise additions will never overflow
    //         into other lanes, and thus this operation itself will
    //         never overflow.
    //
    //         This is likely unecessary, but it may help the compiler.
    let pair_sum = unsafe { a.unchecked_add(b) };

    // This multiplication will effectively sum all shorts in `pair_sum`
    // into the upper most short lane.
    //
    // As such, we will need to truncate it next.
    let result = pair_sum.wrapping_mul(LSB_SHORTS);

    // Truncate the result by shifting until to the last lane.
    let result = result >> TRUNC_SHIFT;

    // SAFETY: We know that `result` will be at most
    //         `255 * size_of::<usize>()`.
    //
    //         This is likely unecessary, but it may help the compiler.
    unsafe { assert_unchecked!(result <= MAX_VALUE) };

    result
}

#[inline(always)]
#[cold]
pub(crate) const fn cold() {}

#[inline(always)]
#[must_use]
pub(crate) const fn unlikely(cond: bool) -> bool {
    if cond {
        cold();
    }

    cond
}

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
pub(crate) const unsafe fn post_inc<T>(ptr: &mut NonNull<T>, n: usize) -> NonNull<T> {
    let old = *ptr;

    *ptr = unsafe { ptr.add(n) };

    old
}

#[inline(always)]
#[must_use]
pub(crate) const unsafe fn pre_dec<T>(ptr: &mut NonNull<T>, n: usize) -> NonNull<T> {
    *ptr = unsafe { ptr.sub(n) };

    *ptr
}
