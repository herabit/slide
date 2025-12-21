methods! {
    /// Returns the provided slice's length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn len[T](slice: *const [T]) -> usize {
        slice.len()
    }

    /// Returns whether the provided slice is empty.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub const fn is_empty[T](slice: *const [T]) -> bool {
        slice.is_empty()
    }
}
