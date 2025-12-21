methods! {
    /// Returns the provided string's length, in bytes.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn len(string: *const str) -> usize {
        (string as *const [u8]).len()
    }

    /// Returns whether the provided string is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_empty(string: *const str) -> bool {
        (string as *const [u8]).is_empty()
    }
}
