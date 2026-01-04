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
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::len`]
    /// as implemented for `S`/`Self`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn len[S](
        slice: *const S,
    ) -> usize
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.len(slice)
    }

    /// Returns whether the given slice contains no elements.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::is_empty`]
    /// as implemented for `S`/`Self`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_empty[S](
        slice: *const S,
    ) -> bool
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.is_empty(slice)
    }

    /// Creates a raw slice given a data pointer and length.
    ///
    /// For more detailed documentation and safety info, refer to the
    /// documentation for [`Slice::raw_slice`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// The returned raw slice *may not be valid*. It is not safe to make any safety assumptions
    /// about a raw slice.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice[S](
        data: *const S::Elem,
        len: usize,
    ) -> *const S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice(data, len)
    }

    /// Creates a mutable raw slice given a data pointer and length.
    ///
    /// For more detailed documentation and safety info, refer to the
    /// documentation for [`Slice::raw_slice_mut`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// The returned raw slice *may not be valid*. It is not safe to make any safety assumptions
    /// about a raw slice.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice_mut[S](
        data: *mut S::Elem,
        len: usize,
    ) -> *mut S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice_mut(data, len)
    }

    /// Create a [`NonNull`] raw slice given a data pointer and length.
    ///
    /// For more detailed documentation and safety info, refer to the
    /// documentation for [`Slice::raw_slice_nonnull`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// The returned raw slice *may not be valid*. It is not safe to make any safety assumptions
    /// about a raw slice.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn raw_slice_nonnull[S](
        data: NonNull<S::Elem>,
        len: usize,
    ) -> NonNull<S>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.raw_slice_nonnull(data, len)
    }

    /// Create a shared slice reference given a data pointer and length.
    ///
    /// For more detailed documentation and safety info, refer to the
    /// documentation for [`Slice::from_raw_parts`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure the following, and failure to do so is *undefined behavior*:
    ///
    /// - That the invariants of [`core::slice::from_raw_parts`] are upheld.
    /// - That the invariants of [`Slice::from_elems_unchecked`] for `S`/`Self` are upheld.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_raw_parts['a, S](
        data: *const S::Elem,
        len: usize,
    ) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is safe.
        unsafe { S::KIND.0.from_raw_parts(data, len) }
    }

    /// Create a mutable slice reference given a data pointer and length.
    ///
    /// For more detailed documenatation and safety info, refer to the
    /// documentation for [`Slice::from_raw_parts_mut`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure the following, and failure to do so is *undefined behavior*:
    ///
    /// - That the invariants of [`core::slice::from_raw_parts_mut`] are upheld.
    /// - That the invariants of [`Slice::from_elems_mut_unchecked`] for `S`/`Self` are upheld.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_raw_parts_mut['a, S](
        data: *mut S::Elem,
        len: usize,
    ) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is safe.
        unsafe { S::KIND.0.from_raw_parts_mut(data, len) }
    }

    /// Try to create a slice from a slice of its elements.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::try_from_elems`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Returns
    ///
    /// - Returns [`Ok`] upon success.
    /// - Returns [`Err`] upon failure.
    #[inline(always)]
    #[track_caller]
    pub const fn try_from_elems['a, S](
        elems: &'a [S::Elem],
    ) -> Result<&'a S, FromElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_from_elems(elems)
    }

    /// Try to create a mutable slice from a mutable slice of its elements.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::try_from_elems_mut`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Returns
    ///
    /// - Returns [`Ok`] upon success.
    /// - Returns [`Err`] upon failure.
    #[inline(always)]
    #[track_caller]
    pub const fn try_from_elems_mut['a, S](
        elems: &'a mut [S::Elem],
    ) -> Result<&'a mut S, FromElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_from_elems_mut(elems)
    }

    /// Create a slice from a slice of its elements.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::from_elems`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Panics
    ///
    /// Panics if [`Slice::try_from_elems`] fails.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_elems['a, S](
        elems: &'a [S::Elem],
    ) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.from_elems(elems)
    }

    /// Create a mutable slice from a mutable slice of its elements.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::from_elems_mut`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Panics
    ///
    /// Panics if [`Slice::try_from_elems_mut`] fails.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn from_elems_mut['a, S](
        elems: &'a mut [S::Elem],
    ) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.from_elems_mut(elems)
    }

    /// Create a slice from a slice of its elements without any checks.
    ///
    /// For more detailed documentation and safety info, refer to
    /// the documentation for [`Slice::from_elems_unchecked`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the invariants of [`Slice::from_elems_unchecked`]
    /// as implemented for `S`/`Self` are upheld. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_elems_unchecked['a, S](
        elems: &'a [S::Elem],
    ) -> &'a S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.from_elems_unchecked(elems) }
    }

    /// Create a mutable slice from a mutable slice of its elements without any checks.
    ///
    /// For more detailed documentation and safety info, refer to
    /// the documentation for [`Slice::from_elems_mut_unchecked`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the invariants of [`Slice::from_elems_mut_unchecked`]
    /// as implemented for `S`/`Self` are upheld. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn from_elems_mut_unchecked['a, S](
        elems: &'a mut [S::Elem],
    ) -> &'a mut S
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.from_elems_mut_unchecked(elems) }
    }

    /// Try to get a reference to the underlying elements of a slice.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::try_as_elems`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Returns
    ///
    /// - Returns [`Ok`] upon success.
    /// - Returns [`Err`] upon failure.
    #[inline(always)]
    #[track_caller]
    pub const fn try_as_elems['a, S](
        slice: &'a S,
    ) -> Result<&'a [S::Elem], AsElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_as_elems(slice)
    }

    /// Get a reference to the underlying elements of a slice.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::as_elems`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Panics
    ///
    /// Panics if [`Slice::try_as_elems`] fails.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn as_elems['a, S](
        slice: &'a S,
    ) -> &'a [S::Elem]
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.as_elems(slice)
    }

    /// Get a reference to the underlying elements of a slice without any checks.
    ///
    /// For more detailed documentation and safety info, refer to
    /// the documentation for [`Slice::as_elems_unchecked`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the invariants of [`Slice::as_elems_unchecked`]
    /// as implemented for `S`/`Self` are upheld. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn as_elems_unchecked['a, S](
        slice: &'a S,
    ) -> &'a [S::Elem]
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.as_elems_unchecked(slice) }
    }

    /// Try to get a mutable reference to the underlying elements of a slice.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::try_as_elems_mut`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Returns
    ///
    /// - Returns [`Ok`] upon success.
    /// - Returns [`Err`] upon failure.
    #[inline(always)]
    #[track_caller]
    pub const fn try_as_elems_mut['a, S](
        slice: &'a mut S,
    ) -> Result<&'a mut [S::Elem], AsElemsError<S>>
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.try_as_elems_mut(slice)
    }

    /// Get a mutable reference to the underlying elements of a slice.
    ///
    /// For more detailed documentation, refer to the documentation for [`Slice::as_elems_mut`]
    /// as implemented for `S`/`Self`.
    ///
    /// # Panics
    ///
    /// Panics if [`Slice::try_as_elems_mut`] fails.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn as_elems_mut['a, S](
        slice: &'a mut S,
    ) -> &'a mut [S::Elem]
    where (
        S: Slice + ?Sized,
    ) {
        S::KIND.0.as_elems_mut(slice)
    }

    /// Get a mutable reference to the underlying elements of a slice without any checks.
    ///
    /// For more detailed documentation and safety info, refer to
    /// the documentation for [`Slice::as_elems_mut_unchecked`] as implemented for `S`/`Self`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the invariants of [`Slice::as_elems_mut_unchecked`]
    /// as implemented for `S`/`Self` are upheld. Failure to do so is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const unsafe fn as_elems_mut_unchecked['a, S](
        slice: &'a mut S,
    ) -> &'a mut [S::Elem]
    where (
        S: Slice + ?Sized,
    ) {
        // SAFETY: The caller ensures this is sound.
        unsafe { S::KIND.0.as_elems_mut_unchecked(slice)  }
    }
}

/// Trait for the various slice types we support.
///
/// # Safety
///
/// ***TODO***
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Slice: private::Sealed {
    /// An associated type that "backs" the underlying in-memory representation of this slice type.
    ///
    /// This *also* is how the length of a slice is determined. A slice's length, regardless of type,
    /// is the *amount of elements it contains*.
    ///
    /// # Safety
    ///
    /// The underlying memory for any `Self` must be a valid `[Elem]`. No exceptions.
    ///
    /// This requires the following:
    ///
    /// - The alignment requirements for `Self` and `[Elem]` must be the same.
    /// - That the bit validity and initialization requirements of `Self` is
    ///   at least as restrictive as `[Elem]`, but not any less restrictive.
    ///
    ///   For example, `[u8]` is the backing storage of all [`prim@str`]s, and
    ///   much like a `[u8]`, all bytes must be initialized.
    ///
    ///   However, [`prim@str`] has further invariants. In order for some region
    ///   of memory to be a valid [`prim@str`], it *must* be a valid `[u8]`, and
    ///   be entirely comprised of well-formed UTF-8.
    /// - Likely more, don't mess this up. Seriously.
    type Elem: Sized;

    /// An error that is returned when trying to create a `Self` from some `[Elem]`.
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
    #[must_use]
    #[track_caller]
    fn len(&self) -> usize;

    #[doc = docs!(is_empty)]
    #[must_use]
    #[track_caller]
    fn is_empty(&self) -> bool;

    #[doc = docs!(raw_slice)]
    #[must_use]
    #[track_caller]
    fn raw_slice(data: *const Self::Elem, len: usize) -> *const Self;

    #[doc = docs!(raw_slice_mut)]
    #[must_use]
    #[track_caller]
    fn raw_slice_mut(data: *mut Self::Elem, len: usize) -> *mut Self;

    #[doc = docs!(raw_slice_nonnull)]
    #[must_use]
    #[track_caller]
    fn raw_slice_nonnull(data: NonNull<Self::Elem>, len: usize) -> NonNull<Self>;

    #[doc = docs!(from_raw_parts)]
    #[must_use]
    #[track_caller]
    unsafe fn from_raw_parts<'a>(data: *const Self::Elem, len: usize) -> &'a Self;

    #[doc = docs!(from_raw_parts_mut)]
    #[must_use]
    #[track_caller]
    unsafe fn from_raw_parts_mut<'a>(data: *mut Self::Elem, len: usize) -> &'a mut Self;

    #[doc = docs!(try_from_elems)]
    #[track_caller]
    fn try_from_elems<'a>(elems: &'a [Self::Elem]) -> Result<&'a Self, FromElemsError<Self>>;

    #[doc = docs!(try_from_elems_mut)]
    #[track_caller]
    fn try_from_elems_mut<'a>(
        elems: &'a mut [Self::Elem],
    ) -> Result<&'a mut Self, FromElemsError<Self>>;

    #[doc = docs!(from_elems)]
    #[must_use]
    #[track_caller]
    fn from_elems<'a>(elems: &'a [Self::Elem]) -> &'a Self;

    #[doc = docs!(from_elems_mut)]
    #[must_use]
    #[track_caller]
    fn from_elems_mut<'a>(elems: &'a mut [Self::Elem]) -> &'a mut Self;

    #[doc = docs!(from_elems_unchecked)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_unchecked<'a>(elems: &'a [Self::Elem]) -> &'a Self;

    #[doc = docs!(from_elems_mut_unchecked)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_mut_unchecked<'a>(elems: &'a mut [Self::Elem]) -> &'a mut Self;

    #[doc = docs!(try_as_elems)]
    #[track_caller]
    fn try_as_elems<'a>(&'a self) -> Result<&'a [Self::Elem], AsElemsError<Self>>;

    #[doc = docs!(try_as_elems_mut)]
    #[track_caller]
    fn try_as_elems_mut<'a>(&'a mut self) -> Result<&'a mut [Self::Elem], AsElemsError<Self>>;

    #[doc = docs!(as_elems)]
    #[must_use]
    #[track_caller]
    fn as_elems<'a>(&'a self) -> &'a [Self::Elem];

    #[doc = docs!(as_elems_mut)]
    #[must_use]
    #[track_caller]
    fn as_elems_mut<'a>(&'a mut self) -> &'a mut [Self::Elem];

    #[doc = docs!(as_elems_unchecked)]
    #[must_use]
    #[track_caller]
    unsafe fn as_elems_unchecked<'a>(&'a self) -> &'a [Self::Elem];

    #[doc = docs!(as_elems_mut_unchecked)]
    #[must_use]
    #[track_caller]
    unsafe fn as_elems_mut_unchecked<'a>(&'a mut self) -> &'a mut [Self::Elem];
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

                #[doc = self::$module::docs!(from_elems)]
                #[inline(always)]
                #[track_caller]
                fn from_elems<'a>(elems: &'a [Self::Elem]) -> &'a Self {
                    from_elems(elems)
                }

                #[doc = self::$module::docs!(from_elems_mut)]
                #[inline(always)]
                #[track_caller]
                fn from_elems_mut<'a>(elems: &'a mut [Self::Elem]) -> &'a mut Self {
                    from_elems_mut(elems)
                }

                #[doc = self::$module::docs!(from_elems_unchecked)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_elems_unchecked<'a>(elems: &'a [Self::Elem]) -> &'a Self {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_elems_unchecked(elems) }
                }

                #[doc = self::$module::docs!(from_elems_mut_unchecked)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_elems_mut_unchecked<'a>(elems: &'a mut [Self::Elem]) -> &'a mut Self {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_elems_mut_unchecked(elems) }
                }

                #[doc = self::$module::docs!(try_as_elems)]
                #[inline(always)]
                #[track_caller]
                fn try_as_elems<'a>(&'a self) -> Result<&'a [Self::Elem], AsElemsError<Self>> {
                    try_as_elems(self)
                }

                #[doc = self::$module::docs!(try_as_elems_mut)]
                #[inline(always)]
                #[track_caller]
                fn try_as_elems_mut<'a>(&'a mut self) -> Result<&'a mut [Self::Elem], AsElemsError<Self>> {
                    try_as_elems_mut(self)
                }

                #[doc = self::$module::docs!(as_elems)]
                #[inline(always)]
                #[track_caller]
                fn as_elems<'a>(&'a self) -> &'a [Self::Elem] {
                    as_elems(self)
                }

                #[doc = self::$module::docs!(as_elems_mut)]
                #[inline(always)]
                #[track_caller]
                fn as_elems_mut<'a>(&'a mut self) -> &'a mut [Self::Elem] {
                    as_elems_mut(self)
                }

                #[doc = self::$module::docs!(as_elems_unchecked)]
                #[inline(always)]
                #[track_caller]
                unsafe fn as_elems_unchecked<'a>(&'a self) -> &'a [Self::Elem] {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { as_elems_unchecked(self) }
                }

                #[doc = self::$module::docs!(as_elems_mut_unchecked)]
                #[inline(always)]
                #[track_caller]
                unsafe fn as_elems_mut_unchecked<'a>(&'a mut self) -> &'a mut [Self::Elem] {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { as_elems_mut_unchecked(self) }
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
                        Self::$variant {
                            slice, ..
                        } => self::$module::len(
                            slice.coerce_ptr(s),
                        ),
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
                        Self::$variant {
                            slice, ..
                        } => self::$module::is_empty(
                            slice.coerce_ptr(s),
                        ),
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
                                    elem.coerce_slice(elems),
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
                                    elem.coerce_slice_mut(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
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
                            .uncoerce_ref(
                                self::$module::from_elems(
                                    elem.coerce_slice(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
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
                            .uncoerce_mut(
                                self::$module::from_elems_mut(
                                    elem.coerce_slice_mut(elems),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
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
                                    elem.coerce_slice(elems),
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
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
                                    elem.coerce_slice_mut(elems),
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn try_as_elems<'a>(
                self,
                slice: &'a S,
            ) -> Result<&'a [S::Elem], AsElemsError<S>> {
                let this = slice;
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .wrap_slice()
                            .wrap_ref()
                            .wrap_result(slice.wrap_as_elems_error())
                            .uncoerce(
                                self::$module::try_as_elems(
                                    slice.coerce_ref(this),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn as_elems<'a>(
                self,
                slice: &'a S,
            ) -> &'a [S::Elem] {
                let this = slice;

                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .uncoerce_slice(
                                self::$module::as_elems(
                                    slice.coerce_ref(this),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const unsafe fn as_elems_unchecked<'a>(
                self,
                slice: &'a S,
            ) -> &'a [S::Elem] {
                let this = slice;
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .uncoerce_slice(unsafe {
                                self::$module::as_elems_unchecked(
                                    slice.coerce_ref(this),
                                )
                            }),
                    )*
                }
            }

            #[inline(always)]
            #[track_caller]
            const fn try_as_elems_mut<'a>(
                self,
                slice: &'a mut S,
            ) -> Result<&'a mut [S::Elem], AsElemsError<S>> {
                let this = slice;

                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .wrap_slice()
                            .wrap_mut()
                            .wrap_result(slice.wrap_as_elems_error())
                            .uncoerce(
                                self::$module::try_as_elems_mut(
                                    slice.coerce_mut(this),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const fn as_elems_mut<'a>(
                self,
                slice: &'a mut S,
            ) -> &'a mut [S::Elem] {
                let this = slice;

                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .uncoerce_slice_mut(
                                self::$module::as_elems_mut(
                                    slice.coerce_mut(this),
                                ),
                            ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            const unsafe fn as_elems_mut_unchecked<'a>(
                self,
                slice: &'a mut S,
            ) -> &'a mut [S::Elem] {
                let this = slice;

                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant {
                            slice, elem, ..
                        } => elem
                            .uncoerce_slice_mut(unsafe {
                                self::$module::as_elems_mut_unchecked(
                                    slice.coerce_mut(this),
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
                        Self::$variant {
                            from_elems_error, ..
                        } => self::$module::handle_from_elems_error(
                            from_elems_error.coerce(error),
                        ),
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
                        Self::$variant {
                            from_elems_error, ..
                        } => unsafe {
                            self::$module::handle_from_elems_error_unchecked(
                                from_elems_error.coerce(error),
                            )
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
                        Self::$variant {
                            as_elems_error, ..
                        } => self::$module::handle_as_elems_error(
                            as_elems_error.coerce(error),
                        ),
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
                        Self::$variant {
                            as_elems_error, ..
                        } => unsafe {
                            self::$module::handle_as_elems_error_unchecked(
                                as_elems_error.coerce(error),
                            )
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
                        Self::$variant {
                            split_error, ..
                        } => self::$module::handle_split_error(
                            split_error.coerce(error),
                        ),
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
                            self::$module::handle_split_error_unchecked(
                                split_error.coerce(error),
                            )
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
