use core::{fmt, ptr::NonNull};

/// Internal implementation details.
pub(crate) mod private;

/// Marker trait the kinds of slices this crate can work with.
pub unsafe trait Slice: private::Sealed {
    /// An associated item that details what the underlying items of this
    /// slice, are.
    ///
    /// The length of this slice is equivalent to the amount of `Self::Elem`s
    /// stored within.
    ///
    /// # Safety
    ///
    /// The underlying memory for a slice must be a valid `[Self::Elem]`. No exceptions.
    ///
    /// This means that the underlying memory of this slice type must be: initialized
    /// and properly aligned `Self::Elem`s.
    ///
    /// # Validity
    ///
    /// Not all `[Self::Elem]`s may be valid a valid `Self`.
    type Elem: Sized;

    /// An error that occurs when trying to decode this slice from a `[Self::Item]`.
    type DecodeError: Sized + fmt::Debug + fmt::Display;

    // A type witness to allow const polymorphism.
    #[doc(hidden)]
    const KIND: private::SliceKind<Self>;

    /// Returns the length of a this slice.
    ///
    /// This is equivalent to the amount of elements it contains.
    ///
    /// See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn len(&self) -> usize {
        len(self)
    }

    /// Attempt to decode a slice from a slice of its elements.
    ///
    /// # Returns
    ///
    /// Returns an error if decoding failed.
    ///
    /// See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn try_from_elems(elems: &[Self::Elem]) -> Result<&Self, Self::DecodeError> {
        try_from_elems(elems)
    }

    /// Decode a slice from a slice of its elements without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to construct an `&Self` from the provided
    /// elements.
    ///
    /// See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_unchecked(elems: &[Self::Elem]) -> &Self {
        // SAFETY: The caller ensures this is safe.
        unsafe { from_elems_unchecked(elems) }
    }

    /// Attempt to decode a mutable slice from a mutable slice of its elements.
    ///
    /// # Returns
    ///
    /// Returns an error if decoding failed.
    ///
    /// See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn try_from_elems_mut(elems: &mut [Self::Elem]) -> Result<&mut Self, Self::DecodeError> {
        try_from_elems_mut(elems)
    }

    /// Decode a mutable slice from a mutable slice of its elements without
    /// any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to construct an `&mut Self` from the provided
    /// elements.
    ///
    /// See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_mut_unchecked(elems: &mut [Self::Elem]) -> &mut Self {
        // SAFETY: The caller ensures this is safe.
        unsafe { from_elems_mut_unchecked(elems) }
    }

    /// Create a raw slice from a pointer and a length.
    ///
    /// The length is the amount of elements the slice contains.
    ///
    /// # Safety
    ///
    /// This is always safe, but dereferencing the resulting value is not.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn raw(data: *const Self::Elem, len: usize) -> *const Self {
        raw_slice(data, len)
    }

    /// Create a mutable raw slice from a pointer and a length.
    ///
    /// The length is the amount of elements the slice contains.
    ///
    /// # Safety
    ///
    /// This is always safe, but dereferencing the resulting value is not.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn raw_mut(data: *mut Self::Elem, len: usize) -> *mut Self {
        raw_slice_mut(data, len)
    }

    /// Create a [`NonNull`] raw slice from a pointer and a length.
    ///
    /// The length is the amount of elements the slice contains.
    ///
    /// # Safety
    ///
    /// This is always safe, but dereferencing the resulting value is not.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    fn raw_nonnull(data: NonNull<Self::Elem>, len: usize) -> NonNull<Self> {
        raw_slice_nonnull(data, len)
    }

    /// Create a slice from a pointer and a length.
    ///
    /// The length is the amount of elements the slice contains.
    ///
    /// # Safety
    ///
    /// It is undefined behavior if:
    ///
    /// - Any of the conditions for [`core::slice::from_raw_parts`] are violated
    ///   for `&'a [Self::Elem]`.
    ///
    /// - If the resulting `&'a [Self::Elem]` is not a valid `&'a Self`.
    ///
    /// - See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_raw_parts<'a>(data: *const Self::Elem, len: usize) -> &'a Self {
        // SAFETY: The caller ensures this is safe.
        unsafe { from_raw_parts(data, len) }
    }

    /// Create a mutable slice from a pointer and a length.
    ///
    ///  The length is the amount of elements the slice contains.
    ///
    /// # Safety
    ///
    /// It is undefined behavior if:
    ///
    /// - Any of the conditions for [`core::slice::from_raw_parts_mut`] are violated
    ///   for `&'a mut [Self::Elem]`.
    ///
    /// - If the resulting `&'a mut [Self::Elem]` is not a valid `&'a mut Self`.
    ///
    /// - See the implementation of [`Slice`] for `Self` for more information.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_raw_parts_mut<'a>(data: *mut Self::Elem, len: usize) -> &'a mut Self {
        // SAFETY: The caller ensures this is safe.
        unsafe { from_raw_parts_mut(data, len) }
    }
}

/// Returns the length of a given slice pointer.
///
/// This is equivalent to the amount of elements it contains.
///
/// See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn len<S: Slice + ?Sized>(slice: *const S) -> usize {
    S::KIND.0.len(slice)
}

/// Attempt to decode a slice from a slice of its elements.
///
/// # Returns
///
/// Returns an error if decoding failed.
///
/// See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn try_from_elems<S: Slice + ?Sized>(elems: &[S::Elem]) -> Result<&S, S::DecodeError> {
    S::KIND.0.decode_elems(elems)
}

/// Decode a slice from a slice of its elements without any checks.
///
/// # Safety
///
/// The caller must ensure that it is safe to construct an `&S` from the provided
/// elements.
///
/// See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_elems_unchecked<S: Slice + ?Sized>(elems: &[S::Elem]) -> &S {
    // SAFETY: The caller ensures this is safe.
    unsafe { S::KIND.0.decode_elems_unchecked(elems) }
}

/// Attempt to decode a mutable slice from a mutable slice of its elements.
///
/// # Returns
///
/// Returns an error if decoding failed.
///
/// See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn try_from_elems_mut<S: Slice + ?Sized>(
    elems: &mut [S::Elem],
) -> Result<&mut S, S::DecodeError> {
    S::KIND.0.decode_elems_mut(elems)
}

/// Decode a mutable slice from a mutable slice of its elements without
/// any checks.
///
/// # Safety
///
/// The caller must ensure that it is safe to construct an `&mut S` from the provided
/// elements.
///
/// See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_elems_mut_unchecked<S: Slice + ?Sized>(elems: &mut [S::Elem]) -> &mut S {
    // SAFETY: The caller ensures this is safe.
    unsafe { S::KIND.0.decode_elems_mut_unchecked(elems) }
}

/// Create a raw slice from a pointer and a length.
///
/// The length is the amount of elements the slice contains.
///
/// # Safety
///
/// This is always safe, but dereferencing the resulting value is not.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn raw_slice<S: Slice + ?Sized>(data: *const S::Elem, len: usize) -> *const S {
    S::KIND.0.raw_slice(data, len)
}

/// Create a mutable raw slice from a pointer and a length.
///
/// The length is the amount of elements the slice contains.
///
/// # Safety
///
/// This is always safe, but dereferencing the resulting value is not.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn raw_slice_mut<S: Slice + ?Sized>(data: *mut S::Elem, len: usize) -> *mut S {
    S::KIND.0.raw_slice_mut(data, len)
}

/// Create a [`NonNull`] raw slice from a pointer and a length.
///
/// The length is the amount of elements the slice contains.
///
/// # Safety
///
/// This is always safe, but dereferencing the resulting value is not.
#[inline(always)]
#[must_use]
#[track_caller]
pub const fn raw_slice_nonnull<S: Slice + ?Sized>(
    data: NonNull<S::Elem>,
    len: usize,
) -> NonNull<S> {
    S::KIND.0.raw_slice_nonnull(data, len)
}

/// Create a slice from a pointer and a length.
///
/// The length is the amount of elements the slice contains.
///
/// # Safety
///
/// It is undefined behavior if:
///
/// - Any of the conditions for [`core::slice::from_raw_parts`] are violated
///   for `&'a [S::Elem]`.
///
/// - If the resulting `&'a [S::Elem]` is not a valid `&'a S`.
///
/// - See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_raw_parts<'a, S: Slice + ?Sized>(
    data: *const S::Elem,
    len: usize,
) -> &'a S {
    // SAFETY: The caller ensures this is safe.
    unsafe { S::KIND.0.from_raw_parts(data, len) }
}

/// Create a mutable slice from a pointer and a length.
///
///  The length is the amount of elements the slice contains.
///
/// # Safety
///
/// It is undefined behavior if:
///
/// - Any of the conditions for [`core::slice::from_raw_parts_mut`] are violated
///   for `&'a mut [S::Elem]`.
///
/// - If the resulting `&'a mut [S::Elem]` is not a valid `&'a mut S`.
///
/// - See the implementation of [`Slice`] for `S` for more information.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_raw_parts_mut<'a, S: Slice + ?Sized>(
    data: *mut S::Elem,
    len: usize,
) -> &'a mut S {
    // SAFETY: The caller ensures this is safe.
    unsafe { S::KIND.0.from_raw_parts_mut(data, len) }
}
