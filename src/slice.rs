#![allow(clippy::empty_docs)]

use crate::{marker::TypeEq, slice::private::SliceKind};
use core::{cmp::Ordering, convert::Infallible, fmt, hash, mem, ptr::NonNull, str::Utf8Error};

/// Internal implementation details.
pub(crate) mod private;

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
    type FromElemsErr: Sized + fmt::Debug + fmt::Display;

    /// An error that is returned when trying to safely get a `[Elem]` from some `Self`.
    type AsElemsErr: Sized + fmt::Debug + fmt::Display;

    /// An error that may occur when attempting to split this slice into a subslice.
    ///
    /// This does not include out of bounds errors.
    type SplitErr: Sized + fmt::Debug + fmt::Display;

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
}

/// The error type that is returned when creating
/// a slice from its component elements.
///
/// ***TODO***
pub struct FromElemsError<S>(pub S::FromElemsErr)
where
    S: Slice + ?Sized;

impl<S> fmt::Debug for FromElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for FromElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> Default for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S> Clone for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<S> Copy for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Copy,
{
}

impl<S> PartialEq for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Eq,
{
}

impl<S> PartialOrd for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<S> core::error::Error for FromElemsError<S>
where
    S: Slice + ?Sized,
    S::FromElemsErr: core::error::Error,
{
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }
}

/// The error type that is returned when trying to get a slice's
/// component elements.
///
/// ***TODO***
pub struct AsElemsError<S>(pub S::AsElemsErr)
where
    S: Slice + ?Sized;

impl<S> fmt::Debug for AsElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> fmt::Display for AsElemsError<S>
where
    S: Slice + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S> Default for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S> PartialEq for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<S> Eq for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Eq,
{
}

impl<S> PartialOrd for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> hash::Hash for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<S> core::error::Error for AsElemsError<S>
where
    S: Slice + ?Sized,
    S::AsElemsErr: core::error::Error,
{
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }
}

/// The error type that is returned when attempting to split a
/// slice.
///
/// ***TODO***
pub enum SplitError<S>
where
    S: Slice + ?Sized,
{
    /// Cannot split at the specified index, it is out of bounds.
    OutOfBounds {
        /// The index that is out of bounds.
        index: usize,
        /// The length of the slice.
        len: usize,
    },
    /// Some other error occurred.
    Other(S::SplitErr),
}

impl<S> Clone for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match *self {
            Self::OutOfBounds { index, len } => Self::OutOfBounds { index, len },
            Self::Other(ref other) => Self::Other(other.clone()),
        }
    }
}

impl<S> Copy for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Copy,
{
}

impl<S> fmt::Debug for SplitError<S>
where
    S: Slice + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { index, len } => f
                .debug_struct("OutOfBounds")
                .field("index", index)
                .field("len", len)
                .finish(),
            Self::Other(other) => f.debug_tuple("Other").field(other).finish(),
        }
    }
}

impl<S> fmt::Display for SplitError<S>
where
    S: Slice + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitError::OutOfBounds { index, len } => {
                core::write!(f, "index is out of bounds: `{index} >= {len}`")
            }
            SplitError::Other(other) => other.fmt(f),
        }
    }
}

impl<S> PartialEq for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                Self::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => l_index == r_index && l_len == r_len,
            (Self::Other(l), Self::Other(r)) => l == r,
            _ => false,
        }
    }
}

impl<S> Eq for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Eq,
{
}

impl<S> PartialOrd for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                SplitError::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                SplitError::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => Some({
                let index = l_index.cmp(r_index);
                let len = l_len.cmp(r_len);

                index.then(len)
            }),
            (SplitError::Other(l), SplitError::Other(r)) => l.partial_cmp(r),
            (SplitError::OutOfBounds { .. }, SplitError::Other(_)) => Some(Ordering::Greater),
            (SplitError::Other(_), SplitError::OutOfBounds { .. }) => Some(Ordering::Less),
        }
    }
}

impl<S> Ord for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                SplitError::OutOfBounds {
                    index: l_index,
                    len: l_len,
                },
                SplitError::OutOfBounds {
                    index: r_index,
                    len: r_len,
                },
            ) => {
                let index = l_index.cmp(r_index);
                let len = l_len.cmp(r_len);

                index.then(len)
            }
            (SplitError::Other(l), SplitError::Other(r)) => l.cmp(r),
            (SplitError::OutOfBounds { .. }, SplitError::Other(_)) => Ordering::Greater,
            (SplitError::Other(_), SplitError::OutOfBounds { .. }) => Ordering::Less,
        }
    }
}

impl<S> hash::Hash for SplitError<S>
where
    S: Slice + ?Sized,
    S::SplitErr: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);

        match self {
            SplitError::OutOfBounds { index, len } => {
                index.hash(state);
                len.hash(state);
            }
            SplitError::Other(other) => other.hash(state),
        }
    }
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

                #[doc = $module::docs!(len)]
                #[inline(always)]
                #[track_caller]
                fn len(&self) -> usize {
                    len(self)
                }

                #[doc = $module::docs!(is_empty)]
                #[inline(always)]
                #[track_caller]
                fn is_empty(&self) -> bool {
                    is_empty(self)
                }

                #[doc = $module::docs!(raw_slice)]
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

                #[doc = $module::docs!(raw_slice_nonnull)]
                #[inline(always)]
                #[track_caller]
                fn raw_slice_nonnull(data: NonNull<$elem>, len: usize) -> NonNull<$slice> {
                    raw_slice_nonnull(data, len)
                }

                #[doc = $module::docs!(from_raw_parts)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_raw_parts<'a>(data: *const $elem, len: usize) -> &'a $slice {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_raw_parts(data, len) }
                }

                #[doc = $module::docs!(from_raw_parts_mut)]
                #[inline(always)]
                #[track_caller]
                unsafe fn from_raw_parts_mut<'a>(data: *mut $elem, len: usize) -> &'a mut $slice {
                    // SAFETY: The caller ensures this is safe.
                    unsafe { from_raw_parts_mut(data, len) }
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
            pub const fn len(self, s: *const S) -> usize {
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
            pub const fn is_empty(self, s: *const S) -> bool {
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
            pub const fn raw_slice(self, data: *const S::Elem, len: usize) -> *const S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, elem, .. } => slice.uncoerce_ptr(
                            self::$module::raw_slice(elem.coerce_ptr(data), len),
                        ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub const fn raw_slice_mut(self, data: *mut S::Elem, len: usize) -> *mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, elem, .. } => slice.uncoerce_ptr_mut(
                            self::$module::raw_slice_mut(elem.coerce_ptr_mut(data), len)
                        ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub const fn raw_slice_nonnull(self, data: NonNull<S::Elem>, len: usize) -> NonNull<S> {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, elem, .. } => slice.uncoerce_nonnull(
                            self::$module::raw_slice_nonnull(elem.coerce_nonnull(data), len)
                        ),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub const unsafe fn from_raw_parts<'a>(self, data: *const S::Elem, len: usize) -> &'a S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, elem, .. } => slice.uncoerce_ref(unsafe {
                            self::$module::from_raw_parts(elem.coerce_ptr(data), len)
                        }),
                    )*
                }
            }

            #[inline(always)]
            #[must_use]
            #[track_caller]
            pub const unsafe fn from_raw_parts_mut<'a>(self, data: *mut S::Elem, len: usize) -> &'a mut S {
                match self {
                    $(
                        $(#[cfg($($cfg)*)])*
                        Self::$variant { slice, elem, .. } => slice.uncoerce_mut(unsafe {
                            self::$module::from_raw_parts_mut(elem.coerce_ptr_mut(data), len)
                        }),
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
        type AsElemsErr = &'static str;
        type SplitErr = &'static str;

        type Variant = Str;
        type Module = str;
    }
}

#[doc(inline)]
pub use self::str::*;
