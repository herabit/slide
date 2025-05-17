use core::marker::PhantomData;

/// Marker type for marking a parameter as covariant.
#[repr(transparent)]
pub(crate) struct Covariant<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> Covariant<T> {
    /// Create a new [`Covariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Clone for Covariant<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Covariant<T> {}

/// Marker type for marking a parameter as contravariant.
#[repr(transparent)]
pub(crate) struct Contravariant<T: ?Sized>(PhantomData<fn(T)>);

impl<T: ?Sized> Contravariant<T> {
    /// Create a new [`Contravariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Clone for Contravariant<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Contravariant<T> {}

/// Marker type for marking a parameter as invariant.
#[repr(transparent)]
pub(crate) struct Invariant<T: ?Sized>(PhantomData<fn(T) -> T>);

impl<T: ?Sized> Invariant<T> {
    /// Create a new [`Invariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Clone for Invariant<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Invariant<T> {}
