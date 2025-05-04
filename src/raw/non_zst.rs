use core::{
    mem,
    ptr::{self, NonNull},
};

use crate::util::{assert_unchecked, nonnull_slice, ptr_len};

use super::RawSlide;

/// A slide for types that are not zero sized.
#[repr(C)]
pub(super) struct Slide<T> {
    /// The start of the buffer.
    ///
    /// INVARIANT: `start <= end`.
    /// INVARIANT: `start <= cursor`.
    start: NonNull<T>,
    /// The end of the buffer.
    ///
    /// This is a pointer to the past-the-end element of the buffer,
    /// calculated as `slice.as_ptr().add(slice.len())`, more or less.
    ///
    /// INVARIANT: `end >= start`.
    /// INVARIANT: `end >= cursor`.
    end: NonNull<T>,
    /// The current position in the buffer.
    ///
    /// It is either a pointer to the next element in the buffer,
    /// or a pointer to the past-the-end element of the buffer.
    ///
    /// In other words:
    ///
    /// - `cursor < end`: There is at least one element in the slide.
    /// - `cursor == end`: The slide has consumed all available elements.
    /// - `cursor > end`: Impossible.
    ///
    /// INVARIANT: `cursor >= start`.
    /// INVARIANT: `cursor <= end`.
    cursor: NonNull<T>,
}

impl<T> Slide<T> {
    /// Create a new slide for types that are not zero sized, without any checks.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure:
    ///
    /// - That `size_of::<T>() != 0`.
    /// - That `slice` points to some valid allocated object.
    /// - That `offset <= slice.len()`.
    /// - Likely other stuff, don't fuck this up.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const unsafe fn new(slice: NonNull<[T]>, offset: usize) -> Self {
        debug_assert!(
            size_of::<T>() != 0,
            "undefined behavior: size must not be zero"
        );
        debug_assert!(
            size_of::<T>().saturating_mul(slice.len()) <= isize::MAX as usize,
            "undefined behavior: invalid size for allocated object"
        );
        debug_assert!(
            offset <= slice.len(),
            "undefined behavior: offset exceeds slice length"
        );

        let start = slice.cast::<T>();
        let end = unsafe { start.add(slice.len()) };
        let cursor = unsafe { start.add(offset) };

        Self { start, end, cursor }
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
        assert!(size_of::<T>() != 0, "size must not be zero");

        unsafe { &*ptr::from_ref(raw).cast() }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn from_raw_mut(raw: &mut RawSlide<T>) -> &mut Self {
        assert!(size_of::<T>() != 0, "size must not be zero");

        unsafe { &mut *ptr::from_mut(raw).cast() }
    }
}

impl<T> Slide<T> {
    #[inline(always)]
    #[track_caller]
    pub(super) const fn compiler_hints(&self) {
        let Self { start, end, cursor } = self;

        let offset = unsafe { ptr_len(start.as_ptr()..cursor.as_ptr()) };
        let len = unsafe { ptr_len(start.as_ptr()..end.as_ptr()) };

        unsafe { assert_unchecked!(offset <= len, "undefined behavior: `offset > len`") }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn source(&self) -> NonNull<[T]> {
        nonnull_slice(self.start, unsafe {
            ptr_len(self.start.as_ptr()..self.end.as_ptr())
        })
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn consumed(&self) -> NonNull<[T]> {
        nonnull_slice(self.start, unsafe {
            ptr_len(self.start.as_ptr()..self.cursor.as_ptr())
        })
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn remaining(&self) -> NonNull<[T]> {
        nonnull_slice(self.cursor, unsafe {
            ptr_len(self.cursor.as_ptr()..self.end.as_ptr())
        })
    }
}

impl<T> Slide<T> {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(super) const fn offset(&self) -> usize {
        unsafe { ptr_len(self.start.as_ptr()..self.cursor.as_ptr()) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        self.cursor = unsafe { self.start.add(offset) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn advance_unchecked(&mut self, n: usize) {
        self.cursor = unsafe { self.cursor.add(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn rewind_unchecked(&mut self, n: usize) {
        self.cursor = unsafe { self.cursor.sub(n) }
    }

    #[inline(always)]
    #[track_caller]
    pub(super) const unsafe fn seek_unchecked(&mut self, n: isize) {
        self.cursor = unsafe { self.cursor.offset(n) }
    }

    #[inline(always)]
    #[must_use]
    pub(super) const unsafe fn peek_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        nonnull_slice(self.cursor, n)
    }

    #[inline(always)]
    #[must_use]
    pub(super) const unsafe fn peek_back_slice_unchecked(&self, n: usize) -> NonNull<[T]> {
        nonnull_slice(unsafe { self.cursor.sub(n) }, n)
    }
}

impl<T> Clone for Slide<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slide<T> {}
