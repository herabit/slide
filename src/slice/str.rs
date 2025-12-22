use core::{
    mem,
    ptr::{self, NonNull},
    slice,
};

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
