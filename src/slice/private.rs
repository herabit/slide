#![allow(dead_code)]

use core::{
    convert::Infallible,
    ptr::{self, NonNull},
    str::Utf8Error,
};

use crate::{marker::TypeEq, slice::Slice};

/// Trait to seal what we consider to be slices.
pub trait Sealed {}

macro_rules! get_impl {
    ($ty:ty) => {
        $ty
    };
    ($ty:ty = $impl_ty:ty) => {
        $impl_ty
    };
}

macro_rules! define_slices {
    ($(

        $(#[doc = $doc:expr])*
        $(#[cfg($cfg:meta)])?
        $variant:ident $( ( $($gen:tt)* ) )? {
            // The slice type.
            $(#[doc = $slice_doc:expr])*
            slice: $ty:ty $(= $impl_ty:ty)?,

            // The slice items type.
            $(#[doc = $item_doc:expr])*
            items: [$item_ty:ty] $(= [$impl_item_ty:ty])?,

            // The slice decode error type.
            $(#[doc = $decode_error_doc:expr])*
            decode_error: $decode_error_ty:ty $(= $impl_decode_error_ty:ty)?,

            // The slice length function.
            $(#[doc = $len_doc:expr])*
            len: |$len_params:pat_param| $len_body:expr,

            // The decode items function.
            $(#[doc = $decode_items_doc:expr])*
            decode_items: |$decode_items_params:pat_param| $decode_items_body:expr,

            // The decode items unchecked function.
            $(#[doc = $decode_items_unchecked_doc:expr])*
            decode_items_unchecked:
                |$decode_items_unchecked_params:pat_param| $decode_items_unchecked_body:expr,

            // The decode mutable items function.
            $(#[doc = $decode_items_mut_doc:expr])*
            decode_items_mut:
                |$decode_items_mut_params:pat_param| $decode_items_mut_body:expr,

            // The decode mutable items unchecked function.
            $(#[doc = $decode_items_mut_unchecked_doc:expr])*
            decode_items_mut_unchecked:
                |$decode_items_mut_unchecked_params:pat_param| $decode_items_mut_unchecked_body:expr,

            // The raw slice function.
            $(#[doc = $raw_slice_doc:expr])*
            raw_slice: |$raw_slice_params:pat_param| $raw_slice_body:expr,

            // The mutable raw slice function.
            $(#[doc = $raw_slice_mut_doc:expr])*
            raw_slice_mut: |$raw_slice_mut_params:pat_param| $raw_slice_mut_body:expr,

            // The nonnull raw slice function.
            $(#[doc = $raw_slice_nonnull_doc:expr])*
            raw_slice_nonnull: |$raw_slice_nonnull_params:pat_param| $raw_slice_nonnull_body:expr,

            // The from raw parts function.
            $(#[doc = $from_raw_parts_doc:expr])*
            from_raw_parts: $from_raw_parts_body:expr,

            // The from raw parts mut function.
            $(#[doc = $from_raw_parts_mut_doc:expr])*
            from_raw_parts_mut: $from_raw_parts_mut_body:expr,

            $(,)?
        }
    ),* $(,)?) => {

        /// A type witness for const polymorphism over types.
        #[non_exhaustive]
        pub(crate) enum SliceWit<S: Slice + ?Sized> {
            $(
                $(#[doc = $doc])*
                $(#[cfg($cfg)])?
                $variant {
                    slice: TypeEq<S, $ty>,
                    items: TypeEq<[S::Item], [$item_ty]>,
                    decode_error: TypeEq<S::DecodeError, $decode_error_ty>,
                },
            )*
        }

        impl<S: Slice + ?Sized> SliceWit<S> {
            /// Returns the length of a given slice.
            #[inline(always)]
            #[must_use]
            pub(crate) const fn len(self, slice: *const S) -> usize {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice: this,
                            ..
                        } => {
                            let $len_params = this.coerce_ptr(slice);

                            $len_body
                        },
                    )*
                }
            }

            /// Decode a slice of items into this slice type.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn decode_items(self, items: &[S::Item]) -> Result<&S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            decode_error,
                            items: this,
                        } => {
                            let result = slice.wrap_ref().wrap_result(decode_error);
                            let $decode_items_params = this.coerce_ref(items);

                            result.uncoerce($decode_items_body)
                        },
                    )*
                }
            }

            /// Decode a slice of items into this slice type without any checks.
            ///
            /// # Safety
            ///
            /// The caller must ensure that it is safe to construct an `&S` from the provided &[S::Item]`.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn decode_items_unchecked(self, items: &[S::Item]) -> &S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            items: this,
                            ..
                        } => {
                            let $decode_items_unchecked_params = this.coerce_ref(items);

                            slice.uncoerce_ref($decode_items_unchecked_body)
                        },
                    )*
                }
            }

            /// Decode a mutable slice of items into this slice type.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn decode_items_mut(self, items: &mut [S::Item]) -> Result<&mut S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            decode_error,
                            items: this,
                        } => {
                            let result = slice.wrap_mut().wrap_result(decode_error);
                            let $decode_items_mut_params = this.coerce_mut(items);

                            result.uncoerce($decode_items_mut_body)
                        },
                    )*
                }
            }

            /// Decode a mutable slice of items into this slice type without any checks.
            ///
            /// # Safety
            ///
            /// The caller must ensure that it is safe to construct an `&mut S` from the provided
            /// `&mut [S::Item]`.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn decode_items_mut_unchecked(self, items: &mut [S::Item]) -> &mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            items: this,
                            ..
                        } => {
                            let $decode_items_mut_unchecked_params = this.coerce_mut(items);

                            slice.uncoerce_mut($decode_items_mut_unchecked_body)
                        },
                    )*
                }
            }

            /// Create a raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Item`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice(self, data: *const S::Item, len: usize) -> *const S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            items,
                            ..
                        } => {
                            let $raw_slice_params = items.coerce_ptr(
                                ptr::slice_from_raw_parts(data, len),
                            );

                            slice.uncoerce_ptr($raw_slice_body)
                        },
                    )*
                }
            }

            /// Create a mutable raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Item`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_mut(self, data: *mut S::Item, len: usize) -> *mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            items,
                            ..
                        } => {
                            let $raw_slice_mut_params = items.coerce_ptr_mut(
                                ptr::slice_from_raw_parts_mut(data, len),
                            );

                            slice.uncoerce_ptr_mut($raw_slice_mut_body)
                        },
                    )*
                }
            }

            /// Create a [`NonNull`] raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Item`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_nonnull(self, data: NonNull<S::Item>, len: usize) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            items,
                            ..
                        } => {
                            let $raw_slice_nonnull_params = items.coerce_nonnull(
                                NonNull::slice_from_raw_parts(data, len),
                            );

                            slice.uncoerce_nonnull($raw_slice_nonnull_body)
                        },
                    )*
                }
            }

            /// Create a new slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Item`s the slice contains.
            ///
            /// # Safety
            ///
            /// The caller must ensure that the provided pointer and length can make
            /// a valid `&'a [Self::Item]` according to [`core::slice::from_raw_parts`].
            ///
            /// Additionally the caller must ensure that the `&'a [Self::Item]` created is a
            /// valid `&'a S`.
            ///
            /// Failure to ensure the above is undefined behavior.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts<'a>(self, data: *const S::Item, len: usize) -> &'a S {
                // match self {
                //     $(
                //         $(#[cfg($cfg)])?
                //         Self::$variant {
                //             slice,
                //             items,
                //             ..
                //         } => {
                //             todo!()
                //         },
                //     )*
                // }
                //
                todo!()
            }

            /// Create a new mutable slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Item`s the slice contains.
            ///
            /// # Safety
            ///
            /// The caller must ensure that the provided pointer and length can make
            /// a valid `&'a mut [Self::Item]` according to [`core::slice::from_raw_parts_mut`].
            ///
            /// Additionally the caller must ensure that the `&'a mut [Self::Item]` created is
            /// a valid `&'a mut S`.
            ///
            /// Failure to ensure the above is undefined behavior.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts_mut<'a>(self, data: *mut S::Item, len: usize) -> &'a mut S {
                todo!()
            }
        }

        impl<S: Slice + ?Sized> Clone for SliceWit<S> {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<S: Slice + ?Sized> Copy for SliceWit<S> {}

        /// A wrapper around a [`SliceWit`] that can be exposed publicly.
        #[repr(transparent)]
        pub struct SliceKind<S: Slice + ?Sized>(pub(crate) SliceWit<S>);

        impl<S: Slice + ?Sized> Clone for SliceKind<S> {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<S: Slice + ?Sized> Copy for SliceKind<S> {}

        $(
            $(#[cfg($cfg)])?
            impl $($($gen)*)? Sealed for get_impl!($ty $(= $impl_ty)?) {}

            $(#[cfg($cfg)])?
            $(#[doc = $slice_doc])*
            unsafe impl $($($gen)*)? Slice for get_impl!($ty $(= $impl_ty)?) {
                $(#[doc = $item_doc])*
                type Item = get_impl!($item_ty $(= $impl_item_ty)?);

                $(#[doc = $decode_error_doc])*
                type DecodeError = get_impl!($decode_error_ty $(= $impl_decode_error_ty)?);

                const KIND: SliceKind<Self> = SliceKind(SliceWit::$variant {
                    slice: TypeEq::new(),
                    items: TypeEq::new(),
                    decode_error: TypeEq::new(),
                });
            }
        )*
    };
}

define_slices! {
    /// Just normal slices.
    Slice (<T>) {
        /// This is a normal slice, so the implementation is rather basic.
        slice: [S::Item] = [T],
        /// Just the elements of this slice.
        items: [S::Item] = [T],
        /// Since this is a normal slice, there is zero possible errors
        /// that can occur when "decoding" it from a slice of items.
        ///
        /// They're the same types.
        decode_error: Infallible,
        /// Returns the length of the given slice.
        len: |slice| slice.len(),
        /// This is a no-op since this is a normal slice.
        decode_items: |slice| Ok(slice),
        /// This is always safe to call since this is just
        /// a normal slice.
        decode_items_unchecked: |slice| slice,
        /// This is a no-op since this is a normal slice.
        decode_items_mut: |slice| Ok(slice),
        /// This is always safe to call since this is just
        /// a normal slice.
        decode_items_mut_unchecked: |slice| slice,
        /// Create a raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice: |slice| slice,
        /// Create a mutable raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice_mut: |slice| slice,
        /// Create a [`NonNull`] raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice_nonnull: |slice| slice,
        from_raw_parts: todo!(),
        from_raw_parts_mut: todo!(),
    },

    /// A string slice.
    Str {
        /// We're implementing [`Slice`] on UTF-8 strings.
        slice: str,
        /// A [`str`] is a byte slice with the added invariant that
        /// the bytes are a valid UTF-8 string.
        ///
        /// Not all `[u8]`s are valid `str`s.
        items: [u8],
        /// If the provided byte slice is not UTF-8, this is the error that is
        /// returns.
        decode_error: Utf8Error,
        /// Returns the length of the string in bytes.
        len: |string| (string as *const [u8]).len(),
        /// Create a string slice from a byte slice, given it's valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not UTF-8.
        decode_items: |bytes| core::str::from_utf8(bytes),
        /// Create a string slice from a byte slice without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure that `bytes` is valid UTF-8.
        decode_items_unchecked: |bytes| unsafe { core::str::from_utf8_unchecked(bytes) },
        /// Create a mutable string slice from a mutable byte slice, given it's valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not UTF-8.
        decode_items_mut: |bytes| core::str::from_utf8_mut(bytes),
        /// Create a mutable string slice from a mutable byte slice without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure `bytes` is valid UTF-8.
        decode_items_mut_unchecked: |bytes| unsafe { core::str::from_utf8_unchecked_mut(bytes) },

        /// Create a raw string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice: |slice| slice as *const str,
        /// Create a mutable raw string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice_mut: |slice| slice as *mut str,
        /// Create a [`NonNull`] raw string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, but dereferencing the resulting value is not.
        raw_slice_nonnull: |slice| NonNull::new(slice.as_ptr() as *mut str).unwrap(),
        from_raw_parts: todo!(),
        from_raw_parts_mut: todo!(),
    },
}
