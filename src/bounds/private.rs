use crate::{
    bounds::{SliceBounds, SliceRange},
    marker::TypeEq,
    util::bound_ref,
};
use core::ops::{self, Bound};

/// Trait to seal what types we're considering to be ranges.
pub trait Sealed {}

macro_rules! define_bounds {
    ($(
        $(#[doc = $doc:tt])*
        $(#[cfg($cfg:meta)])?

        $variant:ident($pat:pat => $ty:ty) => $bounds:expr
    ),* $(,)?) => {
        /// A type witness for the supported range bounds types.
        #[non_exhaustive]
        pub(crate) enum BoundsWit<B: SliceBounds + ?Sized> {
            $(
                $(#[doc = $doc])*
                $(#[cfg($cfg)])?
                $variant(TypeEq<B, $ty>),
            )*
        }

        impl<B: SliceBounds + ?Sized> BoundsWit<B> {
            /// Borrow the bounds of some `&B` as a tuple.
            #[inline(always)]
            #[must_use]
            pub(crate) const fn as_bounds(self, bounds: &B) -> (Bound<&usize>, Bound<&usize>) {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant(conv) => {
                            let $pat = conv.coerce_ref(bounds);

                            $bounds
                        },
                    )*
                }
            }
        }

        impl<B: SliceBounds + ?Sized> Clone for BoundsWit<B> {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<B: SliceBounds + ?Sized> Copy for BoundsWit<B> {}


        /// A wrapper for [`BoundsWit`] that can be exposed publicly.
        #[repr(transparent)]
        pub struct BoundsKind<B: SliceBounds + ?Sized>(pub(crate) BoundsWit<B>);

        impl<B: SliceBounds + ?Sized> Clone for BoundsKind<B> {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<B: SliceBounds + ?Sized> Copy for BoundsKind<B> {}

        $(
            $(#[cfg($cfg)])?
            impl Sealed for $ty {}

            $(#[cfg($cfg)])?
            unsafe impl super::SliceBounds for $ty
            {
                type Inner = $ty;
                const KIND: BoundsKind<$ty> = BoundsKind(BoundsWit::$variant(TypeEq::new()));
            }
        )*
    };
}

define_bounds! {
    /// The [`SliceRange`] type.
    SliceRange(range => SliceRange) => (
        Bound::Included(range.start_ref()),
        Bound::Excluded(range.end_ref())
    ),
    /// The [`ops::Range`] type.
    Range(range => ops::Range<usize>) => (
        Bound::Included(&range.start),
        Bound::Excluded(&range.end),
    ),

    /// The [`ops::RangeInclusive`] type.
    ///
    /// Currently this type is disabled because we cannot,
    /// from `const`, get the actual end bound of the range
    /// in a manner consistent with [`ops::RangeBounds`].
    ///
    /// Until this is possible, we're choosing not to support this type.
    #[cfg(all())]
    RangeInclusive(range => ops::RangeInclusive<usize>) => (
        Bound::Included(range.start()),
        Bound::Excluded(range.end()),
    ),
    /// The [`ops::RangeTo`] type.
    RangeTo(range => ops::RangeTo<usize>) => (
        Bound::Unbounded,
        Bound::Excluded(&range.end),
    ),
    /// The [`ops::RangeToInclusive`] type.
    RangeToInclusive(range => ops::RangeToInclusive<usize>) => (
        Bound::Unbounded,
        Bound::Included(&range.end),
    ),
    /// The [`ops::RangeFrom`] type.
    RangeFrom(range => ops::RangeFrom<usize>) => (
        Bound::Included(&range.start),
        Bound::Unbounded,
    ),
    /// The [`ops::RangeFull`] type.
    RangeFull(_ => ops::RangeFull) => (
        Bound::Unbounded,
        Bound::Unbounded,
    ),

    /// The [`(Bound<usize>, Bound<usize>)`] type.
    Bounds((start, end) => (Bound<usize>, Bound<usize>)) => (
        bound_ref(start),
        bound_ref(end),
    ),
}
