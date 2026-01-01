#![allow(clippy::empty_docs)]

use crate::{marker::TypeEq, slice::private::SliceKind};
use core::{convert::Infallible, fmt, ptr::NonNull, str::Utf8Error};

/// Internal implementation details.
pub(crate) mod private;

/// Error types.
pub(crate) mod error;
#[doc(inline)]
pub use error::*;

/// Extracts documentation and creates a string from it.
macro_rules! extract_docs {
    () => { "" };
    (#[doc = $doc:expr] $($rest:tt)*) => {
        ::core::concat!(
            $doc,
            "\n",
            $crate::slice::extract_docs!($($rest)*),
        )
    };
    (#[$meta:meta] $($rest:tt)*) => { $crate::slice::extract_docs!($($rest)*) };
}

#[allow(unused_imports)]
pub(crate) use extract_docs;

/// Generates a set of functions (usually methods, hence the name) with an associated `docs` macro that hands out
/// a string containing a given function's documentation.
macro_rules! methods {
    (
        $(
            $(#[$($meta:tt)*])*
            $vis:vis
                $(const $($const:lifetime)?)?
                $(unsafe $($unsafe:lifetime)?)?
                fn $func:ident $([$($gen:tt)*])? ($($args:tt)*)
                $(-> $ret:ty)?
                $(where ($($where:tt)*))?
                $body:block
        )*
    ) => {
        macro_rules! docs {
            $(($func) => {
                $crate::slice::extract_docs!($(#[$($meta)*])*)
            };)*
        }

        #[allow(unused_imports)]
        pub(crate) use docs;

        $(
            $(#[$($meta)*])*
            $vis
            $(const $($const)?)?
            $(unsafe $($unsafe)?)?
            fn $func $(< $($gen)* >)? ($($args)*)
            $(-> $ret)?
            $(where $($where)* )?
            $body
        )*
    };
}

#[allow(unused_imports)]
pub(crate) use methods;

methods! {
    /// Returns the length of the provided slice, in elements.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn len[S](slice: *const S) -> usize
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.len(slice)
    }

    /// Returns whether the provided slice is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_empty[S](slice: *const S) -> bool
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.is_empty(slice)
    }

    /// Creates a raw slice given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice[S](data: *const S::Elem, len: usize) -> *const S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice(data, len)
    }

    /// Creates a mutable raw slice given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice_mut[S](data: *mut S::Elem, len: usize) -> *mut S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice_mut(data, len)
    }

    /// Create a [`NonNull`] raw slice given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice_nonnull[S](data: NonNull<S::Elem>, len: usize) -> NonNull<S>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice_nonnull(data, len)
    }

    /// Create a shared slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_raw_parts['a, S](data: *const S::Elem, len: usize) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is safe.
        unsafe { S::KIND.0.from_raw_parts(data, len) }
    }

    /// Create a mutable slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_raw_parts_mut['a, S](data: *mut S::Elem, len: usize) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is safe.
        unsafe { S::KIND.0.from_raw_parts_mut(data, len) }
    }

    // TODO: Write better docs.
    /// Try to create a slice from a slice of its elements.
    #[inline(always)]
    #[track_caller]
    pub const fn try_from_elems['a, S](elems: &'a [S::Elem]) -> Result<&'a S, FromElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_from_elems(elems)
    }

    // TODO: Write better docs.
    /// Try to create a mutable slice from a mutable slice of its elements.
    #[inline(always)]
    #[track_caller]
    pub const fn try_from_elems_mut['a, S](elems: &'a mut [S::Elem]) -> Result<&'a mut S, FromElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_from_elems_mut(elems)
    }

    // TODO: Write better docs.
    /// Create a slice from a slice of its elements.
    #[inline(always)]
    #[track_caller]
    pub const fn from_elems['a, S](elems: &'a [S::Elem]) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.from_elems(elems)
    }

    // TODO: Write better docs.
    /// Create a mutable slice from a mutable slice of its elements.
    #[inline(always)]
    #[track_caller]
    pub const fn from_elems_mut['a, S](elems: &'a mut [S::Elem]) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.from_elems_mut(elems)
    }

    // TODO: Write better docs.
    /// Create a slice from a slice of its elements without any checks.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[track_caller]
    pub const unsafe fn from_elems_unchecked['a, S](elems: &'a [S::Elem]) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.from_elems_unchecked(elems) }
    }

    // TODO: Write better docs.
    /// Create a mutable slice from a mutable slice of its elements without any checks.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[track_caller]
    pub const unsafe fn from_elems_mut_unchecked['a, S](elems: &'a mut [S::Elem]) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.from_elems_mut_unchecked(elems) }
    }
}

/// Trait for the various slice types we support.
///
/// # Safety
///
/// ***TODO***
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Slice: private::Sealed {
    /// An associated item detailing the underlying in-memory representation
    /// of the implementor.
    ///
    /// The length of implementor is equal to the almount of `Elem`s stored within it.
    ///
    /// # Safety
    ///
    /// The backing memory for this slice must be a valid `[Elem]`. No exceptions are made.
    ///
    /// Therefore, `Self` must have the same alignment requirements as `[Elem]`, as well as
    /// properly initialized for `[Elem]`.
    ///
    /// # Bit Validity
    ///
    /// While all `Self`s contain a `[Elem]`, not all `[Elem]`s are valid as a `Self`.
    type Elem: Sized;

    /// An error that is returned when trying to create a `Self` from some `[Elem]`.``
    type FromElemsErr: 'static + Sized + fmt::Debug + fmt::Display;

    /// An error that is returned when trying to safely get a `[Elem]` from some `Self`.
    type AsElemsErr: 'static + Sized + fmt::Debug + fmt::Display;

    /// An error that may occur when attempting to split this slice into a subslice.
    ///
    /// This does not include out of bounds errors.
    type SplitErr: 'static + Sized + fmt::Debug + fmt::Display;

    // Type witness.
    #[doc(hidden)]
    const KIND: SliceKind<Self>;

    #[doc = docs!(len)]
    #[track_caller]
    #[must_use]
    fn len(&self) -> usize;

    #[doc = docs!(is_empty)]
    #[track_caller]
    #[must_use]
    fn is_empty(&self) -> bool;

    #[doc = docs!(raw_slice)]
    #[track_caller]
    #[must_use]
    fn raw_slice(data: *const Self::Elem, len: usize) -> *const Self;

    #[doc = docs!(raw_slice_mut)]
    #[track_caller]
    #[must_use]
    fn raw_slice_mut(data: *mut Self::Elem, len: usize) -> *mut Self;

    #[doc = docs!(raw_slice_nonnull)]
    #[track_caller]
    #[must_use]
    fn raw_slice_nonnull(data: NonNull<Self::Elem>, len: usize) -> NonNull<Self>;

    #[doc = docs!(from_raw_parts)]
    #[track_caller]
    #[must_use]
    unsafe fn from_raw_parts<'a>(data: *const Self::Elem, len: usize) -> &'a Self;

    #[doc = docs!(from_raw_parts_mut)]
    #[track_caller]
    #[must_use]
    unsafe fn from_raw_parts_mut<'a>(data: *mut Self::Elem, len: usize) -> &'a mut Self;

    #[doc = docs!(try_from_elems)]
    #[track_caller]
    fn try_from_elems<'a>(elems: &'a [Self::Elem]) -> Result<&'a Self, FromElemsError<Self>>;

    #[doc = docs!(try_from_elems_mut)]
    #[track_caller]
    fn try_from_elems_mut<'a>(
        elems: &'a mut [Self::Elem],
    ) -> Result<&'a mut Self, FromElemsError<Self>>;
}

/// Gets a type or it's alternative, preferring the alternative.
macro_rules! get {
    ($a:ty $(|)?) => {
        $a
    };

    ($a:ty | $b:ty $(|)?) => {
        $b
    };
}

/// Defines the various slice implementations.
macro_rules! slice {
    ($(
        $(#[cfg($($cfg:tt)*)])*
        $(#[doc = $doc:expr])*
        unsafe impl $((  $($gen:tt)* ))? Slice
            for $slice:ty $(| $slice_override:ty)? $(|)?
        {
            $(#[$elem_attr:meta])*
            type Elem = $elem:ty $(| $elem_override:ty)? $(|)?;

            $(#[$from_elems_attr:meta])*
            type FromElemsErr = $from_elems:ty $(| $from_elems_override:ty)? $(|)?;

            $(#[$as_elems_attr:meta])*
            type AsElemsErr = $as_elems:ty $(| $as_elems_override:ty)? $(|)?;

            $(#[$split_attr:meta])*
            type SplitErr = $split:ty $(| $split_override:ty)? $(|)?;

            $(#[$variant_attr:meta])*
            type Variant = $variant:ident;

            $(#[$module_attr:meta])*
            type Module = $module:ident;
        }
    )+) => {
        $(
            $(#[$module_attr])*
            $(#[cfg($($cfg)*)])*
            mod $module;

            $(#[cfg($($cfg)*)])*
            impl $(< $($gen)* >)? private::Sealed for $slice {}

            $(#[cfg($($cfg)*)])*
            $(#[doc = $doc])*
            unsafe impl $(< $($gen)* >)? Slice for $slice {
                $(#[$elem_attr])*
                type Elem = $elem;

                $(#[$from_elems_attr])*
                type FromElemsErr = $from_elems;

                $(#[$as_elems_attr])*
                type AsElemsErr = $as_elems;

                $(#[$split_attr])*
                type SplitErr = $split;

                const KIND: SliceKind<Self> = SliceKind(SliceWit::$variant {
                    slice: TypeEq::new(),
                    elem: TypeEq::new(),
                    from_elems_error: TypeEq::new(),
                    as_elems_error: TypeEq::new(),
                    split_error: TypeEq::new(),
                });

                #[doc = self::$module::docs!(len)]
                #[inline(always)]
                #[track_caller]
                fn len(&self) -> usize {
                    len(self)
                }

                #[doc = self::$module::docs!(is_empty)]
                #[inline(always)]
                #[track_caller]
                fn is_empty(&self) -> bool {
                    is_empty(self)
                }

                #[doc = self::$module::docs!(raw_slice)]
                #[inline(always)]
                #[track_caller]
                fn raw_slice(data: *const $elem, len: usize) -> *const $slice {
                    raw_slice(data, len)
                }

                #[doc = $module::docs!(raw_slice_mut)]
                #[inline(always)]
                #[track_caller]
                fn raw_slice_mut(data: *mut $elem, len: usize) -> *mut $slice {
                    raw_slice_mut(data, len)
                }

                #[doc = self::$module::docs!(raw_slice_nonnull)]
                #[inline(always)]
                #[track_caller]
                fn raw_slice_nonnull(data: NonNull<$elem>, len: usize) -> NonNull<$slice> {
                    raw_slice_nonnull(data, len)
                }

                #[doc = self::$module::docs!(from_raw_parts)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_raw_parts<'a>(data: *const $elem, len: usize) -> &'a $slice {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_raw_parts(data, len) }
                }

                #[doc = self::$module::docs!(from_raw_parts_mut)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_raw_parts_mut<'a>(data: *mut $elem, len: usize) -> &'a mut $slice {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_raw_parts_mut(data, len) }
                }

                #[doc = self::$module::docs!(try_from_elems)]
                #[inline(always)]
                #[track_caller]
                fn try_from_elems<'a>(elems: &'a [Self::Elem]) -> Result<&'a Self, FromElemsError<Self>> {
                    try_from_elems(elems)
                }

                #[doc = self::$module::docs!(try_from_elems_mut)]
                #[inline(always)]
                #[track_caller]
                fn try_from_elems_mut<'a>(elems: &'a mut [Self::Elem]) -> Result<&'a mut Self, FromElemsError<Self>> {
                    try_from_elems_mut(elems)
                }
            }
        )+

        #[allow(dead_code)]
        pub(crate) enum SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            $(
                $(#[cfg($($cfg)*)])*
                $variant {
                    slice: TypeEq<S, get!($slice | $($slice_override)?)>,
                    elem: TypeEq<S::Elem, get!($elem | $($elem_override)?)>,
                    from_elems_error: TypeEq<S::FromElemsErr, get!($from_elems | $($from_elems_override)?)>,
                    as_elems_error: TypeEq<S::AsElemsErr, get!($as_elems | $($as_elems_override)?)>,
                    split_error: TypeEq<S::SplitErr, get!($split | $($split_override)?)>,
                },
            )+
        }

        impl<S> Clone for SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            #[inline(always)]
            fn clone(&self) -> SliceWit<S> {
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
            const fn len(
                self,
                s: *const S,
            ) -> usize {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, .. } => self::$module::len(slice.coerce_ptr(s)),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn is_empty(
                self,
                s: *const S,
            ) -> bool {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, .. } => self::$module::is_empty(slice.coerce_ptr(s)),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn raw_slice(
                self,
                data: *const S::Elem,
                len: usize,
            ) -> *const S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_ptr(
                                self::$module::raw_slice(
                                    elem.coerce_ptr(data),
                                    len,
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn raw_slice_mut(
                self,
                data: *mut S::Elem,
                len: usize,
            ) -> *mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_ptr_mut(
                                self::$module::raw_slice_mut(
                                    elem.coerce_ptr_mut(data),
                                    len,
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn raw_slice_nonnull(
                self,
                data: NonNull<S::Elem>,
                len: usize,
            ) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_nonnull(
                                self::$module::raw_slice_nonnull(
                                    elem.coerce_nonnull(data),
                                    len,
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const unsafe fn from_raw_parts<'a>(
                self,
                data: *const S::Elem,
                len: usize,
            ) -> &'a S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_ref(unsafe {
                                self::$module::from_raw_parts(
                                    elem.coerce_ptr(data),
                                    len,
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const unsafe fn from_raw_parts_mut<'a>(
                self,
                data: *mut S::Elem,
                len: usize,
            ) -> &'a mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_mut(unsafe {
                                self::$module::from_raw_parts_mut(
                                    elem.coerce_ptr_mut(data),
                                    len,
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn try_from_elems<'a>(
                self,
                elems: &'a [S::Elem],
            ) -> Result<&'a S, FromElemsError<S>> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .wrap_ref()
                            .wrap_result(slice.wrap_from_elems_error())
                            .uncoerce(
                                self::$module::try_from_elems(
                                    elem
                                        .wrap_slice()
                                        .coerce_ref(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn try_from_elems_mut<'a>(
                self,
                elems: &'a mut [S::Elem],
            ) -> Result<&'a mut S, FromElemsError<S>> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .wrap_mut()
                            .wrap_result(slice.wrap_from_elems_error())
                            .uncoerce(
                                self::$module::try_from_elems_mut(
                                    elem
                                        .wrap_slice()
                                        .coerce_mut(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn from_elems<'a>(
                self,
                elems: &'a [S::Elem],
            ) -> &'a S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .wrap_ref()
                            .uncoerce(
                                self::$module::from_elems(
                                    elem
                                        .wrap_slice()
                                        .coerce_ref(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn from_elems_mut<'a>(
                self,
                elems: &'a mut [S::Elem],
            ) -> &'a mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .wrap_mut()
                            .uncoerce(
                                self::$module::from_elems_mut(
                                    elem
                                        .wrap_slice()
                                        .coerce_mut(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const unsafe fn from_elems_unchecked<'a>(
                self,
                elems: &'a [S::Elem],
            ) -> &'a S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_ref(unsafe {
                                self::$module::from_elems_unchecked(
                                    elem
                                        .wrap_slice()
                                        .coerce_ref(elems),
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const unsafe fn from_elems_mut_unchecked<'a>(
                self,
                elems: &'a mut [S::Elem],
            ) -> &'a mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => slice
                            .uncoerce_mut(unsafe {
                                self::$module::from_elems_mut_unchecked(
                                    elem
                                        .wrap_slice()
                                        .coerce_mut(elems),
                                )
                            }),
                    )*
                }
            }
        }

        // We're going to keep error handlers separate, just for my sanity.

        #[allow(unreachable_code)]
        impl<S> SliceWit<S>
        where
            S: Slice + ?Sized,
        {
            #[inline(always)]
            #[track_caller]
            const fn handle_from_elems_error(
                self,
                error: S::FromElemsErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { from_elems_error, .. } => self::$module::handle_from_elems_error(from_elems_error.coerce(error)),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const unsafe fn handle_from_elems_error_unchecked(
                self,
                error: S::FromElemsErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { from_elems_error, .. } => unsafe {
                            self::$module::handle_from_elems_error_unchecked(from_elems_error.coerce(error))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn handle_as_elems_error(
                self,
                error: S::AsElemsErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { as_elems_error, .. } => self::$module::handle_as_elems_error(as_elems_error.coerce(error)),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const unsafe fn handle_as_elems_error_unchecked(
                self,
                error: S::AsElemsErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { as_elems_error, .. } => unsafe {
                            self::$module::handle_as_elems_error_unchecked(as_elems_error.coerce(error))
                        },
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn handle_split_error(
                self,
                error: S::SplitErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { split_error, .. } => self::$module::handle_split_error(split_error.coerce(error)),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const unsafe fn handle_split_error_unchecked(
                self,
                error: S::SplitErr,
            ) -> ! {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { split_error, .. } => unsafe {
                            self::$module::handle_split_error_unchecked(split_error.coerce(error))
                        },
                    )*
                }
            }
        }
    };
}

slice! {
    unsafe impl(T) Slice for [T] | [S::Elem]  {
        type Elem = T | S::Elem;

        type FromElemsErr = Infallible;
        type AsElemsErr = Infallible;
        type SplitErr = Infallible;

        type Variant = Slice;
        type Module = slice;
    }

    unsafe impl Slice for str {
        type Elem = u8;

        type FromElemsErr = Utf8Error;
        type AsElemsErr = StrAsElemsError;
        type SplitErr = StrSplitError;

        type Variant = Str;
        type Module = str;
    }
}

#[doc(inline)]
pub use self::str::*;
