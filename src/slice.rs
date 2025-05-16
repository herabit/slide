use core::{fmt, hash, ptr::NonNull};

use crate::{str::char_count_unchecked, type_eq::TypeEq};

mod private {
    pub trait Sealed {}
}

/// Trait for slices that we're allowed to slide over.
pub unsafe trait Slice: private::Sealed {
    // The type contained by the underlying buffer of a slice.
    #[doc(hidden)]
    type Item: Sized;

    // Associated metadata for this slice type.
    #[doc(hidden)]
    type Meta: 'static + Send + Sync + Copy + Eq + Ord + hash::Hash + fmt::Debug;

    // What kind of slice this is.
    #[doc(hidden)]
    const KIND: SliceKind<Self>;

    // The name of the metadata field, if we want to show it in `fmt::Debug`.
    #[doc(hidden)]
    const META_NAME: Option<&'static str> = None;

    // Whether `Item` is a ZST.
    #[doc(hidden)]
    const IS_ZST: bool = size_of::<Self::Item>() == 0;
}

#[doc(hidden)]
#[non_exhaustive]
pub enum SliceKind<S: Slice + ?Sized> {
    Slice {
        /// Convert `S` to a slice of items.
        this: TypeEq<S, [S::Item]>,
        /// Convert `S::Meta` to `()`.
        unit: TypeEq<S::Meta, ()>,
    },
    Str {
        /// Convert `S` to a `str`.
        this: TypeEq<S, str>,
        /// Convert `S::Item` to `u8`.
        item: TypeEq<S::Item, u8>,
        /// Convert `S::Meta` to `usize`.
        ///
        /// This is represents the character count of
        /// the string.
        char_count: TypeEq<S::Meta, CharCount>,
    },
}

impl<S: Slice + ?Sized> Clone for SliceKind<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for SliceKind<S> {}

impl<T> private::Sealed for [T] {}
unsafe impl<T> Slice for [T] {
    type Item = T;
    type Meta = ();

    const KIND: SliceKind<Self> = SliceKind::Slice {
        this: TypeEq::new(),
        unit: TypeEq::new(),
    };
}

impl private::Sealed for str {}
unsafe impl Slice for str {
    type Item = u8;
    type Meta = CharCount;

    const KIND: SliceKind<Self> = SliceKind::Str {
        this: TypeEq::new(),
        item: TypeEq::new(),
        char_count: TypeEq::new(),
    };

    const META_NAME: Option<&'static str> = Some("char_count");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CharCount(pub usize);
