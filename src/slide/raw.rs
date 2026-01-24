use core::{cmp::Ordering, num::NonZero, ptr::NonNull};

use crate::{
    macros::assert_unchecked,
    mem::NoDrop,
    slice::{
        OobIndex, Slice, SplitError, len, raw_slice_nonnull, split_error_handler, validate_split_at,
    },
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
///   valid split boundary for `S` while the given [`RawSlide`] is in use.
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
        let result = validate_split_at(unsafe { slice.as_ref() }, offset);

        match NoDrop::new(result).transpose() {
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
                "`consumed.len().checked_add(remaining.len()) != Some(entire.len())`"
            )
        };
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

    /// Returns the length of the entire slice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn entire_len(&self) -> usize {
        len(self.entire_raw().as_ptr())
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

    /// Returns the length of the consumed slice.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn consumed_len(&self) -> usize {
        len(self.consumed_raw().as_ptr())
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

    /// Returns the length of the remaining slice.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn remaining_len(&self) -> usize {
        len(self.remaining_raw().as_ptr())
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
        let (consumed, remaining) = self.split_raw();

        // SAFETY: The caller ensures that it is safe to create shared borrows
        //         to the underlying slice that last for `'a`.
        unsafe { (consumed.as_ref(), remaining.as_ref()) }
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
        let (mut consumed, mut remaining) = self.split_raw();

        // SAFETY: The caller ensures that it is safe to create exclusive borrows
        //         to the underlying slice that last for `'a`.
        unsafe { (consumed.as_mut(), remaining.as_mut()) }
    }
}

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Attempt to peek ahead by `amount` elements.
    ///
    /// # Returns
    ///
    /// - Upon success,`Ok(peeked)` is returned where `peeked` is the peeked subslice.
    /// - Upon failure, `Err(error)` is returned indicating why it is not valid to
    ///   look ahead by `amount` elements.
    ///
    ///   See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    ///   what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn try_peek_ahead(
        &self,
        amount: usize,
    ) -> Result<NonNull<S>, SplitError<S>> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to peek ahead the cursor.
        let result = validate_split_at(unsafe { self.remaining_ref() }, amount);

        match NoDrop::new(result).transpose() {
            Ok(..) => {
                // SAFETY: If we're peeking ahead, and we can peek `amount` elements ahead,
                //         then `amount <= remaining.len()`.
                //
                //         We insert this because, at least for `str`s, this information gets lost.
                unsafe {
                    assert_unchecked!(amount <= self.remaining_len(), "`amount > remaining.len()`")
                };

                // SAFETY: We already know `cursor` to be at a valid position.
                let ptr = unsafe { self.cursor.apply(self.start) };

                Ok(raw_slice_nonnull(ptr, amount))
            }
            Err(error) => {
                // SAFETY: All erroneous indices are nonzero, this is because for *any* slice,
                //         it is always valid to get `slice[0..]`, as `slice[0..] == slice`.
                unsafe { assert_unchecked!(amount != 0, "`index == 0`") };

                Err(error.into_inner())
            }
        }
    }

    /// Attempt to peek behind by `amount` elements.
    ///
    /// # Returns
    ///
    /// - Upon success, `Ok(peeked)` is returned where `peeked` is the peeked subslice.
    /// - Upon failure, `Err(error)` is returned indicating why it is not valid to
    ///   look behind by `amount` elements.
    ///
    ///   See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    ///   what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn try_peek_behind(
        &self,
        amount: usize,
    ) -> Result<NonNull<S>, SplitError<S>> {
        let result = match cmp_usize(amount, self.consumed_len()) {
            Ordering::Less | Ordering::Equal => validate_split_at(
                // SAFETY: The caller ensures that it is safe to create a temporary reference
                //         to validate whether we're able to peek behind the cursor.
                unsafe { self.consumed_ref() },
                self.consumed_len().strict_sub(amount),
            ),
            Ordering::Greater => {
                let index = (self.consumed_len() as OobIndex).strict_sub(amount as OobIndex);

                Err(SplitError::OutOfBounds {
                    index: NonZero::new(index).expect("erroneous indices are always nonzero"),
                    len: self.consumed_len(),
                })
            }
        };

        match NoDrop::new(result).transpose() {
            Ok(..) => {
                // SAFETY: If we're peeking behind, and we can peek `amount` elements behind,
                //         then `amount <= consumed.len()`.
                //
                //         We insert this because, at least for `str`s, this information gets lost.
                unsafe {
                    assert_unchecked!(amount <= self.consumed_len(), "`amount > consumed.len()`")
                };

                // SAFETY: We now know `cursor - amount` to be at a valid position.
                let ptr = unsafe { self.cursor.rewind(amount).apply(self.start) };

                Ok(raw_slice_nonnull(ptr, amount))
            }
            Err(error) => {
                let index = (self.consumed_len() as OobIndex).strict_sub(amount as OobIndex);

                // SAFETY: All erroneous indices are nonzero, this is because for *any* slice,
                //         it is always valid to get `slice[0..]`, as `slice[0..] == slice`.
                unsafe { assert_unchecked!(index != 0, "`index == 0`") };

                Err(error.into_inner())
            }
        }
    }

    /// Attempt to advance the slide by `amount` elements.
    ///
    /// # Returns
    ///
    /// - Upon success, `Ok(advanced)` is returned where `advanced` is the advanced subslice.
    /// - Upon failure, `Err(error)` is returned indicating why it is not valid to
    ///   advance by `amount` elements.
    ///
    ///   See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    ///   what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn try_advance(
        &mut self,
        amount: usize,
    ) -> Result<NonNull<S>, SplitError<S>> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to advance the cursor.
        let result = unsafe { self.try_peek_ahead(amount) };

        match NoDrop::new(result).transpose() {
            Ok(advanced) => {
                // SAFETY: If we can peek ahead by `amount`, then it is sound to
                //         advance by `amount`.
                unsafe { self.cursor.advance_assign(amount) };

                // SAFETY: We advanced the cursor, so now we need to tell LLVM that
                //         `amount <= consumed.len()`, as this is now true but LLVM
                //         sometimes struggles to understand this, particularly for
                //         `str`s when the slide is not index based.
                //
                // NOTE: This also has the added benefit of running `compiler_hints`
                //       for us as well, additionally proving that the slide is now
                //       still valid.
                unsafe {
                    assert_unchecked!(amount <= self.consumed_len(), "`amount > consumed.len()`")
                };

                Ok(advanced.into_inner())
            }
            Err(error) => Err(error.into_inner()),
        }
    }

    /// Attempt to rewind the slide by `amount` elements.
    ///
    /// # Returns
    ///
    /// - Upon success, `Ok(rewound)` is returned where `rewound` is the rewound subslice.
    /// - Upon failure, `Err(error)` is returned indicating why it is not valid to
    ///   rewind by `amount` elements.
    ///
    ///   See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    ///   what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn try_rewind(
        &mut self,
        amount: usize,
    ) -> Result<NonNull<S>, SplitError<S>> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to rewind the cursor.
        let result = unsafe { self.try_peek_behind(amount) };

        match NoDrop::new(result).transpose() {
            Ok(rewound) => {
                // SAFETY: If we can peek behind by `amount`, then it is sound to
                //         rewind by `amount`.
                unsafe { self.cursor.rewind_assign(amount) };

                // SAFETY: We rewound the cursor, so we need to tell LLVM that
                //         `amount <= remaining.len()`, as this is now true but LLVM
                //         sometimes struggles to understand this, particularly for
                //         `str`s when the slide is not index based.
                //
                // NOTE: This also has the added benefit of running `compiler_hints`
                //       for us as well, additionally proving that the slide is now
                //       still valid.
                unsafe {
                    assert_unchecked!(amount <= self.remaining_len(), "`amount > remaining.len()`")
                };

                Ok(rewound.into_inner())
            }
            Err(error) => Err(error.into_inner()),
        }
    }
}

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Peek ahead by `amount` elements.
    ///
    /// # Returns
    ///
    /// Returns the peeked subslice.
    ///
    /// # Panics
    ///
    /// Panics when it is invalid to look ahead by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_ahead(
        &self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to peek ahead the cursor.
        let result = unsafe { self.try_peek_ahead(amount) };

        match NoDrop::new(result).transpose() {
            Ok(peeked) => peeked.into_inner(),
            Err(..) => {
                // SAFETY: The caller ensures that it is safe to create a temporary reference
                //         which we only use for the sake of panicking.
                let remaining = unsafe { self.remaining_ref() };

                // SAFETY: We know that we encountered an error.
                unsafe {
                    split_error_handler(
                        remaining,
                        NonZero::new(amount as OobIndex)
                            .expect("erroneous indices are always nonzero"),
                    )
                }
            }
        }
    }

    /// Peek behind by `amount` elements.
    ///
    /// # Returns
    ///
    /// Returns the peeked subslice.
    ///
    /// # Panics
    ///
    /// Panics when it is invalid to look behind by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_behind(
        &self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to peek behind the cursor.
        let result = unsafe { self.try_peek_behind(amount) };

        match NoDrop::new(result).transpose() {
            Ok(peeked) => peeked.into_inner(),
            Err(..) => {
                // SAFETY: The caller ensures that it is safe to create a temporary reference
                //         which we only use for the sake of panicking.
                let consumed = unsafe { self.consumed_ref() };

                let index = (self.consumed_len() as OobIndex).strict_sub(amount as OobIndex);

                // SAFETY: We know that we encountered an error.
                unsafe {
                    split_error_handler(
                        consumed,
                        NonZero::new(index).expect("erroneous indices are always nonzero"),
                    )
                }
            }
        }
    }

    /// Advance the slide by `amount` elements.
    ///
    /// # Returns
    ///
    /// Returns the advanced subslice.
    ///
    /// # Panics
    ///
    /// Panics when it is invalid to advance the slide by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn advance(
        &mut self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to advance the cursor.
        let result = unsafe { self.try_advance(amount) };

        match NoDrop::new(result).transpose() {
            Ok(advanced) => advanced.into_inner(),
            Err(..) => {
                // SAFETY: The caller ensures that it is safe to create a temporary reference
                //         which we only use for the sake of panicking.
                let remaining = unsafe { self.remaining_ref() };

                // SAFETY: We know that we encountered an error.
                unsafe {
                    split_error_handler(
                        remaining,
                        NonZero::new(amount as OobIndex)
                            .expect("erroneous indices are always nonzero"),
                    )
                }
            }
        }
    }

    /// Rewind the slide by `amount` elements.
    ///
    /// # Returns
    ///
    /// Returns the rewound subslice.
    ///
    /// # Panics
    ///
    /// Panics when it is invalid to rewind the slide by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn rewind(
        &mut self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to rewind the cursor.
        let result = unsafe { self.try_rewind(amount) };

        match NoDrop::new(result).transpose() {
            Ok(rewound) => rewound.into_inner(),
            Err(..) => {
                // SAFETY: The caller ensures that it is safe to create a temporary reference
                //         which we only use for the sake of panicking.
                let consumed = unsafe { self.consumed_ref() };

                let index = (self.consumed_len() as OobIndex).strict_sub(amount as OobIndex);

                // SAFETY: We know that we encountered an error.
                unsafe {
                    split_error_handler(
                        consumed,
                        NonZero::new(index).expect("erroneous indices are always nonzero"),
                    )
                }
            }
        }
    }
}

impl<S> RawSlide<S>
where
    S: Slice + ?Sized,
{
    /// Peek ahead the slide by `amount` elements without any checks.
    ///
    /// # Returns
    ///
    /// Returns the peeked subslice.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation. Despite this method not doing any checks on release builds,
    /// it *does* actually do checks on debug builds to catch *undefined behavior*.
    ///
    /// Additionally, the caller needs to ensure that it is indeed valid to peek ahead by `amount`
    /// elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary to avoid *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_ahead_unchecked(
        &self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to peek ahead the cursor.
        let result = unsafe { self.try_peek_ahead(amount) };

        match NoDrop::new(result).transpose() {
            Ok(peeked) => peeked.into_inner(),
            // SAFETY: The caller ensures that it is always valid to peek ahead the cursor
            //         by `amount` elements.
            Err(error) => unsafe { error.into_inner().panic_unchecked() },
        }
    }

    /// Peek behind the slide by `amount` elements without any checks.
    ///
    /// # Returns
    ///
    /// Returns the peeked subslice.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation. Despite this method not doing any checks on release builds,
    /// it *does* actually do checks on debug builds to catch *undefined behavior*.
    ///
    /// Additionally, the caller needs to ensure that it is indeed valid to peek behind by `amount`
    /// elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary to avoid *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn peek_behind_unchecked(
        &self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to peek behind the cursor.
        let result = unsafe { self.try_peek_behind(amount) };

        match NoDrop::new(result).transpose() {
            Ok(peeked) => peeked.into_inner(),
            // SAFETY: The caller ensures that it is always valid to peek behind the cursor
            //         by `amount` elements.
            Err(error) => unsafe { error.into_inner().panic_unchecked() },
        }
    }

    /// Advance the slide by `amount` elements without any checks.
    ///
    /// # Returns
    ///
    /// Returns the advanced subslice.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation. Despite this method not doing any checks on release builds,
    /// it *does* actually do checks on debug builds to catch *undefined behavior*.
    ///
    /// Additionally, the caller needs to ensure that it is indeed valid to advance by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary to avoid *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn advance_unchecked(
        &mut self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to advance the cursor.
        let result = unsafe { self.try_advance(amount) };

        match NoDrop::new(result).transpose() {
            Ok(advanced) => advanced.into_inner(),
            // SAFETY: The caller ensures that it is always valid to advance the cursor
            //         by `amount` elements.
            Err(error) => unsafe { error.into_inner().panic_unchecked() },
        }
    }

    /// Rewind the slide by `amount` elements without any checks.
    ///
    /// # Returns
    ///
    /// Returns the rewound subslice.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that it is safe to create a temporary reference to the
    /// underlying buffer for validation. Despite this method not doing any checks on release builds,
    /// it *does* actually do checks on debug builds to catch *undefined behavior*.
    ///
    /// Additionally, the caller needs to ensure that it is indeed valid to rewind by `amount` elements.
    ///
    /// See the documentation for [`Slice::validate_split_at`] as implemented for `S` for details on
    /// what is considered an invalid split boundary to avoid *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn rewind_unchecked(
        &mut self,
        amount: usize,
    ) -> NonNull<S> {
        // SAFETY: The caller ensures that it is safe to create a temporary reference
        //         to validate whether we're able to advance the cursor.
        let result = unsafe { self.try_rewind(amount) };

        match NoDrop::new(result).transpose() {
            Ok(rewound) => rewound.into_inner(),
            // SAFETY: The caller ensures that it is always valid to rewind the cursor
            //         by `amount` elements.
            Err(error) => unsafe { error.into_inner().panic_unchecked() },
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
