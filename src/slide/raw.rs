use core::{cmp::Ordering, num::NonZero, ptr::NonNull};

use crate::{
    macros::assert_unchecked,
    mem::NoDrop,
    slice::{OobIndex, Slice, SplitError, len, raw_slice_nonnull, validate_split_at},
    slide::location::Location,
    util::cmp_usize,
};

/// An unsafe slide that is used to implement the other, "real" slides.
///
/// Currently this is an implementation detail, however we may move to make
/// this public.
///
/// This could be used to implement slides for smart pointers... Potentially.
///
/// # Safety
///
/// One has to be extremely careful when dealing with this type, as this utilizes
/// raw pointers and makes few guarantees.
///
/// Some things to keep in mind:
///
/// - That the backing slice is well formed for `S` while a
///   given [`RawSlice`] is in use.
///
///   For example, it is invalid to have a `RawSlide<str>` over a slice
///   that contains ill-formed UTF-8.
///
/// - That the backing slice is derived from a single, contiguous
///   *allocated object*.
///
/// - That the cursor is *always within bounds* and lies upon a
///   valid split boundary for `S` while the given [`RawSlice`] is in use.
///
/// - Other shit... Don't fuck this up. I need to flesh this fully out later.
#[repr(C)]
pub(crate) struct RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// The start of the buffer.
    ///
    /// # Safety
    ///
    /// The invariant `start <= cursor && start <= end` must always hold true.
    start: NonNull<S::Elem>,
    /// The current location.
    ///
    /// # Safety
    ///
    /// These invariants must always hold true:
    ///
    /// - `cursor >= start && cursor <= end`
    ///
    /// - Must lie on a valid splitting boundary for `S`.
    cursor: Location<S>,
    /// The end of the buffer.
    ///
    /// # Safety
    ///
    /// These invariants must always hold true:
    ///
    /// - `end >= start && end >= cursor`
    ///
    /// - Must lie on a valid splitting boundary for `S`.
    end: Location<S>,
}

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Create a new raw slide without checks.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure:
    ///
    /// - That `slice` is a single, valid *allocated object* that is properly initialized
    ///   for `S` while the returned [`RawSlide`] is in use.
    ///
    /// - That `offset` is within the bounds of `slice` (`offset <= slice.len()`)
    ///   and lies on a valid split boundary for `S`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new_unchecked(
        slice: NonNull<S>,
        offset: usize,
    ) -> Self {
        let this = {
            let len = len(slice.as_ptr());
            let start = slice.cast::<S::Elem>();

            // SAFETY: The caller ensures `offset` is a valid split boundary for `slice`.
            let cursor = unsafe { Location::new(start, offset) };

            // SAFETY: The caller ensures `len` is the end of the slice, and therefore a
            //         valid split boundary for `slice`.
            let end = unsafe { Location::new(start, len) };

            RawSlide { start, cursor, end }
        };

        this.compiler_hints();

        this
    }

    /// Try to create a new raw slide.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure:
    ///
    /// - That `slice` is a single, valid *allocated object* that is properly initialized
    ///   for `S` while the returned [`RawSlide`] is in use.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn new(
        slice: NonNull<S>,
        offset: usize,
    ) -> Result<Self, SplitError<S>> {
        // SAFETY: The caller ensures that `slice` is a valid `S`.
        match NoDrop::new(validate_split_at(unsafe { slice.as_ref() }, offset)).transpose() {
            // SAFETY: We just checked that `offset` is a valid split boundary.
            Ok(..) => Ok(unsafe { Self::new_unchecked(slice, offset) }),
            Err(err) => Err(err.into_inner()),
        }
    }

    /// Create a new raw slide given a reference without any checks.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that `offset` is within the bounds
    /// of `slice` (`offset <= slice.len()`) and lies on a valid split
    /// boundary for `S`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_ref_unchecked(
        slice: &S,
        offset: usize,
    ) -> Self {
        // SAFETY: We know that `slice` is valid, and the caller ensures `offset`
        //         is a valid split boundary.
        unsafe { Self::new_unchecked(NonNull::from_ref(slice), offset) }
    }

    /// Try to create a new raw slide given a reference.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn from_ref(
        slice: &S,
        offset: usize,
    ) -> Result<Self, SplitError<S>> {
        // SAFETY: We know that `slice` is valid.
        unsafe { Self::new(NonNull::from_ref(slice), offset) }
    }

    /// Create a new raw slide given a mutable reference without any checks.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that `offset` is within the bounds
    /// of `slice` (`offset <= slice.len()`) and lies on a valid split
    /// boundary for `S`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_mut_unchecked(
        slice: &mut S,
        offset: usize,
    ) -> Self {
        // SAFETY: We know that `slice` is valid, and the caller ensures `offset`
        //         is a valid split boundary.
        unsafe { Self::new_unchecked(NonNull::from_mut(slice), offset) }
    }

    /// Try to create a new raw slide given a mutable reference.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn from_mut(
        slice: &mut S,
        offset: usize,
    ) -> Result<Self, SplitError<S>> {
        // SAFETY: We know that `slice` is valid.
        unsafe { Self::new(NonNull::from_mut(slice), offset) }
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

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Determine whether we're able to advance the slide by `amount` elems.
    ///
    /// ***TODO***
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn validate_advance(
        &self,
        amount: usize,
    ) -> Result<(), SplitError<S>> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to advance the cursor.
        validate_split_at(unsafe { self.remaining_ref() }, amount)
    }

    /// Determine whether we're able to rewind the slide by `amount` elems.
    ///
    /// ***TODO***
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn validate_rewind(
        &self,
        amount: usize,
    ) -> Result<(), SplitError<S>> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to rewind the cursor.
        let consumed = unsafe { self.consumed_ref() };

        match cmp_usize(amount, len(consumed)) {
            Ordering::Less | Ordering::Equal => validate_split_at(consumed, len(consumed) - amount),
            Ordering::Greater => Err(SplitError::OutOfBounds {
                index: NonZero::new(len(consumed) as OobIndex - amount as OobIndex).unwrap(),
                len: len(consumed),
            }),
        }
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
