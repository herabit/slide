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

            // The slice elems type.
            $(#[doc = $elem_doc:expr])*
            elems: [$elem_ty:ty] $(= [$impl_elem_ty:ty])?,

            // The slice decode error type.
            $(#[doc = $decode_error_doc:expr])*
            decode_error: $decode_error_ty:ty $(= $impl_decode_error_ty:ty)?,

            // The slice length function.
            $(#[doc = $len_doc:expr])*
            len: |$len_params:pat_param| $len_body:expr,

            // The decode elems function.
            $(#[doc = $decode_elems_doc:expr])*
            decode_elems: |$decode_elems_params:pat_param| $decode_elems_body:expr,

            // The decode elems unchecked function.
            $(#[doc = $decode_elems_unchecked_doc:expr])*
            decode_elems_unchecked:
                |$decode_elems_unchecked_params:pat_param| $decode_elems_unchecked_body:expr,

            // The decode mutable elems function.
            $(#[doc = $decode_elems_mut_doc:expr])*
            decode_elems_mut:
                |$decode_elems_mut_params:pat_param| $decode_elems_mut_body:expr,

            // The decode mutable elems unchecked function.
            $(#[doc = $decode_elems_mut_unchecked_doc:expr])*
            decode_elems_mut_unchecked:
                |$decode_elems_mut_unchecked_params:pat_param| $decode_elems_mut_unchecked_body:expr,

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
            from_raw_parts: |$from_raw_parts_params:pat_param| $from_raw_parts_body:expr,

            // The from raw parts mut function.
            $(#[doc = $from_raw_parts_mut_doc:expr])*
            from_raw_parts_mut: |$from_raw_parts_mut_params:pat_param| $from_raw_parts_mut_body:expr,

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
                    elems: TypeEq<[S::Elem], [$elem_ty]>,
                    decode_error: TypeEq<S::DecodeError, $decode_error_ty>,
                },
            )*
        }

        impl<S: Slice + ?Sized> SliceWit<S> {
            /// Returns the length of a given slice.
            #[inline(always)]
            #[must_use]
            #[track_caller]
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

            /// Decode a slice of elems into this slice type.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn decode_elems(self, elems: &[S::Elem]) -> Result<&S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            decode_error,
                            elems: this,
                        } => {
                            let result = slice.wrap_ref().wrap_result(decode_error);
                            let $decode_elems_params = this.coerce_ref(elems);

                            result.uncoerce($decode_elems_body)
                        },
                    )*
                }
            }

            /// Decode a slice of elems into this slice type without any checks.
            ///
            /// # Safety
            ///
            /// The caller must ensure that it is safe to construct an `&S` from the provided &[S::Elem]`.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn decode_elems_unchecked(self, elems: &[S::Elem]) -> &S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems: this,
                            ..
                        } => {
                            let $decode_elems_unchecked_params = this.coerce_ref(elems);

                            slice.uncoerce_ref($decode_elems_unchecked_body)
                        },
                    )*
                }
            }

            /// Decode a mutable slice of elems into this slice type.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn decode_elems_mut(self, elems: &mut [S::Elem]) -> Result<&mut S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            decode_error,
                            elems: this,
                        } => {
                            let result = slice.wrap_mut().wrap_result(decode_error);
                            let $decode_elems_mut_params = this.coerce_mut(elems);

                            result.uncoerce($decode_elems_mut_body)
                        },
                    )*
                }
            }

            /// Decode a mutable slice of elems into this slice type without any checks.
            ///
            /// # Safety
            ///
            /// The caller must ensure that it is safe to construct an `&mut S` from the provided
            /// `&mut [S::Elem]`.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn decode_elems_mut_unchecked(self, elems: &mut [S::Elem]) -> &mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems: this,
                            ..
                        } => {
                            let $decode_elems_mut_unchecked_params = this.coerce_mut(elems);

                            slice.uncoerce_mut($decode_elems_mut_unchecked_body)
                        },
                    )*
                }
            }

            /// Create a raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Elem`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice(self, data: *const S::Elem, len: usize) -> *const S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            let $raw_slice_params = elems.coerce_ptr(
                                ptr::slice_from_raw_parts(data, len),
                            );

                            slice.uncoerce_ptr($raw_slice_body)
                        },
                    )*
                }
            }

            /// Create a mutable raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Elem`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_mut(self, data: *mut S::Elem, len: usize) -> *mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            let $raw_slice_mut_params = elems.coerce_ptr_mut(
                                ptr::slice_from_raw_parts_mut(data, len),
                            );

                            slice.uncoerce_ptr_mut($raw_slice_mut_body)
                        },
                    )*
                }
            }

            /// Create a [`NonNull`] raw slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Elem`s the slice contains.
            ///
            /// # Safety
            ///
            /// This is always safe, but dereferencing the resulting value is not.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_nonnull(self, data: NonNull<S::Elem>, len: usize) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            let $raw_slice_nonnull_params = elems.coerce_nonnull(
                                NonNull::slice_from_raw_parts(data, len),
                            );

                            slice.uncoerce_nonnull($raw_slice_nonnull_body)
                        },
                    )*
                }
            }

            /// Create a new slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Elem`s the slice contains.
            ///
            /// # Safety
            ///
            /// The caller must ensure that the provided pointer and length can make
            /// a valid `&'a [Self::Elem]` according to [`core::slice::from_raw_parts`].
            ///
            /// Additionally the caller must ensure that the `&'a [Self::Elem]` created is a
            /// valid `&'a S`.
            ///
            /// Failure to ensure the above is undefined behavior.
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts<'a>(self, data: *const S::Elem, len: usize) -> &'a S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            let $from_raw_parts_params = (
                                elems.unproject().coerce_ptr(data),
                                len,
                            );

                            slice.uncoerce_ref($from_raw_parts_body)
                        },
                    )*
                }
            }

            /// Create a new mutable slice from a pointer and a length.
            ///
            /// The length is the amount of `Slice::Elem`s the slice contains.
            ///
            /// # Safety
            ///
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts_mut<'a>(self, data: *mut S::Elem, len: usize) -> &'a mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            let $from_raw_parts_mut_params = (
                                elems.unproject().coerce_ptr_mut(data),
                                len,
                            );

                            slice.uncoerce_mut($from_raw_parts_mut_body)
                        },
                    )*
                }
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
                $(#[doc = $elem_doc])*
                type Elem = get_impl!($elem_ty $(= $impl_elem_ty)?);

                $(#[doc = $decode_error_doc])*
                type DecodeError = get_impl!($decode_error_ty $(= $impl_decode_error_ty)?);

                const KIND: SliceKind<Self> = SliceKind(SliceWit::$variant {
                    slice: TypeEq::new(),
                    elems: TypeEq::new(),
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
        slice: [S::Elem] = [T],
        /// Just the elements of this slice.
        elems: [S::Elem] = [T],
        /// Since this is a normal slice, there is zero possible errors
        /// that can occur when "decoding" it from a slice of elems.
        ///
        /// They're the same types.
        decode_error: Infallible,
        /// Returns the length of the given slice.
        len: |slice| slice.len(),
        /// This is a no-op since this is a normal slice.
        decode_elems: |slice| Ok(slice),
        /// This is always safe to call since this is just
        /// a normal slice.
        decode_elems_unchecked: |slice| slice,
        /// This is a no-op since this is a normal slice.
        decode_elems_mut: |slice| Ok(slice),
        /// This is always safe to call since this is just
        /// a normal slice.
        decode_elems_mut_unchecked: |slice| slice,
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
        /// Create a new slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// The caller must ensure that all of the conditions for [`core::slice::from_raw_parts`]
        /// are upheld. Failure to do so is undefined behavior.
        from_raw_parts: |(data, len)| unsafe { core::slice::from_raw_parts(data, len) },
        /// Create a new mutable slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// The caller must ensure that all of the conditions for [`core::slice::from_raw_parts_mut`]
        /// are upheld. Failure to do so is undefined behavior.
        from_raw_parts_mut: |(data, len)| unsafe { core::slice::from_raw_parts_mut(data, len) },
    },

    /// A string slice.
    Str {
        /// We're implementing [`Slice`] on UTF-8 strings.
        slice: str,
        /// A [`str`] is a byte slice with the added invariant that
        /// the bytes are a valid UTF-8 string.
        ///
        /// Not all `[u8]`s are valid `str`s.
        elems: [u8],
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
        decode_elems: |bytes| core::str::from_utf8(bytes),
        /// Create a string slice from a byte slice without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure that `bytes` is valid UTF-8.
        decode_elems_unchecked: |bytes| unsafe { core::str::from_utf8_unchecked(bytes) },
        /// Create a mutable string slice from a mutable byte slice, given it's valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not UTF-8.
        decode_elems_mut: |bytes| core::str::from_utf8_mut(bytes),
        /// Create a mutable string slice from a mutable byte slice without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure `bytes` is valid UTF-8.
        decode_elems_mut_unchecked: |bytes| unsafe { core::str::from_utf8_unchecked_mut(bytes) },

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
        /// Create a new string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior if:
        ///
        /// - If any of the conditions for [`core::slice::from_raw_parts`] are not met
        ///   for `&'a [u8]`.
        ///
        /// - If the resulting `&'a [u8]` is not valid UTF-8.
        from_raw_parts: |(data, len)| {
            // SAFETY: The caller ensures this is safe.
            let bytes = unsafe { core::slice::from_raw_parts(data, len) };

            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked(bytes) }
        },
        /// Create a new string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior if:
        ///
        /// - Any of the conditions for [`core::slice::from_raw_parts_mut`] are not met
        ///   for `&'a mut [u8]`.
        ///
        /// - If the resulting `&'a mut [u8]` is not valid UTF-8.
        from_raw_parts_mut: |(data, len)| {
            // SAFETY: The caller ensures this is safe.
            let bytes = unsafe { core::slice::from_raw_parts_mut(data, len) };

            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked_mut(bytes) }
        },
    },
}
