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
    //
    // Downstream crates must not rely on this existing. This is purely an implementation
    // detail, and it's removal is not considered a breaking change.
    #[doc(hidden)]
    const KIND: private::SliceKind<Self>;
}

/// Marker trait for slices that can safely be accessed immutably as a slice
/// of their inner elements.
///
/// # Safety
///
/// Types such as `[T]`, [`str`], are examples where it is safe to access
/// their internal buffer immutably without fear of it invalidating the
/// invariants of the type.
///
/// Types that impose additional invariants upon their elements, and said elements
/// contain interior mutability that permits invalidating those invariants,
/// ***must not*** implement this trait.
pub unsafe trait AsElems: Slice {}

/// Marker trait for slices that can safely be accessed mutably as a slice
/// of their inner elements.
///
/// # Safety
///
/// Types such as `[T]` is an example where it is safe to access their internal buffer
/// mutably without fear of it invalidating the invariants of the type.
///
/// Types that impose additional invariants upon their elements, and said elements
/// can be modified in such a manner that invalidates those invariants, ***must not***
/// implement this trait.
///
/// An example of a type where this is the case is [`str`], which while just a byte slice
/// in memory, it is one that ***must*** be valid UTF-8. If you were to safely get access
/// to a `&mut [u8]` from a `&mut str`, it would be undefined behavior as one could modify
/// the `&mut [u8]` in such a way that results in it no longer being UTF-8, resulting in undefined
/// behavior.
pub unsafe trait AsElemsMut: AsElems {}
