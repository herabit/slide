#![allow(dead_code)]

use crate::{marker::TypeEq, mem::NoDrop, slice::Slice};
use core::{convert::Infallible, ptr::NonNull, str::Utf8Error};

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

macro_rules! define_slices {
    ($(
        $(#[cfg($($cfg:tt)*)])*
        $(#[doc = $doc:expr])*
        unsafe impl $(( $($generics:tt)* ))? Slice
        for $slice:ty $(| $slice_override:ty)? $(|)?
        {
            // The kind of elements this slice contains.
            $(#[$elem_attr:meta])*
            type Elem = $elem:ty $(| $elem_override:ty)? $(|)?;

            // An error that occurs when decoding.
            $(#[$decode_err_attr:meta])*
            type DecodeError = $decode_err:ty $(| $decode_err_override:ty)? $(|)?;

            // An error that occurs when attempting to get the inner elements of a slice.
            $(#[$elem_err_attr:meta])*
            type ElemError = $elem_err:ty $(| $elem_err_override:ty)? $(|)?;

            // An error that occurs when attempting to split or index a slice.
            $(#[$split_err_attr:meta])*
            type SplitError = $split_err:ty $(| $split_err_override:ty)? $(|)?;

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

            // A function to handle element errors in const.
            $(#[$handle_elem_err_attr:meta])*
            const fn handle_elem_error(
                $($handle_elem_err_param:ident: $handle_elem_err_param_ty:ty),+
                $(,)?
            ) -> $handle_elem_err_ret:ty $handle_elem_err_body:block

            // A function to handle split errors in const.
            $(#[$handle_split_err_attr:meta])*
            const fn handle_split_error(
                $($handle_split_err_param:ident: $handle_split_err_param_ty:ty),+
                $(,)?
            ) -> $handle_split_err_ret:ty $handle_split_err_body:block

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

            // A function to decode a slice from its elements.
            $(#[$from_elems_attr:meta])*
            fn from_elems(
                $($from_elems_param:ident: $from_elems_param_ty:ty),+
                $(,)?
            ) -> $from_elems_ret:ty $from_elems_body:block

            // A function to decode a mutable slice from its elements.
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

            // A function that allows one to safely access a slice's elements if it is supported.
            $(#[$as_elems_checked_attr:meta])*
            const fn as_elems_checked(
                $($as_elems_checked_param:ident: $as_elems_checked_param_ty:ty),+
                $(,)?
            ) -> $as_elems_checked_ret:ty $as_elems_checked_body:block

            // A function that allows one to safely mutably access a slice's elements if it is supported.
            $(#[$as_elems_mut_checked_attr:meta])*
            const fn as_elems_mut_checked(
                $($as_elems_mut_checked_param:ident: $as_elems_mut_checked_param_ty:ty),+
                $(,)?
            ) -> $as_elems_mut_checked_ret:ty $as_elems_mut_checked_body:block

            // A function that allows one to unsafely access a slice's elements regardless if it is supported.
            $(#[$as_elems_unchecked_attr:meta])*
            const unsafe fn as_elems_unchecked(
                $($as_elems_unchecked_param:ident: $as_elems_unchecked_param_ty:ty),+
                $(,)?
            ) -> $as_elems_unchecked_ret:ty $as_elems_unchecked_body:block

            // A function that allows one to unsafely mutably access a slice's elements regardless if it is supported.
            $(#[$as_elems_mut_unchecked_attr:meta])*
            const unsafe fn as_elems_mut_unchecked(
                $($as_elems_mut_unchecked_param:ident: $as_elems_mut_unchecked_param_ty:ty),+
                $(,)?
            ) -> $as_elems_mut_unchecked_ret:ty $as_elems_mut_unchecked_body:block
        }
    )*) => {
        /// A wrapper around a [`SliceWit`] that can be exposed publicly.
        #[repr(transparent)]
        pub struct SliceKind<S>(pub(crate) SliceWit<S>)
        where
            S: Slice + ?Sized,
        ;

        impl<S> Clone for SliceKind<S>
        where
            S: Slice + ?Sized,
        {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<S> Copy for SliceKind<S>
        where
            S: Slice + ?Sized,
        {}


        /// A type witness for const polymorphism over types.
        #[non_exhaustive]
        pub(crate) enum SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            $(
                $(#[cfg($($cfg)*)])*
                $(#[$variant_attr])*
                #[non_exhaustive]
                $variant {
                    slice: TypeEq<S, get!($slice | $($slice_override)?)>,
                    elems: TypeEq<[S::Elem], get!([$elem] | $( [$elem_override] )?)>,
                    decode_error: TypeEq<S::DecodeError, get!($decode_err | $($decode_err_override)?)>,
                    elem_error: TypeEq<S::ElemError, get!($elem_err | $($elem_err_override)?)>,
                    split_error: TypeEq<S::SplitError, get!($split_err | $($split_err_override)?)>,
                },
            )*
        }


        impl<S> Clone for SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            #[inline(always)]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<S> Copy for SliceWit<S>
        where
            S: Slice + ?Sized,
        {}

        impl<S> SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn len(
                self,
                slice: *const S,
            ) -> usize {
                 match self {
                     $(
                         $(#[cfg($($cfg)*)])*
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
            pub(crate) const fn raw_slice(
                self,
                data: *const S::Elem,
                len: usize,
            ) -> *const S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice $(< $($generics)* >)? (
                                $($raw_slice_param: $raw_slice_param_ty,)+
                            ) -> $raw_slice_ret $raw_slice_body

                            slice.uncoerce_ptr(
                                raw_slice(elems.unproject().coerce_ptr(data), len)
                            )
                        },
                    )*
                }
            }


            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_mut(
                self,
                data: *mut S::Elem,
                len: usize,
            ) -> *mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice_mut $(< $($generics)* >)? (
                                $($raw_slice_mut_param: $raw_slice_mut_param_ty,)+
                            ) -> $raw_slice_mut_ret $raw_slice_mut_body

                            slice.uncoerce_ptr_mut(
                                raw_slice_mut(elems.unproject().coerce_ptr_mut(data), len)
                            )
                        },
                    )*
                }
            }


            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn raw_slice_nonnull(
                self,
                data: NonNull<S::Elem>,
                len: usize,
            ) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const fn raw_slice_nonnull $(< $($generics)* >)? (
                                $($raw_slice_nonnull_param: $raw_slice_nonnull_param_ty,)+
                            ) -> $raw_slice_nonnull_ret $raw_slice_nonnull_body

                            slice.uncoerce_nonnull(
                                raw_slice_nonnull(elems.unproject().coerce_nonnull(data), len)
                            )
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts<'a>(
                self,
                data: *const S::Elem,
                len: usize,
            ) -> &'a S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const unsafe fn from_raw_parts<$from_raw_parts_lt $(, $($generics)*)? >(
                                $($from_raw_parts_param: $from_raw_parts_param_ty,)+
                            ) -> $from_raw_parts_ret $from_raw_parts_body

                            slice.uncoerce_ref(unsafe {
                                from_raw_parts(elems.unproject().coerce_ptr(data), len)
                            })
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_raw_parts_mut<'a>(
                self,
                data: *mut S::Elem,
                len: usize,
            ) -> &'a mut S {
                match self {
                    $(

                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems,
                            ..
                        } => {
                            $(#[$len_attr])*
                            const unsafe fn from_raw_parts_mut<$from_raw_parts_mut_lt $(, $($generics)*)? >(
                                $($from_raw_parts_mut_param: $from_raw_parts_mut_param_ty,)+
                            ) -> $from_raw_parts_mut_ret $from_raw_parts_mut_body

                            slice.uncoerce_mut(unsafe {
                                from_raw_parts_mut(elems.unproject().coerce_ptr_mut(data), len)
                            })
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            pub(crate) const fn handle_decode_error(
                self,
                decode_error: S::DecodeError,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        #[allow(unreachable_code)]
                        Self::$variant {
                            decode_error: this,
                            ..
                        } => {
                            $(#[$handle_decode_err_attr])*
                            const fn handle_decode_error(
                                $($handle_decode_err_param: $handle_decode_err_param_ty,)+
                            ) -> $handle_decode_err_ret $handle_decode_err_body

                            handle_decode_error(this.coerce(decode_error))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            pub(crate) const fn handle_elem_error(
                self,
                elem_error: S::ElemError,
            ) -> ! {
                match self {
                    $(
                        $(#[$cfg($($cfg)*)])*
                        #[allow(unreachable_code)]
                        Self::$variant {
                            elem_error: this,
                            ..
                        } => {
                            $(#[$handle_elem_err_attr])*
                            const fn handle_elem_error(
                                $($handle_elem_err_param: $handle_elem_err_param_ty,)+
                            ) -> $handle_elem_err_ret $handle_elem_err_body

                            handle_elem_error(this.coerce(elem_error))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            pub(crate) const fn handle_split_error(
                self,
                split_error: S::SplitError,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        #[allow(unreachable_code)]
                        Self::$variant {
                            split_error: this,
                            ..
                        } => {
                            $(#[$handle_split_err_attr])*
                            const fn handle_split_error(
                                $($handle_split_err_param: $handle_split_err_param_ty,)+
                            ) -> $handle_split_err_ret $handle_split_err_body

                            handle_split_error(this.coerce(split_error))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            pub(crate) const fn try_from_elems(
                self,
                elems: &[S::Elem],
            ) -> Result<&S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems: this,
                            decode_error,
                            ..
                        } => {
                            $(#[$try_from_elems_attr])*
                            const fn try_from_elems $(< $($generics)*  >)? (
                                $($try_from_elems_param: $try_from_elems_param_ty,)+
                            ) -> $try_from_elems_ret $try_from_elems_body

                            slice
                                .wrap_ref()
                                .wrap_result(decode_error)
                                .uncoerce(try_from_elems(this.coerce_ref(elems)))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            pub(crate) const fn try_from_elems_mut(
                self,
                elems: &mut [S::Elem],
            ) -> Result<&mut S, S::DecodeError> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems: this,
                            decode_error,
                            ..
                        } => {
                            $(#[$try_from_elems_mut_attr])*
                            const fn try_from_elems_mut $( < $($generics)* > )? (
                                $($try_from_elems_mut_param: $try_from_elems_mut_param_ty,)+
                            ) -> $try_from_elems_mut_ret $try_from_elems_mut_body

                            slice
                                .wrap_mut()
                                .wrap_result(decode_error)
                                .uncoerce(try_from_elems_mut(this.coerce_mut(elems)))
                        }
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn from_elems(
                self,
                elems: &[S::Elem],
            ) -> &S {
                match NoDrop::new(self.try_from_elems(elems)).transpose() {
                    Ok(slice) => slice.into_inner(),
                    #[allow(unreachable_code)]
                    Err(err) => self.handle_decode_error(err.into_inner()),
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const fn from_elems_mut(
                self,
                elems: &mut [S::Elem],
            ) -> &mut S {
                match NoDrop::new(self.try_from_elems_mut(elems)).transpose() {
                    Ok(slice) => slice.into_inner(),
                    Err(err) => self.handle_decode_error(err.into_inner()),
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_elems_unchecked(
                self,
                elems: &[S::Elem],
            ) -> &S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems: this,
                            ..
                        } => {
                            $(#[$from_elems_unchecked_attr])*
                            const unsafe fn from_elems_unchecked $(< $($generics)* >)? (
                                $($from_elems_unchecked_param: $from_elems_unchecked_param_ty,)+
                            ) -> $from_elems_unchecked_ret $from_elems_unchecked_body

                            slice.uncoerce_ref(unsafe {
                                from_elems_unchecked(this.coerce_ref(elems))
                            })
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub(crate) const unsafe fn from_elems_mut_unchecked(
                self,
                elems: &mut [S::Elem],
            ) -> &mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice,
                            elems: this,
                            ..
                        } => {
                            $(#[$from_elems_mut_unchecked_attr])*
                            const unsafe fn from_elems_mut_unchecked $(< $($generics)* >)? (
                                $($from_elems_mut_unchecked_param: $from_elems_mut_unchecked_param_ty,)+
                            ) -> $from_elems_mut_unchecked_ret $from_elems_mut_unchecked_body

                            slice.uncoerce_mut(unsafe {
                                from_elems_mut_unchecked(this.coerce_mut(elems))
                            })
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            pub(crate) const fn as_elems_checked(
                self,
                slice: &S,
            ) -> Option<&[S::Elem]> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice: this,
                            elems,
                            ..
                        } => {
                            $(#[$as_elems_checked_attr])*
                            const fn as_elems_checked $(< $($generics)* >)? (
                                $($as_elems_checked_param: $as_elems_checked_param_ty,)+
                            ) -> $as_elems_checked_ret $as_elems_checked_body

                            elems
                                .wrap_ref()
                                .wrap_option()
                                .uncoerce(as_elems_checked(this.coerce_ref(slice)))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            pub(crate) const fn as_elems_mut_checked(
                self,
                slice: &mut S,
            ) -> Option<&mut [S::Elem]>
            {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice: this,
                            elems,
                            ..
                        } => {
                            $(#[$as_elems_mut_checked_attr])*
                            const fn as_elems_mut_checked $(< $($generics)* >)? (
                                $($as_elems_mut_checked_param: $as_elems_mut_checked_param_ty,)+
                            ) -> $as_elems_mut_checked_ret $as_elems_mut_checked_body

                            elems
                                .wrap_mut()
                                .wrap_option()
                                .uncoerce(as_elems_mut_checked(this.coerce_mut(slice)))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            pub(crate) const unsafe fn as_elems_unchecked(
                self,
                slice: &S,
            ) -> &[S::Elem]
            {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice: this,
                            elems,
                            ..
                        } => {
                            $(#[$as_elems_unchecked_attr])*
                            const unsafe fn as_elems_unchecked $(< $($generics)* >)? (
                                $($as_elems_unchecked_param: $as_elems_unchecked_param_ty,)+
                            ) -> $as_elems_unchecked_ret $as_elems_unchecked_body

                            elems.uncoerce_ref(unsafe {
                                as_elems_unchecked(this.coerce_ref(slice))
                            })
                        },
                    )*
                }
            }


            #[inline(always)]
            #[must_use]
            pub(crate) const unsafe fn as_elems_mut_unchecked(
                self,
                slice: &mut S,
            ) -> &mut [S::Elem]
            {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice: this,
                            elems,
                            ..
                        } => {
                            $(#[$as_elems_mut_unchecked_attr])*
                            const unsafe fn as_elems_mut_unchecked $(< $($generics)* >)? (
                                $($as_elems_mut_unchecked_param: $as_elems_mut_unchecked_param_ty,)+
                            ) -> $as_elems_mut_unchecked_ret $as_elems_mut_unchecked_body

                            elems.uncoerce_mut(unsafe {
                                as_elems_mut_unchecked(this.coerce_mut(slice))
                            })
                        },
                    )*
                }
            }
        }

        $(
            $(#[cfg($($cfg)*)])*
            impl $(< $($generics)* >)? Sealed for $slice {}

            $(#[cfg($($cfg)*)])*
            $(#[doc = $doc])*
            unsafe impl $(< $($generics)* >)? Slice for $slice {
                $(#[$elem_attr])*
                type Elem = $elem;

                $(#[$decode_err_attr])*
                type DecodeError = $decode_err;

                $(#[$elem_err_attr])*
                type ElemError = $elem_err;

                $(#[$split_err_attr])*
                type SplitError = $split_err;

                const KIND: SliceKind<Self> = SliceKind(SliceWit::$variant {
                    slice: TypeEq::new(),
                    elems: TypeEq::new(),
                    decode_error: TypeEq::new(),
                    elem_error: TypeEq::new(),
                    split_error: TypeEq::new(),
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
        /// It is impossible for `[T] -> [T]` to fail.
        type ElemError = Infallible;
        /// There are no additional invariants to check for a normal slice besides
        /// indicies being within bounds.
        type SplitError = Infallible;
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
        const fn handle_decode_error(err: Infallible) -> ! {
            match err {}
        }

        /// This is impossible to call, as it is impossible for getting the inner
        /// elements of a normal slice to fail.
        #[inline(always)]
        const fn handle_elem_error(err: Infallible) -> ! {
            match err {}
        }

        /// This is impossible to call, reasons are ***TODO***.
        #[inline(always)]
        const fn handle_split_error(err: Infallible) -> ! {
            match err {}
        }

        /// This just returns `Ok(elems)`.
        ///
        /// # Returns
        ///
        /// This never returns an error.
        #[inline(always)]
        const fn try_from_elems(elems: &[T]) -> Result<&[T], Infallible> {
            Ok(elems)
        }

        /// This just returns `Ok(elems)`.
        ///
        /// # Returns
        ///
        /// This never returns an error.
        #[inline(always)]
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

        /// This just returns this slice.
        ///
        /// # Returns
        ///
        /// This always returns [`Some`].
        #[inline(always)]
        #[must_use]
        const fn as_elems_checked(slice: &[T]) -> Option<&[T]> {
            Some(slice)
        }

        /// This just returns this slice.
        ///
        /// # Returns
        ///
        /// This always returns [`Some`].
        #[inline(always)]
        #[must_use]
        const fn as_elems_mut_checked(slice: &mut [T]) -> Option<&mut [T]> {
            Some(slice)
        }

        /// This just returns this slice.
        ///
        /// # Safety
        ///
        /// This is always safe.
        #[inline(always)]
        #[must_use]
        const unsafe fn as_elems_unchecked(slice: &[T]) -> &[T] {
            slice
        }

        /// This just returns this slice.
        ///
        /// # Safety
        ///
        /// This is always safe.
        #[inline(always)]
        #[must_use]
        const unsafe fn as_elems_mut_unchecked(slice: &mut [T]) -> &mut [T] {
            slice
        }
    }

    unsafe impl Slice for str {
        /// A [`str`] is a byte slice that is always UTF-8.
        type Elem = u8;
        /// Error returned when a `[u8]` is not valid UTF-8.
        type DecodeError = Utf8Error;
        /// Error that occurs when attempting to safely get a `&mut [u8]` from a `&mut str`.
        type ElemError = &'static str;
        /// Error that occurs when attempting to split a [`str`] at some point that is not
        /// a UTF-8 character boundary.
        type SplitError = &'static str;

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
        #[track_caller]
        const fn handle_decode_error(err: Utf8Error) -> ! {
            let _ = err;
            panic!("invalid utf-8")
        }

        /// ***TODO***
        #[inline(always)]
        #[track_caller]
        const fn handle_elem_error(err: &'static str) -> ! {
            panic!("{}", err)
        }

        /// ***TODO***
        #[inline(always)]
        #[track_caller]
        const fn handle_split_error(err: &'static str) -> ! {
            panic!("{}", err)
        }

        /// Create a string slice from `bytes` if `bytes`
        /// is valid UTF-8.
        ///
        /// # Returns
        ///
        /// Returns an error if `bytes` is not valid UTF-8.
        #[inline(always)]
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

        /// Borrow the underlying bytes of this string slice.
        ///
        /// # Returns
        ///
        /// This always returns [`Some`].
        #[inline(always)]
        #[must_use]
        const fn as_elems_checked(slice: &str) -> Option<&[u8]> {
            Some(slice.as_bytes())
        }

        /// Mutably borrow the underlying bytes of this string slice.
        ///
        /// # Returns
        ///
        /// This always returns [`None`].
        ///
        /// This is because there is no safe way of mutably getting
        /// the underlying bytes of a [`str`] without the possibility
        /// of the caller using it in a manner that results in it being
        /// invalid UTF-8 before the borrow ends.
        ///
        /// See [`str::as_bytes_mut`] for more info. If you need to use
        /// the [`Slice`] API, use [`Slice::as_elems_mut_unchecked`].
        #[inline(always)]
        #[must_use]
        const fn as_elems_mut_checked(slice: &mut str) -> Option<&mut [u8]> {
            let _ = slice;
            None
        }

        /// Borrow the underlying bytes of this string slice.
        ///
        /// # Safety
        ///
        /// This is always safe to call.
        #[inline(always)]
        #[must_use]
        const unsafe fn as_elems_unchecked(slice: &str) -> &[u8] {
            slice.as_bytes()
        }

        /// Mutably borrow the underlying bytes of this string slice.
        ///
        /// # Safety
        ///
        /// When using this, the caller has to be really careful when
        /// mutating the data stored within the byte slice.
        ///
        /// If it is invalid UTF-8 when the borrow ends, then you're
        /// left with a [`str`] containing invalid UTF-8, which is
        /// always undefined behavior.
        ///
        /// As such, use this sparringly. See [`str::as_bytes_mut`]
        /// for more info on the topic.
        #[inline(always)]
        #[must_use]
        const unsafe fn as_elems_mut_unchecked(slice: &mut str) -> &mut [u8] {
            // SAFETY: The caller ensures that when the borrow ends,
            //         the string is still valid UTF-8.
            unsafe { slice.as_bytes_mut() }
        }
    }
}
