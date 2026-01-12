use core::ptr::NonNull;

use crate::{
    macros::assert_unchecked,
    slice::{Slice, len, raw_slice_nonnull},
    slide::location::Location,
};

/// A raw slide.
#[repr(C)]
pub(crate) struct RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// The start of the buffer (`start <= cursor && start <= end`).
    start: NonNull<S::Elem>,
    /// The current location (`cursor >= start && cursor <= end`).
    cursor: Location<S>,
    /// The end of the buffer.
    end: Location<S>,
}

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Create a new raw slide without checks.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new_unchecked(
        slice: NonNull<S>,
        offset: usize,
    ) -> Self {
        let len = len(slice.as_ptr());

        unsafe { assert_unchecked!(offset <= len, "`offset > len`") };

        let start = slice.cast::<S::Elem>();
        let cursor = unsafe { Location::new(start, offset) };
        let end = unsafe { Location::new(start, len) };

        let this = Self { start, cursor, end };

        this.compiler_hints();

        this
    }

    /// Compiler hints.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn compiler_hints(&self) {
        // SAFETY: `start..end` is a valid memory range.
        let entire = unsafe { self.end.offset_from(self.start) };
        // SAFETY: `start..cursor` is a valid memory range.
        let consumed = unsafe { self.cursor.offset_from(self.start) };
        // SAFETY: `cursor..end` is a valid memory range.
        let remaining = unsafe { self.end.offset_from(self.cursor) };

        // SAFETY: These invariants are always upheld.
        unsafe {
            assert_unchecked!(
                consumed.unchecked_add(remaining) == entire,
                "`consumed.checked_add(remaining) != Some(entire)`"
            )
        };
    }

    /// Returns a raw pointer into the consumed slice.
    ///
    /// # Safety
    ///
    /// This method returns a raw pointer, and it is up to the
    /// caller to ensure it is utilized properly.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn consumed_raw(&self) -> NonNull<S> {
        self.compiler_hints();

        // SAFETY: `start..cursor` is a valid memory range.
        let len = unsafe { self.cursor.offset_from(self.start) };
        let ptr = self.start;

        raw_slice_nonnull(ptr, len)
    }

    /// Returns a reference into the consumed slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create shared borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn consumed_ref<'a>(&self) -> &'a S {
        // SAFETY: The caller ensures that it is safe to create shared borrows
        //         to the underlying slice that last for `'a`.
        unsafe { self.consumed_raw().as_ref() }
    }

    /// Returns a mutable refereence into the consumed slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create exclusive borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn consumed_mut<'a>(&mut self) -> &'a mut S {
        // SAFETY: The caller ensures that it is safe to create exclusive borrows
        //         to the underlying slice that last for `'a`.
        unsafe { self.consumed_raw().as_mut() }
    }

    /// Returns a raw pointer into the remaining slice.
    ///
    /// # Safety
    ///
    /// This method returns a raw pointer, and it is up to the
    /// caller to ensure it is utilized properly.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn remaining_raw(&self) -> NonNull<S> {
        self.compiler_hints();

        // SAFETY: `cursor..end` is a valid memory range.
        let len = unsafe { self.end.offset_from(self.cursor) };
        // SAFETY: Since `cursor..end` is a valid memory range,
        //         `cursor.apply(start)` is a valid start to the memory range.
        let ptr = unsafe { self.cursor.apply(self.start) };

        raw_slice_nonnull(ptr, len)
    }

    /// Returns a reference into the remaining slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create shared borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn remaining_ref<'a>(&self) -> &'a S {
        // SAFETY: The caller ensure that it is safe to create shared borrows
        //        to the underlying slice that last for `'a`.
        unsafe { self.remaining_raw().as_ref() }
    }

    /// Returns a mutable reference into the remaining slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create exclusive borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn remaining_mut<'a>(&mut self) -> &'a mut S {
        // SAFETY: The caller ensures that it is safe to create exclusive borrows
        //         to the underlying slice that last for `'a`.
        unsafe { self.remaining_raw().as_mut() }
    }

    /// Returns a raw pointer into the slice, but split.
    ///
    /// # Safety
    ///
    /// This method returns raw pointers, and it is up to the
    /// caller to ensure they're utilized properly.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn split_raw(&self) -> (NonNull<S>, NonNull<S>) {
        (self.consumed_raw(), self.remaining_raw())
    }

    /// Returns references into the slice, but split.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(consumed, remaining)`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create shared borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_ref<'a>(&self) -> (&'a S, &'a S) {
        // SAFETY: The caller ensures that it is safe to create shared borrows
        //         to the underlying slice that last for `'a`.
        unsafe { (self.consumed_raw().as_ref(), self.remaining_raw().as_ref()) }
    }

    /// Returns mutable references into the slice, but split.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(consumed, remaining)`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create exclusive borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_mut<'a>(&mut self) -> (&'a mut S, &'a mut S) {
        // SAFETY: The caller ensures that it is safe to create exclusive borrows
        //         to the underlying slice that last for `'a`.
        unsafe { (self.consumed_raw().as_mut(), self.remaining_raw().as_mut()) }
    }

    /// Returns a raw pointer into the entire slice.
    ///
    /// # Safety
    ///
    /// This method returns a raw pointer, and it is up to the
    /// caller to ensure it is utilized properly.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn entire_raw(&self) -> NonNull<S> {
        self.compiler_hints();

        // SAFETY: `start..end` is a valid memory range.
        let len = unsafe { self.end.offset_from(self.start) };
        let ptr = self.start;

        raw_slice_nonnull(ptr, len)
    }

    /// Returns a reference into the entire slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create shared borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn entire_ref<'a>(&self) -> &'a S {
        // SAFETY: The caller ensures that it is safe to create shared borrows
        //         to the underlying slice that last for `'a`.
        unsafe { self.entire_raw().as_ref() }
    }

    /// Returns a mutable reference into the entire slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to create mutable borrows
    /// to the underlying slice that last for `'a`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn entire_mut<'a>(&mut self) -> &'a mut S {
        // SAFETY: The caller ensures that it is safe to create exclusive borrows
        //         to the underltying slice that last for `'a`.
        unsafe { self.entire_raw().as_mut() }
    }
}

impl<S> Clone for RawSlide<S>
where
    S: Slice + ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for RawSlide<S> where S: Slice + ?Sized {}
