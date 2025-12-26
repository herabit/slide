use core::{
    fmt, mem,
    ptr::{self, NonNull},
    slice,
};

use crate::macros::unreachable_unchecked;

methods! {
    /// Returns the provided string's length, in bytes.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn len(string: *const str) -> usize {
        (string as *const [u8]).len()
    }

    /// Returns whether the provided string is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn is_empty(string: *const str) -> bool {
        (string as *const [u8]).is_empty()
    }

    /// Create a raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice(data: *const u8, len: usize) -> *const str {
        ptr::slice_from_raw_parts(data, len) as *const str
    }

    /// Create a mutable raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_mut(data: *mut u8, len: usize) -> *mut str {
        ptr::slice_from_raw_parts_mut(data, len) as *mut str
    }

    /// Create a [`NonNull`] raw string slice pointer given a data pointer and length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn raw_slice_nonnull(data: NonNull<u8>, len: usize) -> NonNull<str> {
        // SAFETY: We know `data` to be nonnull, and we just want to bypass UB checks in debug builds as,
        //         well, they're quite annoying.
        //
        //         Does it matter? Not really. But fuck UB checks >:3
        unsafe { mem::transmute(raw_slice_mut(data.as_ptr(), len)) }
    }

    /// Create a shared string slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts['a](data: *const u8, len: usize) -> &'a str {
        // SAFETY: The caller ensures this is safe.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(data, len)) }
    }

    /// Create a mutable string slice reference given a data pointer and length.
    ///
    /// # Safety
    ///
    /// ***TODO***
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn from_raw_parts_mut['a](data: *mut u8, len: usize) -> &'a mut str {
        // SAFETY: The caller ensures this is safe.
        unsafe { str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(data, len)) }
    }
}

/// An error for attempting to get the inner elements of a [`prim@str`].
///
/// This only affects mutable borrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StrElemsError {
    /// It is not safe to get a mutable reference to the inner `[u8]` for a [`prim@str`].
    UnsafeToMutablyBorrow,
}

impl StrElemsError {
    /// Panics with an error message corresponding to `self`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn handle(&self) -> ! {
        match self {
            StrElemsError::UnsafeToMutablyBorrow => {
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
        match self {
            StrElemsError::UnsafeToMutablyBorrow => unsafe {
                unreachable_unchecked!(
                    "it is unsafe to mutably borrow the underlying bytes of a string"
                )
            },
        }
    }
}

impl fmt::Display for StrElemsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StrElemsError::UnsafeToMutablyBorrow => core::write!(
                f,
                "it is unsafe to mutably borrow the underlying bytes of a string"
            ),
        }
    }
}

impl core::error::Error for StrElemsError {}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrSplitError::InvalidCharBoundary { index } => core::write!(
                f,
                "the byte at index {index} is not a valid UTF-8 character boundary"
            ),
        }
    }
}

impl core::error::Error for StrSplitError {}
