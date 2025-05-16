use core::ptr::NonNull;

use crate::{Slice, macros::assert_unchecked, slice::SliceKind, util::nonnull_slice};

#[repr(C)]
pub(crate) struct Slide<S: Slice + ?Sized> {
    // Start of the slide.
    start: NonNull<S::Item>,
    // Position to the current item in the slide or one past the final item.
    cursor: Pos<S>,
    // Position to one past the final item in the slide.
    end: Pos<S>,

    // Metadata for `start..cursor`.
    consumed_meta: S::Meta,
    // Metadata for `start..end`
    source_meta: S::Meta,
}

impl<S: Slice + ?Sized> Clone for Slide<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for Slide<S> {}

#[repr(C)]
pub(crate) struct Chunk<S: Slice + ?Sized> {
    pub(crate) slice: NonNull<S>,
    pub(crate) meta: S::Meta,
}

impl<S: Slice + ?Sized> Chunk<S> {
    #[inline(always)]
    #[must_use]
    pub(crate) const fn len(&self) -> usize {
        match S::KIND {
            SliceKind::Slice { this, .. } => this.wrap_nonnull().into_right(self.slice).len(),
            SliceKind::Str { this, .. } => {
                (this.wrap_nonnull().into_right(self.slice).as_ptr() as *const [u8]).len()
            }
        }
    }
}

impl Chunk<str> {
    #[inline(always)]
    #[must_use]
    pub(crate) const fn char_count(&self) -> usize {
        unsafe { assert_unchecked!(self.meta.0 <= self.len, "`char_count > len`") };

        self.meta.0
    }

    #[inline(always)]
    #[must_use]
    pub(crate) const fn is_ascii(&self) -> bool {
        self.len() == self.char_count()
    }
}

impl<S: Slice + ?Sized> Clone for Chunk<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for Chunk<S> {}

#[repr(C)]
pub(crate) union Pos<S: Slice + ?Sized> {
    ptr: NonNull<S::Item>,
    offset: usize,
}

impl<S: Slice + ?Sized> Clone for Pos<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Slice + ?Sized> Copy for Pos<S> {}
