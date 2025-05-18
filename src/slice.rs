#![allow(dead_code)]

// //! This module contains the implementation details of stuff relating slices and indices thereof.
// //!
// //! Downstream crate ***must never*** rely upon the implementation of these traits.
// //!
// //! They're hidden away for a reason.
// use core::{
//     convert::Infallible,
//     ops::{self, Bound, Range},
//     str::Utf8Error,
// };

// use crate::{marker::TypeEq, util};

// /// The actual implementation of [`super::Slice`].
// pub unsafe trait Slice {
//     /// What kind of "raw item"s are stored within this slice.
//     ///
//     /// Do note that just because a slice has this item type set to this,
//     /// that does not imply all `[Self::Item]`s are valid instances of this
//     /// slice type.
//     ///
//     /// # Safety
//     ///
//     /// Implementors must guarantee that while not all `[Self::Item]`s are valid
//     /// instances of `Self`, all instances of `Self` can be be constructed from
//     /// some `[Self::Item]`.
//     ///
//     /// Implementors should supply some kind of error for when construction fails.
//     type Item: Sized;

//     /// An error that is returned when constructing a `Self` from a `[Self::Item]`
//     /// fails.
//     type Error: Sized;

//     /// A constant storing a type witness used for observers of this trait
//     /// to implement polymorphism that works in const.
//     const WITNESS: SliceWit<Self>;

//     /// A constant denoting whether the item for this slice is a zero sized type.
//     ///
//     /// This is purely for convenience.
//     const IS_ZST: bool = size_of::<Self::Item>() == 0;
// }

// /// Just a wrapper around [`SliceKind`] that we can expose publicly.
// pub struct SliceWit<S: Slice + ?Sized>(pub(crate) SliceKind<S>);

// /// An enum representing what kind of slice this is.
// pub(crate) enum SliceKind<S: Slice + ?Sized> {
//     /// This slice is just a normal slice.
//     Slice {
//         this: TypeEq<S, [S::Item]>,
//         error: TypeEq<S::Error, Infallible>,
//         items: TypeEq<[S::Item], [S::Item]>,
//     },
//     /// This slice is a utf-8 string.
//     Str {
//         this: TypeEq<S, str>,
//         error: TypeEq<S::Error, Utf8Error>,
//         items: TypeEq<[S::Item], [u8]>,
//     },
// }

// impl<S: Slice + ?Sized> Clone for SliceKind<S> {
//     #[inline(always)]
//     fn clone(&self) -> Self {
//         *self
//     }
// }

// impl<S: Slice + ?Sized> Copy for SliceKind<S> {}

// /// The actual implementation of [`super::SliceIndex`].
// pub unsafe trait SliceIndex<S: Slice + ?Sized> {
//     /// The output of the index operation.
//     type Output: ?Sized;

//     /// A type witness to allow observers of this trait to implement
//     /// polymorphism that works in const.
//     const WITNESS: IndexWit<Self, S>;
// }

// /// Just a wrapper around [`IndexKind`] that we can expose publicly.
// pub struct IndexWit<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized>(pub(crate) IndexKind<I, S>);

// /// Index types supported for slices.
// pub(crate) enum IndexKind<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> {
//     /// An actual index.
//     Index {
//         index: TypeEq<I, usize>,
//         output: TypeEq<I::Output, S::Item>,
//     },
//     /// Some bounds.
//     Bounds {
//         index: TypeEq<I, (Bound<usize>, Bound<usize>)>,
//         output: TypeEq<I::Output, S>,
//     },
//     /// Range type provided by [`core::ops`].
//     Range {
//         index: RangeKind<I>,
//         output: TypeEq<I::Output, S>,
//     },
// }

// impl<I: SliceIndex<S>, S: Slice + ?Sized> IndexKind<I, S> {
//     /// Returns the type equality for the output if it is an
//     /// index type that returns a subslice.
//     #[inline(always)]
//     #[must_use]
//     #[track_caller]
//     pub const fn output(self) -> Option<TypeEq<I::Output, S>> {
//         match self {
//             IndexKind::Index { .. } => None,
//             IndexKind::Bounds { output, .. } => Some(output),
//             IndexKind::Range { output, .. } => Some(output),
//         }
//     }

//     /// Convert an index `I` into its bounds.
//     #[inline(always)]
//     #[must_use]
//     pub const fn into_bounds(self, i: I) -> (Bound<usize>, Bound<usize>) {
//         match self {
//             IndexKind::Index { index, .. } => {
//                 let i = index.coerce(i);

//                 (Bound::Included(i), Bound::Included(i))
//             }
//             IndexKind::Bounds { index, .. } => index.coerce(i),
//             IndexKind::Range { index, .. } => index.into_bounds(i),
//         }
//     }

//     /// Convert an index `I` into its range without checks.
//     #[inline(always)]
//     #[must_use]
//     #[track_caller]
//     pub const fn into_range(self, len: usize, index: I) -> Range<usize> {
//         util::into_range(len, self.into_bounds(index))
//     }

//     /// Convert an index `I` into its range.
//     #[inline(always)]
//     #[must_use]
//     #[track_caller]
//     pub const fn into_range_checked(self, len: usize, index: I) -> Option<Range<usize>> {
//         util::into_range_checked(len, self.into_bounds(index))
//     }
// }

// impl<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> Clone for IndexKind<I, S> {
//     #[inline(always)]
//     fn clone(&self) -> Self {
//         *self
//     }
// }

// impl<I: SliceIndex<S> + ?Sized, S: Slice + ?Sized> Copy for IndexKind<I, S> {}

// /// Range types provided by [`core::ops`].
// pub(crate) enum RangeKind<R: ?Sized> {
//     Range(TypeEq<R, ops::Range<usize>>),
//     RangeInclusive(TypeEq<R, ops::RangeInclusive<usize>>),
//     RangeTo(TypeEq<R, ops::RangeTo<usize>>),
//     RangeToInclusive(TypeEq<R, ops::RangeToInclusive<usize>>),
//     RangeFrom(TypeEq<R, ops::RangeFrom<usize>>),
//     RangeFull(TypeEq<R, ops::RangeFull>),
// }

// impl<R> RangeKind<R> {
//     /// Convert a range `R` into its bounds.
//     #[inline(always)]
//     #[must_use]
//     pub const fn into_bounds(self, range: R) -> (Bound<usize>, Bound<usize>) {
//         match self {
//             RangeKind::Range(conv) => {
//                 let range = conv.coerce(range);

//                 (Bound::Included(range.start), Bound::Excluded(range.end))
//             }
//             RangeKind::RangeInclusive(conv) => {
//                 let range = conv.coerce(range);

//                 (
//                     Bound::Included(*range.start()),
//                     Bound::Included(*range.end()),
//                 )
//             }
//             RangeKind::RangeTo(conv) => {
//                 let range = conv.coerce(range);

//                 (Bound::Unbounded, Bound::Excluded(range.end))
//             }
//             RangeKind::RangeToInclusive(conv) => {
//                 let range = conv.coerce(range);

//                 (Bound::Unbounded, Bound::Included(range.end))
//             }
//             RangeKind::RangeFrom(conv) => {
//                 let range = conv.coerce(range);

//                 (Bound::Included(range.start), Bound::Unbounded)
//             }
//             RangeKind::RangeFull(conv) => {
//                 let _ = conv.coerce(range);

//                 (Bound::Unbounded, Bound::Unbounded)
//             }
//         }
//     }
// }

// impl<R: ?Sized> Clone for RangeKind<R> {
//     #[inline(always)]
//     fn clone(&self) -> Self {
//         *self
//     }
// }

// impl<R: ?Sized> Copy for RangeKind<R> {}

// // Slice implementations.

// unsafe impl<T> Slice for [T] {
//     type Item = T;
//     type Error = Infallible;

//     const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Slice {
//         this: TypeEq::new(),
//         error: TypeEq::new(),
//         items: TypeEq::new(),
//     });
// }

// unsafe impl Slice for str {
//     type Item = u8;
//     type Error = Utf8Error;

//     const WITNESS: SliceWit<Self> = SliceWit(SliceKind::Str {
//         this: TypeEq::new(),
//         error: TypeEq::new(),
//         items: TypeEq::new(),
//     });
// }

// // unsafe impl<T> SliceIndex<[T]> for ops::Range<usize> {
// //     type Output = [T];

// //     const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
// //         RangeKind::Range(TypeEq::new()),
// //         TypeEq::new(),
// //     ));
// // }

// // unsafe impl<T> SliceIndex<[T]> for ops::RangeInclusive<usize> {
// //     type Output = [T];

// //     const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Range(
// //         RangeKind::RangeInclusive(TypeEq::new()),
// //         TypeEq::new(),
// //     ));
// // }
// //

// macro_rules! range_index {
//     ($scope:ident, $var:ident, $kind:ident, $ty:ty, $($gen:tt)*) => {
//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::Range<usize> {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::Range(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }

//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeInclusive<usize> {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::RangeInclusive(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }

//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeTo<usize> {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::RangeTo(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }

//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeToInclusive<usize> {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::RangeToInclusive(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }

//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeFrom<usize> {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::RangeFrom(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }

//         unsafe impl $($gen)* SliceIndex<$ty> for ::core::$scope::RangeFull {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::$var {
//                 index: $kind::RangeFull(TypeEq::new()),
//                 output: TypeEq::new(),
//             });
//         }
//     };
// }

// macro_rules! blanket_index {
//     ($ty:ty, $($gen:tt)*) => {
//         unsafe impl $($gen)* SliceIndex<$ty> for (Bound<usize>, Bound<usize>) {
//             type Output = $ty;

//             const WITNESS: IndexWit<Self, $ty> = IndexWit(IndexKind::Bounds {
//                 index: TypeEq::new(),
//                 output: TypeEq::new(),
//             });
//         }

//         // Handle the core::ops ranges.
//         range_index!(ops, Range, RangeKind, $ty, $($gen)*);

//         // TODO: Handle the future new range api.
//     };
// }

// blanket_index!([T], <T>);
// blanket_index!(str,);

// `usize` is a special case.
// unsafe impl<T> SliceIndex<[T]> for usize {
//     type Output = T;

//     const WITNESS: IndexWit<Self, [T]> = IndexWit(IndexKind::Index {
//         index: TypeEq::new(),
//         output: TypeEq::new(),
//     });
// }

// /// Trait for the various supported slices.
// pub unsafe trait Slice: private::Slice {
//     /// Returns the length of this slice.
//     #[inline(always)]
//     #[must_use]
//     fn len(&self) -> usize {
//         len(self)
//     }

//     #[inline(always)]
//     #[must_use]
//     fn from_elems(elems: &[Self::Item]) -> Result<&Self, Self::Error> {
//         from_elems(elems)
//     }

//     #[inline(always)]
//     #[must_use]
//     fn from_elems_mut(elems: &mut [Self::Item]) -> Result<&mut Self, Self::Error> {
//         from_elems_mut(elems)
//     }

//     #[inline(always)]
//     #[must_use]
//     #[track_caller]
//     unsafe fn from_elems_unchecked(elems: &[Self::Item]) -> &Self {
//         unsafe { from_elems_unchecked(elems) }
//     }

//     #[inline(always)]
//     #[must_use]
//     #[track_caller]
//     unsafe fn from_elems_mut_unchecked(elems: &mut [Self::Item]) -> &mut Self {
//         unsafe { from_elems_mut_unchecked(elems) }
//     }
// }

// unsafe impl<S: private::Slice + ?Sized> Slice for S {}

// /// Trait for the various supported slice indexes.
// pub unsafe trait SliceIndex<S: Slice + ?Sized>: private::SliceIndex<S> {}

// unsafe impl<I, S> SliceIndex<S> for I
// where
//     I: private::SliceIndex<S> + ?Sized,
//     S: Slice + ?Sized,
// {
// }

// /// Returns the length of a given slice pointer.
// #[inline(always)]
// #[must_use]
// pub const fn len<S: Slice + ?Sized>(slice: *const S) -> usize {
//     match S::WITNESS.0 {
//         SliceKind::Slice { this: conv, .. } => conv.coerce_ptr(slice).len(),
//         SliceKind::Str { this: conv, .. } => (conv.coerce_ptr(slice) as *const [u8]).len(),
//     }
// }

// /// Given an error returned by attempting to create a slice, return an error string.
// #[inline(always)]
// #[must_use]
// pub const fn error_string<S: Slice + ?Sized>(err: S::Error) -> &'static str {
//     let err = NoDrop::new(err);

//     match S::WITNESS.0 {
//         #[allow(unreachable_code)]
//         SliceKind::Slice { error, .. } => match error.coerce(err.into_inner()) {},
//         SliceKind::Str { .. } => "invalid utf-8",
//     }
// }

// /// Attempts to create a slice from a slice of its elements.
// #[inline(always)]
// #[must_use]
// pub const fn from_elems<S: Slice + ?Sized>(elems: &[S::Item]) -> Result<&S, S::Error> {
//     match S::WITNESS.0 {
//         SliceKind::Slice { this, .. } => Ok(this.uncoerce_ref(elems)),
//         SliceKind::Str { this, items, error } => this
//             .wrap_ref()
//             .wrap_result(error)
//             .uncoerce(core::str::from_utf8(items.coerce_ref(elems))),
//     }
// }

// /// Attempts to create a mutable slice from a mutable slice of its elements.
// #[inline(always)]
// #[must_use]
// pub const fn from_elems_mut<S: Slice + ?Sized>(elems: &mut [S::Item]) -> Result<&mut S, S::Error> {
//     match S::WITNESS.0 {
//         SliceKind::Slice { this, .. } => Ok(this.uncoerce_mut(elems)),
//         SliceKind::Str { this, error, items } => this
//             .wrap_mut()
//             .wrap_result(error)
//             .uncoerce(core::str::from_utf8_mut(items.coerce_mut(elems))),
//     }
// }

// /// Creates a slice from a slice of its elements without any checks.
// ///
// /// # Safety
// ///
// /// The caller must ensure that the provided element slice is indeed valid
// /// according to the invariants of `S`.
// ///
// /// Failure to do so will likely result in undefined behavior.
// #[inline(always)]
// #[must_use]
// #[track_caller]
// pub const unsafe fn from_elems_unchecked<S: Slice + ?Sized>(elems: &[S::Item]) -> &S {
//     match S::WITNESS.0 {
//         SliceKind::Slice { this, .. } => this.uncoerce_ref(elems),
//         SliceKind::Str { this, items, .. } => {
//             this.uncoerce_ref(unsafe { core::str::from_utf8_unchecked(items.coerce_ref(elems)) })
//         }
//     }
// }

// /// Creates a mutable slice from a mutable slice of its elements without any checks.
// ///
// /// # Safety
// ///
// /// The caller must ensure that the provided element slice is indeed valid
// /// according to the invariants of `S`.
// ///
// /// Failure to do so will likely result in undefined behavior.
// #[inline(always)]
// #[must_use]
// #[track_caller]
// pub const unsafe fn from_elems_mut_unchecked<S: Slice + ?Sized>(elems: &mut [S::Item]) -> &mut S {
//     match S::WITNESS.0 {
//         SliceKind::Slice { this, .. } => this.uncoerce_mut(elems),
//         SliceKind::Str { this, items, .. } => this
//             .uncoerce_mut(unsafe { core::str::from_utf8_unchecked_mut(items.coerce_mut(elems)) }),
//     }
// }
