use core::marker::PhantomData;

use crate::{Direction, raw::RawSlide};

#[repr(transparent)]
pub struct Slide<'a, T> {
    raw: RawSlide<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Slide<'a, T> {
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
    pub const fn new(slice: &'a [T]) -> Self {
        unsafe { Self::from_raw(RawSlide::from_slice(slice)) }
    }

    #[inline]
    #[must_use]
    pub const fn with_offset(slice: &'a [T], offset: usize) -> Option<Self> {
        match RawSlide::from_slice_offset(slice, offset) {
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
        *self
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
    pub const fn split(&self) -> (&'a [T], &'a [T]) {
        (self.consumed(), self.remaining())
    }
}

impl<'a, T> Slide<'a, T> {
    #[inline]
    #[track_caller]
    pub const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe { self.raw.set_offset_unchecked(offset) }
    }

    #[inline]
    #[track_caller]
    pub const fn set_offset_checked(&mut self, offset: usize) -> Option<()> {
        self.raw.set_offset_checked(offset)
    }

    #[inline]
    #[track_caller]
    pub const fn set_offset(&mut self, offset: usize) {
        self.raw.set_offset(offset)
    }
}

impl<'a, T> Slide<'a, T> {
    #[inline]
    #[track_caller]
    pub const unsafe fn slide_unchecked(&mut self, dir: Direction, n: usize) {
        unsafe { self.raw.slide_unchecked(dir, n) }
    }

    #[inline]
    #[track_caller]
    pub const fn slide_checked(&mut self, dir: Direction, n: usize) -> Option<()> {
        self.raw.slide_checked(dir, n)
    }

    #[inline]
    #[track_caller]
    pub const fn slide(&mut self, dir: Direction, n: usize) {
        self.raw.slide(dir, n)
    }
}

impl<'a, T> Slide<'a, T> {
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_slice_unchecked(&self, dir: Direction, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_slice_unchecked(dir, n).as_ref() }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_array_unchecked<const N: usize>(&self, dir: Direction) -> &'a [T; N] {
        unsafe { self.raw.peek_array_unchecked::<N>(dir).as_ref() }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_unchecked(&self, dir: Direction) -> &'a T {
        unsafe { self.raw.peek_unchecked(dir).as_ref() }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice_checked(&self, dir: Direction, n: usize) -> Option<&'a [T]> {
        match self.raw.peek_slice_checked(dir, n) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array_checked<const N: usize>(&self, dir: Direction) -> Option<&'a [T; N]> {
        match self.raw.peek_array_checked(dir) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_checked(&self, dir: Direction) -> Option<&'a T> {
        match self.raw.peek_checked(dir) {
            Some(ptr) => Some(unsafe { ptr.as_ref() }),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice(&self, dir: Direction, n: usize) -> &'a [T] {
        unsafe { self.raw.peek_slice(dir, n).as_ref() }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array<const N: usize>(&self, dir: Direction) -> &'a [T; N] {
        unsafe { self.raw.peek_array::<N>(dir).as_ref() }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek(&self, dir: Direction) -> &'a T {
        unsafe { self.raw.peek(dir).as_ref() }
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
