mod index;
mod private;

use core::ops::{self, Bound};

#[doc(inline)]
pub use index::*;

/// A trait for types that act as ranges for slices.
pub unsafe trait SliceRange: private::Sealed {
    // A type witness for const polymorphism.
    #[doc(hidden)]
    const KIND: private::RangeKind<Self>;
}

#[inline(always)]
#[must_use]
pub const fn as_bounds<R: ?Sized + SliceRange>(range: &R) -> (Bound<&usize>, Bound<&usize>) {
    R::KIND.0.as_bounds(range)
}

#[inline(always)]
#[must_use]
pub const fn into_bounds<R: SliceRange>(range: R) -> (Bound<usize>, Bound<usize>) {
    R::KIND.0.into_bounds(range)
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const fn into_range<R: SliceRange>(range: R, bounds: ops::RangeTo<usize>) -> IndexRange {
    R::KIND.0.into_range(range, bounds)
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const fn into_range_checked<R: SliceRange>(
    range: R,
    bounds: ops::RangeTo<usize>,
) -> Option<IndexRange> {
    R::KIND.0.into_range_checked(range, bounds)
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn into_range_unchecked<R: SliceRange>(
    range: R,
    bounds: ops::RangeTo<usize>,
) -> IndexRange {
    unsafe { R::KIND.0.into_range_unchecked(range, bounds) }
}
