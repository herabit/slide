#![allow(dead_code)]

use crate::macros::assert_unchecked;
use core::mem::ManuallyDrop;

/// An even unsafer version of [`mem::transmute`].
///
/// # Safety
///
/// In addition to the invariants of [`mem::transmute`],
/// the caller must ensure that `A` and `B` are the same size.
///
/// Failure to do so is immediate undefined behavior.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn transmute_unchecked<A, B>(a: A) -> B {
    #[repr(C)]
    union Transmute<A, B> {
        a: ManuallyDrop<A>,
        b: ManuallyDrop<B>,
    }

    unsafe {
        // SAFETY: The caller ensures that `A` and `B` are the same size.
        assert_unchecked!(size_of::<A>() == size_of::<B>(), "size mismatch");

        // SAFETY: The caller ensures that the transmute is safe.
        ManuallyDrop::into_inner(
            Transmute::<A, B> {
                a: ManuallyDrop::new(a),
            }
            .b,
        )
    }
}

/// A version of [`mem::transmute`] that plays better with generics.
///
/// # Safety
///
/// See [`mem::transmute`].
///
/// # Panics
///
/// Panics if `A` and `B` are not the same size.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn transmute<A, B>(a: A) -> B {
    assert!(
        size_of::<A>() == size_of::<B>(),
        "transmutes must be between types of equal size"
    );

    // SAFETY: The caller ensures the transmute is sound, and we know that `A` and `B`
    //         are of equal size.
    unsafe { transmute_unchecked(a) }
}
