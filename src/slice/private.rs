//! This module contains the implementation details of stuff relating slices and indices thereof.
//!
//! Downstream crate ***must never*** rely upon the implementation of these traits.
//!
//! They're hidden away for a reason.
use core::{
    convert::Infallible,
    ops::{self, Bound, Range},
    str::Utf8Error,
};

use crate::{marker::TypeEq, util};

/// The actual implementation of [`super::Slice`].
pub unsafe trait Slice {
    /// What kind of "raw item"s are stored within this slice.
    ///
    /// Do note that just because a slice has this item type set to this,
    /// that does not imply all `[Self::Item]`s are valid instances of this
    /// slice type.
    ///
    /// # Safety
    ///
    /// Implementors must guarantee that while not all `[Self::Item]`s are valid
    /// instances of `Self`, all instances of `Self` can be be constructed from
    /// some `[Self::Item]`.
    ///
    /// Implementors should supply some kind of error for when construction fails.
    type Item: Sized;

    /// An error that is returned when constructing a `Self` from a `[Self::Item]`
    /// fails.
    type Error: Sized;

    /// A constant storing a type witness used for observers of this trait
    /// to implement polymorphism that works in const.
    const WITNESS: SliceWit<Self>;

    /// A constant denoting whether the item for this slice is a zero sized type.
    ///
    /// This is purely for convenience.
    const IS_ZST: bool = size_of::<Self::Item>() == 0;
}

/// Just a wrapper around [`SliceKind`] that we can expose publicly.
pub struct SliceWit<S: Slice + ?Sized>(pub(crate) SliceKind<S>);

/// An enum representing what kind of slice this is.
pub(crate) enum SliceKind<S: Slice + ?Sized> {
    /// This slice is just a normal slice.
    Slice {
        this: TypeEq<S, [S::Item]>,
        error: TypeEq<S::Error, Infallible>,
        items: TypeEq<[S::Item], [S::Item]>,
    },
    /// This slice is a utf-8 string.
    Str {
        this: TypeEq<S, str>,
        error: TypeEq<S::Error, Utf8Error>,
        items: TypeEq<[S::Item], [u8]>,
    },
}

impl<S: Slice + ?Sized> Clone for SliceKind<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for SliceKind<S> {}

/// The actual implementation of [`super::SliceIndex`].
pub unsafe trait SliceIndex<S: Slice + ?Sized> {
    /// The output of the index operation.
    type Output: ?Sized;

    /// A type witness to allow observers of this trait to implement
    /// polymorphism that works in const.
    const WITNESS: IndexWit<Self, S>;
}

/// Just a wrapper around [`IndexKind`] that we can expose publicly.
pub struct IndexWit<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized>(pub(crate) IndexKind<I, S>);

/// Index types supported for slices.
pub(crate) enum IndexKind<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> {
    /// An actual index.
    Index {
        index: TypeEq<I, usize>,
        output: TypeEq<I::Output, S::Item>,
    },
    /// Some bounds.
    Bounds {
        index: TypeEq<I, (Bound<usize>, Bound<usize>)>,
        output: TypeEq<I::Output, S>,
    },
    /// Range type provided by [`core::ops`].
    Range {
        index: RangeKind<I>,
        output: TypeEq<I::Output, S>,
    },
}

impl<I: SliceIndex<S>, S: Slice + ?Sized> IndexKind<I, S> {
    /// Returns the type equality for the output if it is an
    /// index type that returns a subslice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn output(self) -> Option<TypeEq<I::Output, S>> {
        match self {
            IndexKind::Index { .. } => None,
            IndexKind::Bounds { output, .. } => Some(output),
            IndexKind::Range { output, .. } => Some(output),
        }
    }

    /// Convert an index `I` into its bounds.
    #[inline(always)]
    #[must_use]
    pub const fn into_bounds(self, i: I) -> (Bound<usize>, Bound<usize>) {
        match self {
            IndexKind::Index { index, .. } => {
                let i = index.coerce(i);

                (Bound::Included(i), Bound::Included(i))
            }
            IndexKind::Bounds { index, .. } => index.coerce(i),
            IndexKind::Range { index, .. } => index.into_bounds(i),
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
    type Error = Infallible;

    const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Slice {
        this: TypeEq::new(),
        error: TypeEq::new(),
        items: TypeEq::new(),
    });
}

unsafe impl Slice for str {
    type Item = u8;
    type Error = Utf8Error;

    const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Str {
        this: TypeEq::new(),
        error: TypeEq::new(),
        items: TypeEq::new(),
    });
}

// unsafe impl<T> SliceIndex<[T]> for ops::Range<usize> {
//     type Output = [T];

//     const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
//         RangeKind::Range(TypeEq::new()),
//         TypeEq::new(),
//     ));
// }

// unsafe impl<T> SliceIndex<[T]> for ops::RangeInclusive<usize> {
//     type Output = [T];

//     const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
//         RangeKind::RangeInclusive(TypeEq::new()),
//         TypeEq::new(),
//     ));
// }
//

macro_rules! range_index {
    ($scope:ident, $var:ident, $kind:ident, $ty:ty, $($gen:tt)*) => {
        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::Range<usize> {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::Range(TypeEq::new()),
                output: TypeEq::new(),
            });
        }

        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeInclusive<usize> {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::RangeInclusive(TypeEq::new()),
                output: TypeEq::new(),
            });
        }

        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeTo<usize> {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::RangeTo(TypeEq::new()),
                output: TypeEq::new(),
            });
        }

        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeToInclusive<usize> {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::RangeToInclusive(TypeEq::new()),
                output: TypeEq::new(),
            });
        }

        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeFrom<usize> {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::RangeFrom(TypeEq::new()),
                output: TypeEq::new(),
            });
        }

        unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeFull {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
                index: $kind::RangeFull(TypeEq::new()),
                output: TypeEq::new(),
            });
        }
    };
}

macro_rules! blanket_index {
    ($ty:ty, $($gen:tt)*) => {
        unsafe impl $($gen)* SliceIndex<$ty> for (Bound<usize>, Bound<usize>) {
            type Output = $ty;

            const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::Bounds {
                index: TypeEq::new(),
                output: TypeEq::new(),
            });
        }

        // Handle the core::ops ranges.
        range_index!(ops, Range, RangeKind, $ty, $($gen)*);

        // TODO: Handle the future new range api.
    };
}

blanket_index!([T], <T>);
blanket_index!(str,);

// `usize` is a special case.
unsafe impl<T> SliceIndex<[T]> for usize {
    type Output = T;

    const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Index {
        index: TypeEq::new(),
        output: TypeEq::new(),
    });
}
