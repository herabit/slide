/// The implementation details of the [`Slice`] trait.
pub(crate) mod sealed;

/// Trait for the various supported slices.
pub trait Slice: sealed::Sealed {}

impl<S: sealed::Sealed + ?Sized> Slice for S {}
