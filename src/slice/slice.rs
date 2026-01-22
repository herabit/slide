use core::{
    cmp::Ordering,
    convert::Infallible,
    num::NonZero,
    ptr::{self, NonNull},
    slice,
};

use crate::{
    slice::{
        AsElemsError, FromElemsError, OobIndex, Split, SplitError, SplitMut, split_error_handler,
    },
    util::cmp_usize,
};

methods! {
    /// Returns the provided slice's length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn len[T](
        slice: *const [T],
    ) -> usize {
        slice.len()
    }

    /// Returns whether the provided slice is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn is_empty[T](
        slice: *const [T],
    ) -> bool {
        slice.is_empty()
    }

    /// Create a raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice[T](
        data: *const T,
        len: usize,
    ) -> *const [T] {
        ptr::slice_from_raw_parts(data, len)
    }

    /// Create a mutable raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_mut[T](
        data: *mut T,
        len: usize,
    ) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(data, len)
    }

    /// Create a [`NonNull`] raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_nonnull[T](
        data: NonNull<T>,
        len: usize,
    ) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(data, len)
    }

    /// Create a shared slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// See [`core::slice::from_raw_parts`].
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts['a, T](
        data: *const T,
        len: usize,
    ) -> &'a [T] {
        // SAFETY: The caller ensures this is safe.
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Create a mutable slice reference given a data pointer a data pointer and length.
    ///
    /// # Safety
    ///
    /// See [`core::slice::from_raw_parts_mut`].
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts_mut['a, T](
        data: *mut T,
        len: usize,
    ) -> &'a mut [T] {
        // SAFETY: The caller ensures this is safe.
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    /// Try to create a slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(elems)`.
    #[inline(always)]
    pub(crate) const fn try_from_elems['a, T](
        elems: &'a [T],
    ) -> Result<&'a [T], FromElemsError<[T]>> {
        Ok(elems)
    }

    /// Create a slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `elems`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_elems['a, T](
        elems: &'a [T],
    ) -> &'a [T] {
        elems
    }

    /// Create a mutably slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `elems`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn from_elems_mut['a, T](
        elems: &'a mut [T],
    ) -> &'a mut [T] {
        elems
    }

    /// Create a slice from itself without any checks.
    ///
    /// # Returns
    ///
    /// Always returns `elems`.
    ///
    /// # Safety
    ///
    /// This is always safe to call.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn from_elems_unchecked['a, T](
        elems: &'a [T],
    ) -> &'a [T] {
        elems
    }

    /// Create a mutable slice from itself without any checks.
    ///
    /// # Returns
    ///
    /// Always returns `elems`.
    ///
    /// # Safety
    ///
    /// This is always safe to call.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn from_elems_mut_unchecked['a, T](
        elems: &'a mut [T],
    ) -> &'a mut [T] {
        elems
    }

    /// Try to create a mutable slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(elems)`.
    #[inline(always)]
    pub(crate) const fn try_from_elems_mut['a, T](
        elems: &'a mut [T],
    ) -> Result<&'a mut [T], FromElemsError<[T]>> {
        Ok(elems)
    }

    /// Try to create a slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(slice)`.
    #[inline(always)]
    pub(crate) const fn try_as_elems['a, T](
        slice: &'a [T],
    ) -> Result<&'a [T], AsElemsError<[T]>> {
        Ok(slice)
    }

    /// Create a slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `slice`.
    #[inline(always)]
    pub(crate) const fn as_elems['a, T](
        slice: &'a [T],
    ) -> &'a [T] {
        slice
    }

    /// Create a slice from itself without any checks.
    ///
    /// # Returns
    ///
    /// Always returns `slice`.
    ///
    /// # Safety
    ///
    /// This is always safe to call.
    #[inline(always)]
    pub(crate) const unsafe fn as_elems_unchecked['a, T](
        slice: &'a [T],
    ) -> &'a [T] {
        slice
    }

    /// Try to create a mutable slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(slice)`.
    #[inline(always)]
    pub(crate) const fn try_as_elems_mut['a, T](
        slice: &'a mut [T],
    ) -> Result<&'a mut [T], AsElemsError<[T]>> {
        Ok(slice)
    }

    /// Create a mutable slice from itself.
    ///
    /// # Returns
    ///
    /// Always returns `slice`.
    #[inline(always)]
    pub(crate) const fn as_elems_mut['a, T](
        slice: &'a mut [T],
    ) -> &'a mut [T] {
        slice
    }

    /// Create a mutable slice from itself without any checks.
    ///
    /// # Returns
    ///
    /// Always returns `slice`.
    ///
    /// # Safety
    ///
    /// This is always safe to call.
    #[inline(always)]
    pub(crate) const unsafe fn as_elems_mut_unchecked['a, T](
        slice: &'a mut [T],
    ) -> &'a mut [T] {
        slice
    }

    /// Determines whether it is safe to split the provided slice at `index`.
    ///
    /// # Returns
    ///
    /// - `Ok(())` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    ///
    /// # Safety
    ///
    /// The results of this function can be relied upon in `unsafe` code for ensuring
    /// that it is safe to split at `index`.
    #[inline(always)]
    pub(crate) const fn validate_split_at['a, T](
        slice: &'a [T],
        index: usize,
    ) -> Result<(), SplitError<[T]>> {
        match cmp_usize(index, slice.len()) {
            // NOTE: If `index <= slice.len()`, then it is always valid to get `&slice[..index]` and `&slice[index..]`.
            Ordering::Less | Ordering::Equal => Ok(()),
            // NOTE: If `index > slice.len()`, then it is out of bounds.
            Ordering::Greater => Err(SplitError::OutOfBounds {
                index: NonZero::new(index as OobIndex).unwrap(),
                len: slice.len(),
            }),
        }
    }

    /// Split a slice at the given index without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following, or *undefined behavior* will be invoked:
    ///
    /// - That `index` is within bounds (`index <= len`).
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_at_unchecked['a, T](
        slice: &'a [T],
        index: usize,
    ) -> Split<'a, [T]> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe {
                let len = slice.len();
                let ptr = slice.as_ptr();

                let head = from_raw_parts(ptr, index);
                let tail = from_raw_parts(ptr.add(index), len.unchecked_sub(index));

                (head, tail)
            },
            // SAFETY: The caller ensures that it is valid to split `slice` at `index`.
            Err(error) => unsafe { error.panic_unchecked() },
        }
    }

    /// Split a mutable slice at the given index without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following, or *undefined behavior* will be invoked:
    ///
    /// - That `index` is within bounds (`index <= len`).
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_at_mut_unchecked['a, T](
        slice: &'a mut [T],
        index: usize,
    ) -> SplitMut<'a, [T]> {
        match validate_split_at(&*slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe {
                let len = slice.len();
                let ptr = slice.as_mut_ptr();

                let head = from_raw_parts_mut(ptr, index);
                let tail = from_raw_parts_mut(ptr.add(index), len.unchecked_sub(index));

                (head, tail)
            },
            // SAFETY: The caller ensures that it is valid to split `slice` at `index`.
            Err(error) => unsafe { error.panic_unchecked() },
        }
    }

    /// Try to split a slice at the given index.
    ///
    /// # Returns
    ///
    /// - `Ok((head, tail))` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    #[inline(always)]
    pub(crate) const fn try_split_at['a, T](
        slice: &'a [T],
        index: usize,
    ) -> Result<Split<'a, [T]>, SplitError<[T]>> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => Ok(unsafe { split_at_unchecked(slice, index) }),
            Err(error) => Err(error),
        }
    }

    /// Try to split a mutable slice at the given index.
    ///
    /// # Returns
    ///
    /// - `Ok((head, tail))` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    #[inline(always)]
    pub(crate) const fn try_split_at_mut['a, T](
        slice: &'a mut [T],
        index: usize,
    ) -> Result<SplitMut<'a, [T]>, SplitError<[T]>> {
        match validate_split_at(&*slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => Ok(unsafe { split_at_mut_unchecked(slice, index) }),
            Err(error) => Err(error),
        }
    }

    /// Split a slice at the given index.
    ///
    /// # Panics
    ///
    /// - If `index` is out of bounds (`index > len`).
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn split_at['a, T](
        slice: &'a [T],
        index: usize,
    ) -> Split<'a, [T]> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe { split_at_unchecked(slice, index) },
            // SAFETY: We know that it is invalid to split at `index`.
            Err(..) => unsafe { split_error_handler(slice, index) },
        }
    }

    /// Split a mutable slice at the given index.
    ///
    ///
    /// # Panics
    ///
    /// - If `index` is out of bounds (`index > len`).
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn split_at_mut['a, T](
        slice: &'a mut [T],
        index: usize,
    ) -> SplitMut<'a, [T]> {
        match validate_split_at(&*slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe { split_at_mut_unchecked(slice, index) },
            // SAFETY: We know that it is invalid to split at `index`.
            Err(..) => unsafe { split_error_handler(slice, index) },
        }
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_from_elems_error(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_from_elems_error_unchecked(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_as_elems_error(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_as_elems_error_unchecked(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_split_error(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    // This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_split_error_unchecked(
        error: Infallible,
    ) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn split_error_index['a](
        error: &'a Infallible,
    ) -> usize {
        match *error {}
    }
}
