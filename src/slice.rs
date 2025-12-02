use crate::{marker::TypeEq, slice::private::SliceKind};
use core::{convert::Infallible, fmt, str::Utf8Error};

/// Internal implementation details.
pub(crate) mod private;

/// Trait for the various slice types we support.
///
/// # Safety
///
/// ***TODO***
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

    /// An error that is returned when trying to create a `Self` from some `[Elem]`.
    type FromElemsError: Sized + fmt::Debug + fmt::Display;

    /// An error that is returned when trying to safely get a `[Elem]` from some `Self`.
    type AsElemsError: Sized + fmt::Debug + fmt::Display;

    /// An error that may occur when attempting to split this slice into a subslice.
    ///
    /// This does not include out of bounds errors.
    type SplitError: Sized + fmt::Debug + fmt::Display;

    // Type witness.
    #[doc(hidden)]
    const KIND: SliceKind<Self>;
}

pub unsafe trait AsElems: Slice {}
pub unsafe trait AsElemsMut: AsElems {}

unsafe impl<S> AsElems for S where S: Slice<AsElemsError = Infallible> + ?Sized {}
unsafe impl<S> AsElemsMut for S where S: Slice<AsElemsError = Infallible> + ?Sized {}

macro_rules! get {
    ($a:ty $(|)?) => {
        $a
    };

    ($a:ty | $b:ty $(|)?) => {
        $b
    };
}

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
            type FromElemsError = $from_elems:ty $(| $from_elems_override:ty)? $(|)?;

            $(#[$as_elems_attr:meta])*
            type AsElemsError = $as_elems:ty $(| $as_elems_override:ty)? $(|)?;

            $(#[$split_attr:meta])*
            type SplitError = $split:ty $(| $split_override:ty)? $(|)?;

            $(#[$variant_attr:meta])*
            type Variant = $variant:ident;

            $(#[$module_attr:meta])*
            type Module = $module:ident;
        }
    )+) => {
        $(
            $(#[cfg($($cfg)*)])*
            impl $(< $($gen)* >)? private::Sealed for $slice {}

            $(#[cfg($($cfg)*)])*
            $(#[doc = $doc])*
            unsafe impl $(< $($gen)* >)? Slice for $slice {
                $(#[$elem_attr])*
                type Elem = $elem;

                $(#[$from_elems_attr])*
                type FromElemsError = $from_elems;

                $(#[$as_elems_attr])*
                type AsElemsError = $as_elems;

                $(#[$split_attr])*
                type SplitError = $split;

                const KIND: SliceKind<Self> = SliceKind(SliceWit::$variant {
                    slice: TypeEq::new(),
                    elem: TypeEq::new(),
                    from_elems_error: TypeEq::new(),
                    as_elems_error: TypeEq::new(),
                    split_error: TypeEq::new(),
                });
            }


            $(#[$module_attr])*
            $(#[cfg($($cfg)*)])*
            mod $module;
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
                    from_elems_error: TypeEq<S::FromElemsError, get!($from_elems | $($from_elems_override)?)>,
                    as_elems_error: TypeEq<S::AsElemsError, get!($as_elems | $($as_elems_override)?)>,
                    split_error: TypeEq<S::SplitError, get!($split | $($split_override)?)>,
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
    };
}

slice! {
    unsafe impl(T) Slice for [T] | [S::Elem]  {
        type Elem = T | S::Elem;

        type FromElemsError = Infallible;
        type AsElemsError = Infallible;
        type SplitError = Infallible;

        type Variant = Slice;
        type Module = slice;
    }

    unsafe impl Slice for str {
        type Elem = u8;

        type FromElemsError = Utf8Error;
        type AsElemsError = &'static str;
        type SplitError = &'static str;

        type Variant = Str;
        type Module = str;
    }
}

unsafe impl AsElems for str {}
