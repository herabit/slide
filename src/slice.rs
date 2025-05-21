/// Internal implementation details.
pub(crate) mod private;

/// Marker trait the kinds of slices this crate can work with.
pub unsafe trait Slice: private::Sealed {
    /// An associated item that details what the underlying items of this
    /// slice, are.
    ///
    /// The length of this slice is equivalent to the amount of `Self::Item`s
    /// stored within.
    ///
    /// # Validity
    ///
    /// Just because a slice indicates it in memory is a `[Self::Item]`, that does
    /// ***not*** mean that it is a transparent wrapper over `[Self::Item]`.
    ///
    /// Additionally, not all `[Self::Item]`s may be valid instances of this slice.
    /// Examples include [`str`], which in memory is a `[u8]` slice, but has the
    /// added invariant that the underlying slice is valid UTF-8.
    ///
    /// If you need to convert between some `[Self::Item]` and this slice type,
    /// either use the fallible methods or the unsafe methods to do so.
    ///
    /// # Safety
    ///
    /// The underlying memory for a slice must be a valid `[Self::Item]`. No exceptions.
    type Item: Sized;

    /// An error that occurs when trying to decode this slice from a `[Self::Item]`.
    type DecodeError: Sized;

    // A type witness to allow const polymorphism.
    #[doc(hidden)]
    const KIND: private::SliceKind<Self>;
}
