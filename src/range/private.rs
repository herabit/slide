use core::ops::{self, Bound};

use crate::{
    macros::unreachable_unchecked,
    marker::TypeEq,
    mem::NoDrop,
    range::IndexRange,
    util::{bound_copied, bound_ref},
};

/// Trait to seal what types we're considering to be ranges.
pub trait Sealed {}

macro_rules! ranges {
    (
        $(
            $(#[cfg($cfg:meta)])?
            $(#[$attr:meta])*
            $name:ident($ty:ty)
        ),*

        $(,)?
    ) => {

        /// Type witness for the supported range types.
        #[non_exhaustive]
        pub(crate) enum RangeWit<R: ?Sized> {
            $(
                $(#[cfg($cfg)])?
                $(#[$attr])*
                $name(TypeEq<R, $ty>),
            )*
        }


        $(
            $(#[cfg($cfg)])? impl Sealed for $ty {}
            $(#[cfg($cfg)])? unsafe impl super::SliceRange for $ty {
                const KIND: RangeKind<$ty> = RangeKind(RangeWit::$name(TypeEq::new()));
            }
        )*
    };
}

ranges! {
    IndexRange(IndexRange),
    Bounds((Bound<usize>, Bound<usize>)),
    Range(ops::Range<usize>),
    RangeInclusive(ops::RangeInclusive<usize>),
    RangeTo(ops::RangeTo<usize>),
    RangeToInclusive(ops::RangeToInclusive<usize>),
    RangeFrom(ops::RangeFrom<usize>),
    RangeFull(ops::RangeFull),
}

impl<R: ?Sized> RangeWit<R> {
    /// Convert the given range into the bounds it represents.
    ///
    /// # On Correctness
    ///
    /// It is worth noting that currently, what this returns for `core::ops::RangeInclusive<usize>`
    /// is unspecified behavior if it is exhausted.
    ///
    /// As of writing, there is no mechanism through which we can check whether it has been exhausted,
    /// and therefore given the same result as it's `RangeBounds` implementation.
    ///
    /// This is considered to be a bug, but one we're forced to deal with for the time being.
    ///
    /// Luckily, `RangeInclusive` is restricted in how often it is used in comparison to say,
    /// `Range`, but still, it is worth noting that the results of this function for it
    /// will likely differ in the future.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_bounds(self, range: &R) -> (Bound<&usize>, Bound<&usize>) {
        match self {
            RangeWit::IndexRange(conv) => {
                let range = conv.coerce_ref(range);

                (
                    Bound::Included(range.start_ref()),
                    Bound::Excluded(range.end_ref()),
                )
            }
            RangeWit::Bounds(conv) => {
                let (start, end) = conv.coerce_ref(range);

                (bound_ref(start), bound_ref(end))
            }
            RangeWit::Range(conv) => {
                let range = conv.coerce_ref(range);

                (Bound::Included(&range.start), Bound::Excluded(&range.end))
            }
            RangeWit::RangeInclusive(conv) => {
                let range = conv.coerce_ref(range);

                (Bound::Included(range.start()), Bound::Included(range.end()))
            }
            RangeWit::RangeTo(conv) => {
                let range = conv.coerce_ref(range);

                (Bound::Unbounded, Bound::Excluded(&range.end))
            }
            RangeWit::RangeToInclusive(conv) => {
                let range = conv.coerce_ref(range);

                (Bound::Unbounded, Bound::Included(&range.end))
            }
            RangeWit::RangeFrom(conv) => {
                let range = conv.coerce_ref(range);

                (Bound::Included(&range.start), Bound::Unbounded)
            }
            RangeWit::RangeFull(conv) => {
                let _ = conv.coerce_ref(range);

                (Bound::Unbounded, Bound::Unbounded)
            }
        }
    }
}

impl<R> RangeWit<R> {
    /// Convert this range into an [`IndexRange`].
    ///
    /// # Panics
    ///
    /// Panics if the conversion fails.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn into_range(self, range: R, bounds: ops::RangeTo<usize>) -> IndexRange {
        match into_range(self, range, bounds) {
            Ok(range) => range,
            Err(err) => err.fail(),
        }
    }

    /// Convert this range into an [`IndexRange`] if it is within bounds.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn into_range_checked(
        self,
        range: R,
        bounds: ops::RangeTo<usize>,
    ) -> Option<IndexRange> {
        match into_range(self, range, bounds) {
            Ok(range) => Some(range),
            Err(_) => None,
        }
    }

    /// Convert this range into an [`IndexRange`] without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided range is within bounds.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn into_range_unchecked(
        self,
        range: R,
        bounds: ops::RangeTo<usize>,
    ) -> IndexRange {
        match into_range(self, range, bounds) {
            Ok(range) => range,
            // SAFETY: The caller ensures this is acceptable.
            Err(error) => unsafe { error.unreachable_unchecked() },
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn into_bounds(self, range: R) -> (Bound<usize>, Bound<usize>) {
        let range = NoDrop::new(range);

        let (start, end) = self.as_bounds(range.as_ref());

        (bound_copied(start), bound_copied(end))
    }
}

impl<R: ?Sized> Clone for RangeWit<R> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: ?Sized> Copy for RangeWit<R> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RangeError {
    /// Start index overflow.
    StartOverflow,
    /// End index overflow.
    EndOverflow,
    /// Start is larger than end.
    StartLarge,
    /// End is larger than len.
    EndLarge,
}

impl RangeError {
    /// Panic with some error message for this error.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    #[cold]
    const fn fail(self) -> ! {
        match self {
            RangeError::StartOverflow => panic!("start overflow"),
            RangeError::EndOverflow => panic!("end overflow"),
            RangeError::StartLarge => panic!("start > end"),
            RangeError::EndLarge => panic!("end > len"),
        }
    }

    /// Assert that the code path producing this error is impossible.
    ///
    /// # Safety
    ///
    /// This is, extremely fucked.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    const unsafe fn unreachable_unchecked(self) -> ! {
        // SAFETY: The caller ensures this is fine.
        unsafe {
            match self {
                RangeError::StartOverflow => unreachable_unchecked!("start overflow"),
                RangeError::EndOverflow => unreachable_unchecked!("end overflow"),
                RangeError::StartLarge => unreachable_unchecked!("start > end"),
                RangeError::EndLarge => unreachable_unchecked!("end > len"),
            }
        }
    }
}

/// Create an [`IndexRange`] from some bounds.
///
/// This is used to implement the other conversions.
#[inline(always)]
#[must_use]
#[track_caller]
const fn into_range<R>(
    wit: RangeWit<R>,
    range: R,
    bounds: ops::RangeTo<usize>,
) -> Result<IndexRange, RangeError> {
    let len = bounds.end;
    let (start, end) = wit.into_bounds(range);

    let start = match start {
        Bound::Included(start) => start,
        Bound::Excluded(start) => match start.checked_add(1) {
            Some(start) => start,
            None => return Err(RangeError::StartOverflow),
        },
        Bound::Unbounded => 0,
    };

    let end = match end {
        Bound::Included(end) => match end.checked_add(1) {
            Some(end) => end,
            None => return Err(RangeError::EndOverflow),
        },
        Bound::Excluded(end) => end,
        Bound::Unbounded => len,
    };

    let Some(range) = IndexRange::new(start, end) else {
        return Err(RangeError::StartLarge);
    };

    if range.end() > len {
        return Err(RangeError::EndLarge);
    }

    Ok(range)
}

/// Just a wrapper around a [`RangeWit`] that we can expose publicly.
#[repr(transparent)]
pub struct RangeKind<R: ?Sized>(pub(crate) RangeWit<R>);

impl<R: ?Sized> Clone for RangeKind<R> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: ?Sized> Copy for RangeKind<R> {}
