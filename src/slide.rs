use core::marker::PhantomData;

use crate::raw::RawSlide;

#[repr(transparent)]
pub struct Slide<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Slide<'a, T> {
    #[inline]
    #[must_use]
    pub const fn new(slice: &'a [T]) -> Self {
        Self {
            raw: RawSlide::from_slice(slice),
            _marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_offset(slice: &'a [T], offset: usize) -> Option<Self> {
        match RawSlide::from_slice_offset(slice, offset) {
            Some(raw) => Some(Self {
                raw,
                _marker: PhantomData,
            }),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'a [T] {
        unsafe { self.raw.source().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn consumed(&self) -> &'a [T] {
        unsafe { self.raw.consumed().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> &'a [T] {
        unsafe { self.raw.remaining().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        let (offset, _) = self.raw.offset_len();

        offset
    }
}

impl<'a, T> Clone for Slide<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Slide<'a, T> {}

unsafe impl<'a, T> Send for Slide<'a, T> where &'a [T]: Send {}
unsafe impl<'a, T> Sync for Slide<'a, T> where &'a [T]: Sync {}
