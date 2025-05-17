//! This module contains the implementation details of stuff relating slices and indices thereof.
//!
//! Downstream crate ***must never*** rely upon the implementation of these traits.
//!
//! They're hidden away for a reason.
use core::ops::{self, Bound, Range};

use crate::{marker::TypeEq, util};

/// The actual implementation of [`super::Slice`].
pub unsafe trait Slice {
    /// What kind of "raw item"s are stored within this slice.
    ///
    /// Do note that just because a slice has this item type set to this,
    /// that does not imply all `[Self::Item]`s are valid instances of this
    /// slice type.
    type Item: Sized;

    /// A constant storing a type witness used for observers of this trait
    /// to implement polymorphism that works in const.
    const WITNESS: SliceWit<Self>;

    /// A constant denoting whether the item for this slice is a zero sized type.
    ///
    /// This is purely for convenience.
    const IS_ZST: bool = size_of::<Self::Item>() == 0;
}

/// The actual implementation of [`super::SliceIndex`].
pub unsafe trait SliceIndex<S: Slice + ?Sized> {
    type Output: ?Sized;

    /// A type witness to allow observers of this trait to implement
    /// polymorphism that works in const.
    const WITNESS: IndexWit<Self, S>;
}

/// Just a wrapper around [`SliceKind`] that we can expose publicly.
pub struct SliceWit<S: Slice + ?Sized>(pub(crate) SliceKind<S>);

/// Just a wrapper around [`IndexKind`] that we can expose publicly.
pub struct IndexWit<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized>(pub(crate) IndexKind<I, S>);

/// An enum representing what kind of slice this is.
pub(crate) enum SliceKind<S: Slice + ?Sized> {
    /// This slice is just a normal slice.
    Slice(TypeEq<S, [S::Item]>),
    /// This slice is a utf-8 string.
    Str(TypeEq<S, str>),
}

impl<S: Slice + ?Sized> Clone for SliceKind<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for SliceKind<S> {}

/// Index types supported for slices.
pub(crate) enum IndexKind<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> {
    /// An actual index.
    Index(TypeEq<I, usize>, TypeEq<I::Output, S::Item>),
    /// Some bounds.
    Bounds(
        TypeEq<I, (Bound<usize>, Bound<usize>)>,
        TypeEq<I::Output, S>,
    ),
    /// Range type provided by [`core::ops`].
    Range(RangeKind<I>, TypeEq<I::Output, S>),
}

impl<I: SliceIndex<S>, S: Slice + ?Sized> IndexKind<I, S> {
    /// Returns the type equality for the output.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn output(self) -> TypeEq<I::Output, S> {
        match self {
            IndexKind::Bounds(_, output) => output,
            IndexKind::Range(_, output) => output,
            _ => panic!("unsupported index kind"),
        }
    }

    /// Convert an index `I` into its bounds.
    #[inline(always)]
    #[must_use]
    pub const fn into_bounds(self, index: I) -> (Bound<usize>, Bound<usize>) {
        match self {
            IndexKind::Index(conv, ..) => {
                let index = conv.coerce(index);

                (Bound::Included(index), Bound::Included(index))
            }
            IndexKind::Bounds(conv, ..) => conv.coerce(index),
            IndexKind::Range(kind, ..) => kind.into_bounds(index),
        }
    }

    /// Convert an index `I` into its range without checks.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn into_range(self, len: usize, index: I) -> Range<usize> {
        util::into_range(len, self.into_bounds(index))
    }

    /// Convert an index `I` into its range.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn into_range_checked(self, len: usize, index: I) -> Option<Range<usize>> {
        util::into_range_checked(len, self.into_bounds(index))
    }
}

impl<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> Clone for IndexKind<I, S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> Copy for IndexKind<I, S> {}

/// Range types provided by [`core::ops`].
pub(crate) enum RangeKind<R: ?Sized> {
    Range(TypeEq<R, ops::Range<usize>>),
    RangeInclusive(TypeEq<R, ops::RangeInclusive<usize>>),
    RangeTo(TypeEq<R, ops::RangeTo<usize>>),
    RangeToInclusive(TypeEq<R, ops::RangeToInclusive<usize>>),
    RangeFrom(TypeEq<R, ops::RangeFrom<usize>>),
    RangeFull(TypeEq<R, ops::RangeFull>),
}

impl<R> RangeKind<R> {
    /// Convert a range `R` into its bounds.
    #[inline(always)]
    #[must_use]
    pub const fn into_bounds(self, range: R) -> (Bound<usize>, Bound<usize>) {
        match self {
            RangeKind::Range(conv) => {
                let range = conv.coerce(range);

                (Bound::Included(range.start), Bound::Excluded(range.end))
            }
            RangeKind::RangeInclusive(conv) => {
                let range = conv.coerce(range);

                (
                    Bound::Included(*range.start()),
                    Bound::Included(*range.end()),
                )
            }
            RangeKind::RangeTo(conv) => {
                let range = conv.coerce(range);

                (Bound::Unbounded, Bound::Excluded(range.end))
            }
            RangeKind::RangeToInclusive(conv) => {
                let range = conv.coerce(range);

                (Bound::Unbounded, Bound::Included(range.end))
            }
            RangeKind::RangeFrom(conv) => {
                let range = conv.coerce(range);

                (Bound::Included(range.start), Bound::Unbounded)
            }
            RangeKind::RangeFull(conv) => {
                let _ = conv.coerce(range);

                (Bound::Unbounded, Bound::Unbounded)
            }
        }
    }
}

impl<R: ?Sized> Clone for RangeKind<R> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: ?Sized> Copy for RangeKind<R> {}

// Slice implementations.

unsafe impl<T> Slice for [T] {
    type Item = T;

    const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Slice(TypeEq::new()));
}

unsafe impl Slice for str {
    type Item = u8;

    const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Str(TypeEq::new()));
}

// Slice index implementations for `[T]`

unsafe impl<T> SliceIndex<[T]> for usize {
    type Output = T;

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Index(TypeEq::new(), TypeEq::new()));
}

unsafe impl<T> SliceIndex<[T]> for (Bound<usize>, Bound<usize>) {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Bounds(TypeEq::new(), TypeEq::new()));
}

unsafe impl<T> SliceIndex<[T]> for ops::Range<usize> {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::Range(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl<T> SliceIndex<[T]> for ops::RangeInclusive<usize> {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::RangeInclusive(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl<T> SliceIndex<[T]> for ops::RangeTo<usize> {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::RangeTo(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl<T> SliceIndex<[T]> for ops::RangeToInclusive<usize> {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::RangeToInclusive(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl<T> SliceIndex<[T]> for ops::RangeFrom<usize> {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::RangeFrom(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl<T> SliceIndex<[T]> for ops::RangeFull {
    type Output = [T];

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
        RangeKind::RangeFull(TypeEq::new()),
        TypeEq::new(),
    ));
}

// Slice index implementations for `str`.

unsafe impl SliceIndex<str> for (Bound<usize>, Bound<usize>) {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Bounds(TypeEq::new(), TypeEq::new()));
}

unsafe impl SliceIndex<str> for ops::Range<usize> {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::Range(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl SliceIndex<str> for ops::RangeInclusive<usize> {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::RangeInclusive(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl SliceIndex<str> for ops::RangeTo<usize> {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::RangeTo(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl SliceIndex<str> for ops::RangeToInclusive<usize> {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::RangeToInclusive(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl SliceIndex<str> for ops::RangeFrom<usize> {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::RangeFrom(TypeEq::new()),
        TypeEq::new(),
    ));
}

unsafe impl SliceIndex<str> for ops::RangeFull {
    type Output = str;

    const WITNESS: IndexWit<Self, str> = IndexWit(IndexKind::Range(
        RangeKind::RangeFull(TypeEq::new()),
        TypeEq::new(),
    ));
}
