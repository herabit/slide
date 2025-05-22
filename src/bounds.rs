#![allow(dead_code)]

use crate::{mem::NoDrop, util::bound_copied};
use core::ops::{Bound, RangeBounds};

/// Implementation details.
pub(crate) mod private;

/// Module for the [`SliceRange`] type.
mod slice_range;

#[doc(inline)]
pub use slice_range::{SliceRange, SliceRangeError};

/// A marker trait for types that can be used in `const` to create a [`SliceRange`].
pub unsafe trait SliceBounds: private::Sealed + RangeBounds<usize> {
    #[doc(hidden)]
    type Inner: SliceBounds + ?Sized;

    // A type witness that allows for polymorphism in a manner that works with `const`.
    //
    // Downstream crates must not rely on this existing. It is purely an implementation detail.
    // It is not considered a breaking change for this to be removed at a later date.
    #[doc(hidden)]
    const KIND: private::BoundsKind<Self>;

    /// Borrow the start and end bounds as a tuple.
    #[inline(always)]
    #[must_use]
    fn as_bounds(&self) -> (Bound<&usize>, Bound<&usize>) {
        as_bounds(self)
    }

    /// Get the start and end bounds as a tuple.
    #[inline(always)]
    #[must_use]
    fn to_bounds(&self) -> (Bound<usize>, Bound<usize>) {
        to_bounds(self)
    }

    /// Get the start and end bounds as a tuple, taking ownership of `self`.
    #[inline(always)]
    #[must_use]
    fn into_bounds(self) -> (Bound<usize>, Bound<usize>)
    where
        Self: Sized,
    {
        into_bounds(self)
    }
}

/// Borrow the start and end bounds as a tuple.
#[inline(always)]
#[must_use]
pub const fn as_bounds<B: SliceBounds + ?Sized>(bounds: &B) -> (Bound<&usize>, Bound<&usize>) {
    B::KIND.0.as_bounds(bounds)
}

/// Get the start and end bounds as a tuple.
#[inline(always)]
#[must_use]
pub const fn to_bounds<B: SliceBounds + ?Sized>(bounds: &B) -> (Bound<usize>, Bound<usize>) {
    let (start, end) = as_bounds(bounds);

    (bound_copied(start), bound_copied(end))
}

/// Get the start and end bounds as a tuple, taking ownership of `bounds`.
#[inline(always)]
#[must_use]
pub const fn into_bounds<B: SliceBounds>(bounds: B) -> (Bound<usize>, Bound<usize>) {
    // Luckily, all slicebounds don't have a drop implementation
    let bounds = NoDrop::new(bounds);

    to_bounds(bounds.as_ref())
}

// /// Returns whether the provided `value` is contained within `bounds`.
// #[inline(always)]
// #[must_use]
// pub const fn contains<B: SliceBounds + ?Sized>(bounds: &B, value: usize) -> bool {
//     let (start, end) = as_bounds(bounds);

//     (match start {
//         Bound::Included(&start) => start <= value,
//         Bound::Excluded(&start) => start < value,
//         Bound::Unbounded => true,
//     }) && (match end {
//         Bound::Included(&end) => value <= end,
//         Bound::Excluded(&end) => value < end,
//         Bound::Unbounded => true,
//     })
// }

// /// Returns whether the provided bounds are empty.
// #[inline(always)]
// #[must_use]
// pub const fn is_empty<B: SliceBounds + ?Sized>(bounds: &B) -> bool {
//     match as_bounds(bounds) {
//         // `..end` | `start..` | `..`
//         (Bound::Unbounded, _) | (_, Bound::Unbounded) => false,
//         // `start..end`
//         (Bound::Included(&start), Bound::Excluded(&end)) => start >= end,
//         // `start - 1..end`
//         (Bound::Excluded(&start), Bound::Excluded(&end)) => start >= end,
//         // `start - 1..=end`
//         (Bound::Excluded(&start), Bound::Included(&end)) => start >= end,
//         // `start..=end`.
//         (Bound::Included(&start), Bound::Included(&end)) => start > end,
//     }
// }

// /// Returns the intersection of `a` and `b`.
// #[inline(always)]
// #[must_use]
// pub const fn intersection<A, B>(a: &A, b: &B) -> (Bound<usize>, Bound<usize>)
// where
//     A: SliceBounds + ?Sized,
//     B: SliceBounds + ?Sized,
// {
//     let (a_start, a_end) = to_bounds(a);
//     let (b_start, b_end) = to_bounds(b);

//     let start = match (a_start, b_start) {
//         (Bound::Included(a), Bound::Included(b)) => Bound::Included(if a >= b { a } else { b }),
//         (Bound::Excluded(a), Bound::Excluded(b)) => Bound::Excluded(if a >= b { a } else { b }),
//         (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,

//         (x, Bound::Unbounded) | (Bound::Unbounded, x) => x,

//         (Bound::Included(i), Bound::Excluded(e)) | (Bound::Excluded(e), Bound::Included(i)) => {
//             if i > e {
//                 Bound::Included(i)
//             } else {
//                 Bound::Excluded(e)
//             }
//         }
//     };

//     let end = match (a_end, b_end) {
//         (Bound::Included(a), Bound::Included(b)) => Bound::Included(if a <= b { a } else { b }),
//         (Bound::Excluded(a), Bound::Excluded(b)) => Bound::Excluded(if a <= b { a } else { b }),
//         (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,

//         (x, Bound::Unbounded) | (Bound::Unbounded, x) => x,

//         (Bound::Included(i), Bound::Excluded(e)) | (Bound::Excluded(e), Bound::Included(i)) => {
//             if i < e {
//                 Bound::Included(i)
//             } else {
//                 Bound::Excluded(e)
//             }
//         }
//     };

//     (start, end)
// }
