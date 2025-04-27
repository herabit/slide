use core::marker::PhantomData;

use crate::{Slide, raw::RawSlide};

#[repr(transparent)]
pub struct SlideMut<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub(crate) const unsafe fn from_raw(raw: RawSlide<T>) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub const fn new(slice: &'a mut [T]) -> Self {
        unsafe { Self::from_raw(RawSlide::from_slice_mut(slice)) }
    }

    #[inline]
    #[must_use]
    pub const fn with_offset(slice: &'a mut [T], offset: usize) -> Option<Self> {
        match RawSlide::from_slice_mut_offset(slice, offset) {
            Some(raw) => Some(unsafe { Self::from_raw(raw) }),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        let (offset, _) = self.raw.offset_len();

        offset
    }

    #[inline]
    #[must_use]
    pub const fn as_slide(&self) -> Slide<'_, T> {
        unsafe { Slide::from_raw(self.raw) }
    }

    #[inline]
    #[must_use]
    pub const fn as_slide_mut(&mut self) -> SlideMut<'_, T> {
        unsafe { SlideMut::from_raw(self.raw) }
    }

    #[inline]
    #[must_use]
    pub const fn into_slide(self) -> Slide<'a, T> {
        unsafe { Slide::from_raw(self.raw) }
    }
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &[T] {
        unsafe { self.raw.source().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn source_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.source().as_mut() }
    }

    #[inline]
    #[must_use]
    pub const fn into_source(self) -> &'a mut [T] {
        unsafe { self.raw.source().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub const fn consumed(&self) -> &[T] {
        unsafe { self.raw.consumed().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn consumed_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.consumed().as_mut() }
    }

    #[inline]
    #[must_use]
    pub const fn into_consumed(self) -> &'a mut [T] {
        unsafe { self.raw.consumed().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> &[T] {
        unsafe { self.raw.remaining().as_ref() }
    }

    #[inline]
    #[must_use]
    pub const fn remaining_mut(&mut self) -> &mut [T] {
        unsafe { self.raw.remaining().as_mut() }
    }

    #[inline]
    #[must_use]
    pub const fn into_remaining(self) -> &'a mut [T] {
        unsafe { self.raw.remaining().as_mut() }
    }
}

impl<'a, T> SlideMut<'a, T> {
    #[inline]
    #[must_use]
    pub const fn split(&self) -> (&[T], &[T]) {
        (self.consumed(), self.remaining())
    }

    #[inline]
    #[must_use]
    pub const fn split_mut(&mut self) -> (&mut [T], &mut [T]) {
        unsafe { (self.raw.consumed().as_mut(), self.raw.remaining().as_mut()) }
    }

    #[inline]
    #[must_use]
    pub const fn into_split(self) -> (&'a mut [T], &'a mut [T]) {
        unsafe { (self.raw.consumed().as_mut(), self.raw.remaining().as_mut()) }
    }
}

unsafe impl<'a, T> Send for SlideMut<'a, T> where &'a mut [T]: Send {}
unsafe impl<'a, T> Sync for SlideMut<'a, T> where &'a mut [T]: Sync {}
