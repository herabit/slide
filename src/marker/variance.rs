use core::marker::PhantomData;

/// Marker type for marking a parameter as covariant.
#[repr(transparent)]
pub(crate) struct Covariant<T>(PhantomData<fn() -> T>)
where
    T: ?Sized;

impl<T> Covariant<T>
where
    T: ?Sized,
{
    /// Create a new [`Covariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Clone for Covariant<T>
where
    T: ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Covariant<T> where T: ?Sized {}

/// Marker type for marking a parameter as contravariant.
#[repr(transparent)]
pub(crate) struct Contravariant<T>(PhantomData<fn(T)>)
where
    T: ?Sized;

impl<T> Contravariant<T>
where
    T: ?Sized,
{
    /// Create a new [`Contravariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Clone for Contravariant<T>
where
    T: ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Contravariant<T> where T: ?Sized {}

/// Marker type for marking a parameter as invariant.
#[repr(transparent)]
pub(crate) struct Invariant<T>(PhantomData<fn(T) -> T>)
where
    T: ?Sized;

impl<T> Invariant<T>
where
    T: ?Sized,
{
    /// Create a new [`Invariant`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Clone for Invariant<T>
where
    T: ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Invariant<T> where T: ?Sized {}
