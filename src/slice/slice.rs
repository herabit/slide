use core::{
    convert::Infallible,
    ptr::{self, NonNull},
    slice,
};

methods! {
    /// Returns the provided slice's length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn len[T](slice: *const [T]) -> usize {
        slice.len()
    }

    /// Returns whether the provided slice is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn is_empty[T](slice: *const [T]) -> bool {
        slice.is_empty()
    }

    /// Create a raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice[T](data: *const T, len: usize) -> *const [T] {
        ptr::slice_from_raw_parts(data, len)
    }

    /// Create a mutable raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_mut[T](data: *mut T, len: usize) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(data, len)
    }

    /// Create a [`NonNull`] raw slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_nonnull[T](data: NonNull<T>, len: usize) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(data, len)
    }

    /// Create a shared slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts['a, T](data: *const T, len: usize) -> &'a [T] {
        // SAFETY: The caller ensures this is safe.
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Create a mutable slice reference given a data pointer a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts_mut['a, T](data: *mut T, len: usize) -> &'a mut [T] {
        // SAFETY: The caller ensures this is safe.
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_from_elems_error(error: Infallible) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_from_elems_error_unchecked(error: Infallible) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_as_elems_error(error: Infallible) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_as_elems_error_unchecked(error: Infallible) -> ! {
        match error {}
    }

    /// This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const fn handle_split_error(error: Infallible) -> ! {
        match error {}
    }

    // This function is impossible to call. Don't even try.
    #[inline(always)]
    pub(crate) const unsafe fn handle_split_error_unchecked(error: Infallible) -> ! {
        match error {}
    }
}
