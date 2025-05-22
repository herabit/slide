#![allow(dead_code)]

use core::{
    convert::Infallible,
    ptr::{self, NonNull},
    str::Utf8Error,
};

use crate::{marker::TypeEq, slice::Slice};

/// Trait to seal what we consider to be slices.
pub trait Sealed {}

macro_rules! get {
    ($a:ty $(|)?) => {
        $a
    };
    ($a:ty | $b:ty $(|)?) => {
        $b
    };
}

// macro_rules! const_method {
//     (
//         $(#[$attr:meta])*
//         $(const $($const:lifetime)?)?
//         $(unsafe $($unsafe:lifetime)?)?
//         fn $name:ident $([ $($generics:tt)* ])? ($(
//             $arg:ident: $arg_ty:ty $(| $arg_override:ty)? $(|)?
//         ),* $(,)?) $(->
//             $ret:ty $(| $ret_override:ty)? $(|)?
//         )? $body:block
//     ) => {
//         $(#[$attr])*
//         $(const $($const)?)?
//         $(unsafe $($unsafe)?)?
//         fn $name $(< $($generics)* >)? (
//             $(
//                 $arg: get!($arg_ty $(| $arg_override)?)
//             ),*
//         ) $(-> )? $block
//     };
// }

macro_rules! define_slices {
    ($(
        $(#[cfg($cfg:meta)])*
        $(#[doc = $doc:expr])*
        unsafe impl $(( $($generics:tt)* ))? Slice for $slice:ty $(| $slice_override:ty)? $(|)?
        {
            // The kind of elements this slice contains.
            $(#[$elem_attr:meta])*
            type Elem = $elem:ty $(| $elem_override:ty)? $(|)?;

            // An error that occurs when decoding.
            $(#[$decode_err_attr:meta])*
            type DecodeError = $decode_err:ty $(| $decode_err_override:ty)? $(|)?;

            // The variant name for the type witness.
            $(#[$variant_attr:meta])*
            type Variant = $variant:ident;

            // A function that returns the amount of elements in a slice.
            $(#[$len_attr:meta])*
            const fn len(
                $($len_param:ident: $len_param_ty:ty $(| $len_param_override:ty)? ),+
                $(,)?
            ) -> $len_ret:ty $len_body:block

            // A function to create a raw slice from a pointer and a length.
            $(#[$raw_slice_attr:meta])*
            const fn raw_slice(
                $($raw_slice_param:ident: $raw_slice_param_ty:ty),+
                $(,)?
            ) -> $raw_slice_ret:ty $raw_slice_body:block

            // A function to create a mutable raw slice from a pointer and a length.
            $(#[$raw_slice_mut_attr:meta])*
            const fn raw_slice_mut(
                $($raw_slice_mut_param:ident: $raw_slice_mut_param_ty:ty),+
                $(,)?
            ) -> $raw_slice_mut_ret:ty $raw_slice_mut_body:block

            // A function to create a non-null raw slice from a pointer and a length.
            $(#[$raw_slice_nonnull_attr:meta])*
            const fn raw_slice_nonnull(
                $($raw_slice_nonnull_param:ident: $raw_slice_nonnull_param_ty:ty),+
                $(,)?
            ) -> $raw_slice_nonnull_ret:ty $raw_slice_nonnull_body:block

            // A function to create a slice from a pointer and a length.
            $(#[$from_raw_parts_attr:meta])*
            const unsafe fn from_raw_parts<$from_raw_parts_lt:lifetime>(
                $($from_raw_parts_param:ident: $from_raw_parts_param_ty:ty),+
                $(,)?
            ) -> $from_raw_parts_ret:ty $from_raw_parts_body:block

            // A function to create a mutable slice from a pointer and a length.
            $(#[$from_raw_parts_mut_attr:meta])*
            const unsafe fn from_raw_parts_mut<$from_raw_parts_mut_lt:lifetime>(
                $($from_raw_parts_mut_param:ident: $from_raw_parts_mut_param_ty:ty),+
                $(,)?
            ) -> $from_raw_parts_mut_ret:ty $from_raw_parts_mut_body:block

            // A function to handle decoding errors in const.
            $(#[$handle_decode_err_attr:meta])*
            const fn handle_decode_error(
                $($handle_decode_err_param:ident: $handle_decode_err_param_ty:ty),+
                $(,)?
            ) -> $handle_decode_err_ret:ty $handle_decode_err_body:block

            // A function to attempt to decode a slice from its elements.
            $(#[$try_from_elems_attr:meta])*
            const fn try_from_elems(
                $($try_from_elems_param:ident: $try_from_elems_param_ty:ty),+
                $(,)?
            ) -> $try_from_elems_ret:ty $try_from_elems_body:block

            // A function to attempt to decode a mutable slice from its elements.
            $(#[$try_from_elems_mut_attr:meta])*
            const fn try_from_elems_mut(
                $($try_from_elems_mut_param:ident: $try_from_elems_mut_param_ty:ty),+
                $(,)?
            ) -> $try_from_elems_mut_ret:ty $try_from_elems_mut_body:block

            // A function to decode a slice from its elements, panicking with a more
            // detailed error than the const version.
            $(#[$from_elems_attr:meta])*
            fn from_elems(
                $($from_elems_param:ident: $from_elems_param_ty:ty),+
                $(,)?
            ) -> $from_elems_ret:ty $from_elems_body:block

            // A function to decode a mutable slice from its elements, panicking with a more
            // detailed error than the const version.
            $(#[$from_elems_mut_attr:meta])*
            fn from_elems_mut(
                $($from_elems_mut_param:ident: $from_elems_mut_param_ty:ty),+
                $(,)?
            ) -> $from_elems_mut_ret:ty $from_elems_mut_body:block

            // A function to decode a slice from its elements without any checks.
            $(#[$from_elems_unchecked_attr:meta])*
            const unsafe fn from_elems_unchecked(
                $($from_elems_unchecked_param:ident: $from_elems_unchecked_param_ty:ty),+
                $(,)?
            ) -> $from_elems_unchecked_ret:ty $from_elems_unchecked_body:block

            // A function to decode a mutable slice from its elements without any checks.
            $(#[$from_elems_mut_unchecked_attr:meta])*
            const unsafe fn from_elems_mut_unchecked(
                $($from_elems_mut_unchecked_param:ident: $from_elems_mut_unchecked_param_ty:ty),+
                $(,)?
            ) -> $from_elems_mut_unchecked_ret:ty $from_elems_mut_unchecked_body:block
        }
    )*) => {
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


        /// A type witness for const polymorphism over types.
        #[non_exhaustive]
        pub(crate) enum SliceWit<S: Slice + ?Sized> {
            $(
                $(#[cfg($cfg)])*
                $(#[$variant_attr])*
                #[non_exhaustive]
                $variant {
                    slice: TypeEq<S, get!($slice | $($slice_override)?)>,
                    elems: TypeEq<[S::Elem], get!([$elem] | $( [$elem_override] )?)>,
                    decode_error: TypeEq<S::DecodeError, get!($decode_err | $($decode_err_override)?)>,
                },
            )*
        }

        impl<S: Slice + ?Sized> SliceWit<S> {
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn len(self, slice: *const S) -> usize {
                 match self {
                     $(
                         $(#[cfg($cfg)])*
                         Self::$variant {
                             slice: this,
                             ..
                         } => {
                             $(#[$len_attr])*
                             const fn len $(< $($generics)* >)? (
                                 $($len_param: $len_param_ty,)+
                             ) -> $len_ret $len_body

                             len(this.coerce_ptr(slice))
                         },
                     )*
                 }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice(self, data: *const S::Elem, len: usize) -> *const S {
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice $(< $($generics)* >)? (
                                $($raw_slice_param: $raw_slice_param_ty,)+
                            ) -> $raw_slice_ret $raw_slice_body

                            slice.uncoerce_ptr(raw_slice(elems.unproject().coerce_ptr(data), len))
                        },
                    )*
                }
            }


            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_mut(self, data: *mut S::Elem, len: usize) -> *mut S {
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice_mut $(< $($generics)* >)? (
                                $($raw_slice_mut_param: $raw_slice_mut_param_ty,)+
                            ) -> $raw_slice_mut_ret $raw_slice_mut_body

                            slice.uncoerce_ptr_mut(raw_slice_mut(elems.unproject().coerce_ptr_mut(data), len))
                        },
                    )*
                }
            }


            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_nonnull(self, data: NonNull<S::Elem>, len: usize) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice_nonnull $(< $($generics)* >)? (
                                $($raw_slice_nonnull_param: $raw_slice_nonnull_param_ty,)+
                            ) -> $raw_slice_nonnull_ret $raw_slice_nonnull_body

                            slice.uncoerce_nonnull(raw_slice_nonnull(elems.unproject().coerce_nonnull(data), len))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts<'a>(self, data: *const S::Elem, len: usize) -> &'a S {
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const unsafe fn from_raw_parts<$from_raw_parts_lt $(, $($generics)*)? >(
                                $($from_raw_parts_param: $from_raw_parts_param_ty,)+
                            ) -> $from_raw_parts_ret $from_raw_parts_body

                            slice.uncoerce_ref(
                                // SAFETY: The caller ensures this is safe.
                                unsafe { from_raw_parts(elems.unproject().coerce_ptr(data), len) }
                            )
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

        $(
            $(#[cfg($cfg)])*
            impl $(< $($generics)* >)? Sealed for $slice {}

            $(#[cfg($cfg)])*
            $(#[doc = $doc])*
            unsafe impl $(< $($generics)* >)? Slice for $slice {
                $(#[$elem_attr])*
                type Elem = $elem;

                $(#[$decode_err_attr])*
                type DecodeError = $decode_err;

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
    unsafe impl (T) Slice for [T] | [S::Elem] {
        /// A normal slice is just a slice over it's elements.
        type Elem = T | S::Elem;
        /// It is impossible for `[T] -> [T]` to fail.
        type DecodeError = Infallible;
        /// Just a normal slice.
        type Variant = Slice;

        /// Returns the amount of elements within this slice.
        #[inline(always)]
        #[must_use]
        const fn len(slice: *const [T]) -> usize {
            slice.len()
        }

        /// Create a raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice(data: *const T, len: usize) -> *const [T] {
            core::ptr::slice_from_raw_parts(data, len)
        }

        /// Create a mutable raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice_mut(data: *mut T, len: usize) -> *mut [T] {
            core::ptr::slice_from_raw_parts_mut(data, len)
        }

        /// Create a [`NonNull`] raw slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice_nonnull(data: NonNull<T>, len: usize) -> NonNull<[T]> {
            NonNull::slice_from_raw_parts(data, len)
        }

        /// Create a slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior for any of the preconditions of
        /// [`core::slice::from_raw_parts`] to be violated.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_raw_parts<'a>(data: *const T, len: usize) -> &'a [T] {
            // SAFETY: The caller ensures this is safe.
            unsafe { core::slice::from_raw_parts(data, len) }
        }

        /// Create a mutable slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior for any of the preconditions of
        /// [`core::slice::from_raw_parts_mut`] to be violated.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_raw_parts_mut<'a>(data: *mut T, len: usize) -> &'a mut [T] {
            // SAFETY The caller ensures this is safe.
            unsafe { core::slice::from_raw_parts_mut(data, len) }
        }

        /// This is impossible to call, as it is impossible for decoding
        /// of a normal slice to fail.
        #[inline(always)]
        #[must_use]
        #[allow(unused_variables, unreachable_code)]
        const fn handle_decode_error(err: Infallible) -> ! {
            match err {}
        }

        /// This just returns `Ok(elems)`.
        ///
        /// # Returns
        ///
        /// This never returns an error.
        #[inline(always)]
        #[must_use]
        const fn try_from_elems(elems: &[T]) -> Result<&[T], Infallible> {
            Ok(elems)
        }

        /// This just returns `Ok(elems)`.
        ///
        /// # Returns
        ///
        /// This never returns an error.
        #[inline(always)]
        #[must_use]
        const fn try_from_elems_mut(elems: &mut [T]) -> Result<&mut [T], Infallible> {
            Ok(elems)
        }

        /// This just returns `elems`.
        ///
        /// # Panics
        ///
        /// This never panics.
        #[inline(always)]
        #[must_use]
        fn from_elems(elems: &[T]) -> &[T] {
            elems
        }

        /// This just returns `elems`.
        ///
        /// # Panics
        ///
        /// This never panics.
        #[inline(always)]
        #[must_use]
        fn from_elems_mut(elems: &mut [T]) -> &mut [T] {
            elems
        }

        /// This just returns `elems`.
        ///
        /// # Safety
        ///
        /// This is always safe to call.
        #[inline(always)]
        #[must_use]
        const unsafe fn from_elems_unchecked(elems: &[T]) -> &[T] {
            elems
        }

        /// This just returns `elems`.
        ///
        /// # Safety
        ///
        /// This is always safe to call.
        #[inline(always)]
        #[must_use]
        const unsafe fn from_elems_mut_unchecked(elems: &mut [T]) -> &mut [T] {
            elems
        }
    }

    unsafe impl Slice for str {
        /// A [`str`] is a byte slice that is always UTF-8.
        type Elem = u8;
        /// Error returned when a `[u8]` is not valid UTF-8.
        type DecodeError = Utf8Error;
        /// String slice.
        type Variant = Str;

        /// Returns the amount of bytes within the slice.
        #[inline(always)]
        #[must_use]
        const fn len(slice: *const str) -> usize {
            (slice as *const [u8]).len()
        }

        /// Create a raw string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice(data: *const u8, len: usize) -> *const str {
            core::ptr::slice_from_raw_parts(data, len) as *const str
        }

        /// Create a mutable raw string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice_mut(data: *mut u8, len: usize) -> *mut str {
            core::ptr::slice_from_raw_parts_mut(data, len) as *mut str
        }

        /// Create a [`NonNull`] string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// This is always safe to call, dereferencing the resulting
        /// value is not.
        #[inline(always)]
        #[must_use]
        const fn raw_slice_nonnull(data: NonNull<u8>, len: usize) -> NonNull<str> {
            // SAFETY: We know `data` is non-null.
            unsafe {
                NonNull::new_unchecked(
                    core::ptr::slice_from_raw_parts_mut(data.as_ptr(), len) as *mut str
                )
            }
        }

        /// Create a string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior for any of the preconditions of
        /// [`core::slice::from_raw_parts`] to be violated.
        ///
        /// It is undefined behavior for the resulting byte slice
        /// to be invalid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_raw_parts<'a>(data: *const u8, len: usize) -> &'a str {
            // SAFETY: The caller ensures `data` and `len` make a valid `&'a [u8]`.
            let bytes = unsafe { core::slice::from_raw_parts(data, len) };

            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked(bytes) }
        }

        /// Create a mutable string slice from a pointer and a length.
        ///
        /// # Safety
        ///
        /// It is undefined behavior for any of the preconditions of
        /// [`core::slice::from_raw_parts_mut`] to be violated.
        ///
        /// It is undefined behavior for the resulting byte slice
        /// to be invalid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_raw_parts_mut<'a>(data: *mut u8, len: usize) -> &'a mut str {
            // SAFETY: The caller ensures `data` and `len` make a valid `&'a mut [u8]`.
            let bytes = unsafe { core::slice::from_raw_parts_mut(data, len) };

            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked_mut(bytes) }
        }

        /// Panics with a minimal error about the string being invalid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const fn handle_decode_error(err: Utf8Error) -> ! {
            let _ = err;
            panic!("invalid utf-8")
        }

        /// Create a string slice from `bytes` if `bytes`
        /// is valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not valid UTF-8.
        #[inline(always)]
        #[must_use]
        const fn try_from_elems(bytes: &[u8]) -> Result<&str, Utf8Error> {
            core::str::from_utf8(bytes)
        }

        /// Create a mutable string slice from `bytes` if `bytes`
        /// is valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not valid UTF-8.
        #[inline(always)]
        #[must_use]
        const fn try_from_elems_mut(bytes: &mut [u8]) -> Result<&mut str, Utf8Error> {
            core::str::from_utf8_mut(bytes)
        }

        /// Create a string slice from `bytes`.
        ///
        /// # Panics
        ///
        /// Panics if `bytes` is not valid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        fn from_elems(bytes: &[u8]) -> &str {
            match core::str::from_utf8(bytes) {
                Ok(s) => s,
                Err(err) => panic!("{err}"),
            }
        }

        /// Create a mutable string slice from `bytes`.
        ///
        /// # Panics
        ///
        /// Panics if `bytes` is not valid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        fn from_elems_mut(bytes: &mut [u8]) -> &mut str {
            match core::str::from_utf8_mut(bytes) {
                Ok(s) => s,
                Err(err) => panic!("{err}"),
            }
        }

        /// Create a string slice from `bytes` without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure `bytes` is valid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_elems_unchecked(bytes: &[u8]) -> &str {
            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked(bytes) }
        }

        /// Create a mutable string slice from `bytes` without any checks.
        ///
        /// # Safety
        ///
        /// The caller must ensure `bytes` is valid UTF-8.
        #[inline(always)]
        #[must_use]
        #[track_caller]
        const unsafe fn from_elems_mut_unchecked(bytes: &mut [u8]) -> &mut str {
            // SAFETY: The caller ensures `bytes` is valid UTF-8.
            unsafe { core::str::from_utf8_unchecked_mut(bytes) }
        }
    }
}
