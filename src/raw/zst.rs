use core::{
    mem,
    ptr::{self, NonNull},
};

use crate::util::{assert_unchecked, nonnull_slice};

use super::RawSlide;

/// A slide for zero sized types.
#[repr(C)]
pub(super) struct Slide<T> {
    /// The start of the buffer.
    start: NonNull<T>,
    /// The length of the buffer.
    ///
    /// INVARIANT: `length >= offset`.
    length: usize,
    /// Where we are in the buffer.
    ///
    /// This doubles as the length of the consumed buffer.
    ///
    /// INVARIANT: `offset <= length`.
    offset: usize,
}

impl<T> Slide<T> {
    /// Create a new slide for zero sized types, without any checks.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure:
    ///
    /// - That `size_of::<T>() == 0`.
    /// - That `slice` points to some valid allocated object.
    /// - That `offset <= slice.len()`.
    /// - Likely other stuff, don't fuck this up.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const unsafe fn new(slice: NonNull<[T]>, offset: usize) -> Self {
        debug_assert!(size_of::<T>() == 0, "undefined behavior: size must be zero");
        debug_assert!(
            offset <= slice.len(),
            "undefined behavior: offset exceeds slice length"
        );

        Self {
            start: slice.cast(),
            length: slice.len(),
            offset,
        }
    }

    #[inline(always)]
    #[must_use]
    pub(super) const fn into_raw(self) -> RawSlide<T> {
        unsafe { mem::transmute(self) }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn from_raw_ref(raw: &RawSlide<T>) -> &Self {
        assert!(size_of::<T>() == 0, "size must be zero");

        unsafe { &*ptr::from_ref(raw).cast() }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn from_raw_mut(raw: &mut RawSlide<T>) -> &mut Self {
        assert!(size_of::<T>() == 0, "size must be zero");

        unsafe { &mut *ptr::from_mut(raw).cast() }
    }
}

impl<T> Slide<T> {
    #[inline(always)]
    #[track_caller]
    pub(super) const fn compiler_hints(&self) {
        unsafe {
            assert_unchecked!(
                self.offset <= self.length,
                "undefined behavior: `offset > length`"
            );
        }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn source(&self) -> NonNull<[T]> {
        nonnull_slice(self.start, self.length)
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn consumed(&self) -> NonNull<[T]> {
        nonnull_slice(self.start, self.offset)
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn remaining(&self) -> NonNull<[T]> {
        nonnull_slice(self.start, self.length.checked_sub(self.offset).unwrap())
    }
}

impl<T> Slide<T> {
    #[inline(always)]
    #[must_use]
    pub(super) const fn offset(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub(super) const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        self.offset = offset
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn advance_unchecked(&mut self, n: usize) {
        self.offset = unsafe { self.offset.unchecked_add(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn rewind_unchecked(&mut self, n: usize) {
        self.offset = unsafe { self.offset.unchecked_sub(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn seek_unchecked(&mut self, n: isize) {
        self.offset = unsafe { self.offset.checked_add_signed(n).unwrap_unchecked() };
    }

    #[inline(always)]
    #[must_use]
    pub(super) const unsafe fn peek_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        nonnull_slice(self.start, n)
    }

    #[inline(always)]
    #[must_use]
    pub(super) const unsafe fn peek_back_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        nonnull_slice(self.start, n)
    }
}

impl<T> Clone for Slide<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slide<T> {}
