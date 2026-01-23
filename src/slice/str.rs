use core::{
    cmp::Ordering,
    fmt, mem,
    num::NonZero,
    ptr::{self, NonNull},
    slice,
    str::Utf8Error,
};

use crate::{
    macros::unreachable_unchecked,
    slice::{
        AsElemsError, FromElemsError, OobIndex, Split, SplitError, SplitMut, split_error_handler,
    },
    str::is_utf8_char_boundary,
    util::cmp_usize,
};

methods! {
    /// Returns the provided string's length, in bytes.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn len(
        string: *const str,
    ) -> usize {
        (string as *const [u8]).len()
    }

    /// Returns whether the provided string is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn is_empty(
        string: *const str,
    ) -> bool {
        (string as *const [u8]).is_empty()
    }

    /// Create a raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice(
        data: *const u8,
        len: usize,
    ) -> *const str {
        ptr::slice_from_raw_parts(data, len) as *const str
    }

    /// Create a mutable raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_mut(
        data: *mut u8,
        len: usize,
    ) -> *mut str {
        ptr::slice_from_raw_parts_mut(data, len) as *mut str
    }

    /// Create a [`NonNull`] raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_nonnull(
        data: NonNull<u8>,
        len: usize,
    ) -> NonNull<str> {
        // SAFETY: We know `data` to be nonnull, and we just want to bypass UB checks in debug builds as,
        //         well, they're quite annoying.
        //
        //         Does it matter? Not really. But fuck UB checks >:3
        unsafe { mem::transmute(raw_slice_mut(data.as_ptr(), len)) }
    }

    // TODO: Write better docs.
    /// Create a shared string slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// In addition to the invariants described for [`slice::from_raw_parts`](::core::slice::from_raw_parts),
    /// where `T` is [`prim@u8`], the caller must ensure that the created byte slice is valid UTF-8.
    ///
    /// Failure to uphold these invariants is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts['a](
        data: *const u8,
        len: usize,
    ) -> &'a str {
        // SAFETY: The caller ensures this is safe.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(data, len)) }
    }

    // TODO: Write better docs.
    /// Create a mutable string slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// In addition to the invariants described for [`slice::from_raw_parts_mut`](::core::slice::from_raw_parts_mut),
    /// where `T` is [`prim@u8`]m the caller must ensure that the created byte slice is valid UTF-8.
    ///
    /// Failure to uphold these invariiants is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts_mut['a](
        data: *mut u8,
        len: usize,
    ) -> &'a mut str {
        // SAFETY: The caller ensures this is safe.
        unsafe { str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(data, len)) }
    }

    /// Try to create a string from a byte slice.
    ///
    /// # Returns
    ///
    /// Returns an error if the provided byte slice contains invalid UTF-8.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn try_from_elems['a](
        bytes: &'a [u8],
    ) -> Result<&'a str, FromElemsError<str>> {
        match <str>::from_utf8(bytes) {
            Ok(string) => Ok(string),
            Err(error) => Err(FromElemsError(error)),
        }
    }

    /// Try to create a mutable string from a mutable byte slice.
    ///
    /// # Returns
    ///
    /// Returns an error if the provided byte slice contains invalid UTF-8.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn try_from_elems_mut['a](
        bytes: &'a mut [u8],
    ) -> Result<&'a mut str, FromElemsError<str>> {
        match <str>::from_utf8_mut(bytes) {
            Ok(string) => Ok(string),
            Err(error) => Err(FromElemsError(error)),
        }
    }

    /// Create a string from a byte slice.
    ///
    /// # Panics
    ///
    /// Panics if the provided byte slice contains invalid UTF-8.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn from_elems['a](
        bytes: &'a [u8],
    ) -> &'a str {
        match try_from_elems(bytes) {
            Ok(string) => string,
            Err(error) => error.panic(),
        }
    }

    /// Create a mutable string from a mutable byte slice.
    ///
    /// # Panics
    ///
    /// Panics is the provided byte slice contains invalid UTF-8.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn from_elems_mut['a](
        bytes: &'a mut [u8],
    ) -> &'a mut str {
        match try_from_elems_mut(bytes) {
            Ok(string) => string,
            Err(error) => error.panic(),
        }
    }

    /// Create a string from a byte slice without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided byte slice is valid UTF-8.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_elems_unchecked['a](
        elems: &'a [u8],
    ) -> &'a str {
        if cfg!(not(debug_assertions)) {
            // SAFETY: The caller ensures this is sound.
            unsafe { <str>::from_utf8_unchecked(elems) }
        } else {
            match try_from_elems(elems) {
                Ok(string) => string,
                // SAFETY: The caller ensures this is sound.
                Err(error) => unsafe { error.panic_unchecked() },
            }
        }
    }

    /// Create a mutable string from a byte slice without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided byte slice is valid UTF-8.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_elems_mut_unchecked['a](
        elems: &'a mut [u8],
    ) -> &'a mut str {
        if cfg!(not(debug_assertions)) {
            // SAFETY: The caller ensures this is sound.
            unsafe { <str>::from_utf8_unchecked_mut(elems) }
        } else {
            match try_from_elems_mut(elems) {
                Ok(string) => string,
                // SAFETY: The caller ensures this is sound.
                Err(error) => unsafe { error.panic_unchecked() },
            }
        }
    }

    /// Try to get the underlying byte slice of the string.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(slice.as_bytes())`.
    #[inline(always)]
    pub(crate) const fn try_as_elems['a](
        slice: &'a str,
    ) -> Result<&'a [u8], AsElemsError<str>> {
        Ok(slice.as_bytes())
    }

    /// Get the underlying byte slice of the string.
    ///
    /// # Returns
    ///
    /// Always returns `slice.as_bytes()`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_elems['a](
        slice: &'a str,
    ) -> &'a [u8] {
        slice.as_bytes()
    }

    /// Get the underlying byte slice of the string without checks.
    ///
    /// # Returns
    ///
    /// Always returns `slice.as_bytes()`.
    ///
    /// # Safety
    ///
    /// It is always safe to call this.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn as_elems_unchecked['a](
        slice: &'a str,
    ) -> &'a [u8] {
        slice.as_bytes()
    }

    /// Try to get a mutable byte slice of the string.
    ///
    /// # Returns
    ///
    /// Always returns `Err(AsElemsError(StrAsElemsError::UnsafeToMutablyBorrow))`.
    #[inline(always)]
    pub(crate) const fn try_as_elems_mut['a](
        _: &'a mut str,
    ) -> Result<&'a mut [u8], AsElemsError<str>> {
        Err(AsElemsError(StrAsElemsError::UnsafeToMutablyBorrow))
    }

    /// Get a mutable byte slice of the string.
    ///
    /// # Panics
    ///
    /// This always panics, as it is *not* safe to mutably borrow the byte slice of a string.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn as_elems_mut['a](
        slice: &'a mut str,
    ) -> &'a mut [u8] {
        match try_as_elems_mut(slice) {
            Ok(s) => s,
            Err(error) => error.panic(),
        }
    }

    /// Get a mutable byte slice of the string without any checks.
    ///
    /// # Returns
    ///
    /// Always returns `slice.as_bytes_mut()`.
    ///
    /// # Safety
    ///
    /// The caller must esnure that the contents of the slice are valid UTF-8 before the borrow
    /// ends.
    ///
    /// Failure to ensure this may result in a string whose contents are invalid UTF-8,
    /// which is *undefined behavior*.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn as_elems_mut_unchecked['a](
        slice: &'a mut str,
    ) -> &'a mut [u8] {
        // SAFETY: The caller ensure this is sound.
        unsafe { slice.as_bytes_mut() }
    }

    /// Determines whether it is safe to split the provided [`prim@str`] at `index`.
    ///
    /// # Returns
    ///
    /// - `Ok(())` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    ///
    /// - `Err(SplitError::Other(StrSplitError::InvalidCharBoundary { index }))` if the
    ///   the byte at `index` is not a valid UTF-8 character boundary.
    ///
    /// # Safety
    ///
    /// The results of this function can be relied upon in `unsafe` code for ensuring
    /// that it is safe to split at `index`.
    #[inline(always)]
    pub(crate) const fn validate_split_at['a](
        slice: &'a str,
        index: usize,
    ) -> Result<(), SplitError<str>> {
        if index == 0 {
            // NOTE: It is always sound to get `&slice[0..]` and `&slice[..0]`.
            Ok(())
        } else {
            match cmp_usize(index, slice.len()) {
                // NOTE: It is always sound to get `&slice[..slice.len()]` and `&slice[slice.len()..]`.
                Ordering::Equal => Ok(()),

                // NOTE: If `index > slice.len()`, then it is out of bounds.
                Ordering::Greater => Err(SplitError::OutOfBounds {
                    index: NonZero::new(index as OobIndex).unwrap(),
                    len: slice.len(),
                }),

                // NOTE: If `index < slice.len()` and the byte at `index` is a valid UTF-8 character boundary,
                //       then it is always sound to get `&slice[..index]` and `&slice[index..]`.
                Ordering::Less if is_utf8_char_boundary(slice.as_bytes()[index]) => Ok(()),

                // NOTE: If `index < slice.len()` and the byte at `index` is not a valid UTF-8 character boundary,
                //       then it is *not* sound to get `&slice[..index]` and `&slice[index..]`.
                Ordering::Less => Err(SplitError::Other(StrSplitError::InvalidCharBoundary {
                    index,
                })),
            }
        }
    }


    /// Split a string at the given index without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following, or *undefined behavior* will be invoked:
    ///
    /// - That `index` is within bounds (`index <= len`).
    ///
    /// - That the byte at `index`, if any, is a valid UTF-8 character boundary.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_at_unchecked['a](
        slice: &'a str,
        index: usize,
    ) -> Split<'a, str> {
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

    /// Split a mutable string at the given index without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure the following, or *undefined behavior* will be invoked:
    ///
    /// - That `index` is within bounds (`index <= len`).
    ///
    /// - That the byte at `index`, if any, is a valid UTF-8 character boundary.
    ///
    /// # Returns
    ///
    /// Return a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn split_at_mut_unchecked['a](
        slice: &'a mut str,
        index: usize,
    ) -> SplitMut<'a, str> {
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

    /// Try to split a string at the given index.
    ///
    /// # Returns
    ///
    /// - `Ok((head, tail))` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    ///
    /// - `Err(SplitError::Other(StrSplitError::InvalidCharBoundary { index }))` if the
    ///   the byte at `index`, if any, is not a valid UTF-8 character boundary.
    #[inline(always)]
    pub(crate) const fn try_split_at['a](
        slice: &'a str,
        index: usize,
    ) -> Result<Split<'a, str>, SplitError<str>> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => Ok(unsafe { split_at_unchecked(slice, index) }),
            Err(error) => Err(error),
        }
    }

    /// Try to split a mutable string at the given index.
    ///
    /// # Returns
    ///
    /// - `Ok((head, tail))` upon success.
    ///
    /// - `Err(SplitError::OutOfBounds { index, len })` if `index` is out of bounds (`index > len`).
    ///
    /// - `Err(SplitError::Other(StrSplitError::InvalidCharBoundary { index }))` if the
    ///   the byte at `index`, if any, is not a valid UTF-8 character boundary.
    #[inline(always)]
    pub(crate) const fn try_split_at_mut['a](
        slice: &'a mut str,
        index: usize,
    ) -> Result<SplitMut<'a, str>, SplitError<str>> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => Ok(unsafe { split_at_mut_unchecked(slice, index) }),
            Err(error) => Err(error),
        }
    }

    /// Split a string at the given index.
    ///
    /// # Panics
    ///
    /// - If `index` is out of bounds (`index > len`).
    ///
    /// - If the byte at `index`, if any, is not a valid UTF-8 character boundary.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn split_at['a](
        slice: &'a str,
        index: usize,
    ) -> Split<'a, str> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe { split_at_unchecked(slice, index) },
            // SAFETY: We know that it is invalid to split at `index`.
            Err(..) => unsafe { split_error_handler(slice, NonZero::new(index as OobIndex).unwrap()) },
        }
    }

    /// Split a mutable string at the given index.
    ///
    /// # Panics
    ///
    /// - If `index` is out of bounds (`index > len`).
    ///
    /// - If the byte at `index`, if any, is not a valid UTF-8 character boundary.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(head, tail)`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn split_at_mut['a](
        slice: &'a mut str,
        index: usize,
    ) -> SplitMut<'a, str> {
        match validate_split_at(slice, index) {
            // SAFETY: We just checked that it is safe to split above.
            Ok(()) => unsafe { split_at_mut_unchecked(slice, index) },
            // SAFETY: We know that it is invalid to split at `index`.
            Err(..) => unsafe { split_error_handler(slice, NonZero::new(index as OobIndex).unwrap()) },
        }
    }

    /// Panics with an error message for the given [`Utf8Error`](::core::str::Utf8Error).
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle_from_elems_error(
        _: Utf8Error,
    ) -> ! {
        panic!("provided string contains invalid UTF-8")
    }

    /// Marks the code path that produced the given [`Utf8Error`](::core::str::Utf8Error) as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for the given error to have been produced. Failure
    /// to do so is *undefined behavior* as calling this function makes promises to the compiler that change
    /// what optimizations are valid.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn handle_from_elems_error_unchecked(
        _: Utf8Error,
    ) -> ! {
        // SAFETY: The caller ensures that this code path is impossible to reach.
        unsafe { unreachable_unchecked!("provided string contains invalid UTF-8") }
    }

    /// Panics with an error message for the given [`StrAsElemsError`](crate::str::StrAsElemsError).
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle_as_elems_error(
        error: StrAsElemsError,
    ) -> ! {
        error.handle()
    }

    /// Marks the code path that produced the given [`StrAsElemsError`](crate::str::StrAsElemsError)
    /// as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for the given error to have been produced. Failure
    /// to do so is *undefined behavior* as calling this function makes promises to the compiler that change
    /// what optimizations are valid.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn handle_as_elems_error_unchecked(
        error: StrAsElemsError,
    ) -> ! {
        // SAFETY: The caller ensures that this code path is impossible to reach.
        unsafe { error.handle_unchecked() }
    }

    /// Panics with an error message for the given [`StrSplitError`](crate::str::StrSplitError).
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle_split_error(error: StrSplitError) -> ! {
        error.handle()
    }

    /// Marks the code path that produced the given [`StrSplitError`](crate::str::StrSplitError)
    /// as impossible.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure that it is impossible for the given error to have been produced. Failure
    /// to do so is *undefined behavior* as calling this function makes promises to the compiler that change
    /// what optimizations are valid.
    ///
    /// Proceed with caution.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn handle_split_error_unchecked(
        error: StrSplitError,
    ) -> ! {
        // SAFETY: The caller ensures that this code path is impossible to reach.
        unsafe { error.handle_unchecked() }
    }

    /// Gets the index of the provided [`StrSplitError`](crate::str::StrSplitError).
    #[inline(always)]
    pub(crate) const fn split_error_index['a](
        error: &'a StrSplitError,
    ) -> usize {
        error.index()
    }
}

/// An error for attempting to get the inner elements of a [`prim@str`].
///
/// This only affects mutable borrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StrAsElemsError {
    /// It is not safe to get a mutable reference to the inner `[u8]` for a [`prim@str`].
    UnsafeToMutablyBorrow,
}

impl StrAsElemsError {
    /// Panics with an error message corresponding to `self`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle(&self) -> ! {
        match self {
            StrAsElemsError::UnsafeToMutablyBorrow => {
                panic!("it is unsafe to mutably borrow the underlying bytes of a string")
            }
        }
    }

    /// Marks the code path that created this error as impossible.
    ///
    /// # Safety
    ///
    /// The caller ***must ensure*** that this is impossible to reach.
    /// Otherwise, undefined behavior will be invoked.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn handle_unchecked(&self) -> ! {
        // SAFETY: The caller ensures this is sound.
        match self {
            StrAsElemsError::UnsafeToMutablyBorrow => unsafe {
                unreachable_unchecked!(
                    "it is unsafe to mutably borrow the underlying bytes of a string"
                )
            },
        }
    }
}

impl fmt::Display for StrAsElemsError {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            StrAsElemsError::UnsafeToMutablyBorrow => core::write!(
                f,
                "it is unsafe to mutably borrow the underlying bytes of a string"
            ),
        }
    }
}

impl core::error::Error for StrAsElemsError {
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "it is unsafe to mutably borrow the underlying bytes of a string"
    }
}

/// An error that occurs when trying to split a [`prim@str`].
///
/// This does *not* include out-of-bounds errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StrSplitError {
    /// The byte at `index` was not a valid UTF-8 character boundary.
    InvalidCharBoundary { index: usize },
}

impl StrSplitError {
    /// Returns the index that caused the error.
    #[inline(always)]
    #[must_use]
    pub const fn index(&self) -> usize {
        match self {
            StrSplitError::InvalidCharBoundary { index } => *index,
        }
    }

    /// Panics with an error message corresponding to `self`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle(&self) -> ! {
        match self {
            StrSplitError::InvalidCharBoundary { .. } => {
                panic!("the byte at `index` is not a valid UTF-8 character boundary")
            }
        }
    }

    /// Marks the code path that created this error as impossible.
    ///
    /// # Safety
    ///
    /// The caller ***must ensure*** that this is impossible to reach.
    /// Otherwise, undefined behavior will be invoked.
    #[inline(always)]
    #[track_caller]
    pub(crate) const unsafe fn handle_unchecked(&self) -> ! {
        // SAFETY: The caller ensures this is sound.
        match self {
            StrSplitError::InvalidCharBoundary { .. } => unsafe {
                unreachable_unchecked!(
                    "the byte at `index` is not a valid UTF-8 character boundary"
                )
            },
        }
    }
}

impl fmt::Display for StrSplitError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            StrSplitError::InvalidCharBoundary { index } => core::write!(
                f,
                "the byte at index {index} is not a valid UTF-8 character boundary"
            ),
        }
    }
}

impl core::error::Error for StrSplitError {
    #[inline]
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "the byte at the specified index is not a valid UTF-8 character boundary"
    }
}
