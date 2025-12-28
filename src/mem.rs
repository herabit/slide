#![allow(dead_code)]

use crate::macros::assert_unchecked;
use core::{mem::ManuallyDrop, ptr};

/// An even unsafer version of [`mem::transmute`].
///
/// # Safety
///
/// In addition to the invariants of [`mem::transmute`],
/// the caller must ensure that `A` and `B` are the same size.
///
/// Failure to do so is immediate undefined behavior.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn transmute_unchecked<A, B>(a: A) -> B {
    #[allow(clippy::missing_docs_in_private_items)]
    #[repr(C)]
    union Transmute<A, B> {
        a: ManuallyDrop<A>,
        b: ManuallyDrop<B>,
    }

    unsafe {
        // SAFETY: The caller ensures that `A` and `B` are the same size.
        assert_unchecked!(size_of::<A>() == size_of::<B>(), "size mismatch");

        // SAFETY: The caller ensures that the transmute is safe.
        ManuallyDrop::into_inner(
            Transmute::<A, B> {
                a: ManuallyDrop::new(a),
            }
            .b,
        )
    }
}

/// A version of [`mem::transmute`] that plays better with generics.
///
/// # Safety
///
/// See [`mem::transmute`].
///
/// # Panics
///
/// Panics if `A` and `B` are not the same size.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn transmute<A, B>(a: A) -> B {
    assert!(
        size_of::<A>() == size_of::<B>(),
        "transmutes must be between types of equal size"
    );

    // SAFETY: The caller ensures the transmute is sound, and we know that `A` and `B`
    //         are of equal size.
    unsafe { transmute_unchecked(a) }
}

/// Just a wrapper for [`ManuallyDrop`] that has better `const` ergonomics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct NoDrop<T>(ManuallyDrop<T>)
where
    T: ?Sized;

impl<T> NoDrop<T>
where
    T: ?Sized,
{
    /// Get a pointer to the underlying `T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_ptr(&self) -> *const T {
        // SAFETY: `ManuallyDrop` and `NoDrop` are transparent over `T`.
        self as *const Self as *const T
    }

    /// Get a mutable pointer to the underlying `T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_ptr_mut(&mut self) -> *mut T {
        // SAFETY: `ManuallyDrop` and `NoDrop` are transparent over `T`.
        self as *mut Self as *mut T
    }

    /// Get a reference to the underlying `T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_ref(&self) -> &T {
        // SAFETY: The underlying `T` is initialized.
        unsafe { &*self.as_ptr() }
    }

    /// Get a mutable reference to the underlying `T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn as_mut(&mut self) -> &mut T {
        // SAFETY: The underlying `T` is initialized.
        unsafe { &mut *self.as_ptr_mut() }
    }

    /// Drop the underlying value.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this `NoDrop<T>` is never used again.
    ///
    /// See [`ManuallyDrop::drop`] for more details.
    #[inline(always)]
    pub(crate) unsafe fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.as_mut()) }
    }
}

impl<T> NoDrop<T> {
    /// Create a new `NoDrop<T>` from some `T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new(value: T) -> NoDrop<T> {
        NoDrop(ManuallyDrop::new(value))
    }

    /// Take the underlying `T` out of this `NoDrop<T>` container.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this `NoDrop<T>` is never used again.
    ///
    /// See [`ManuallyDrop::take`] for more details.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn take(&mut self) -> T {
        // SAFETY: We're reading from a reference, and the caller
        //         ensures that this container is never used again.
        unsafe { ptr::read(self.as_ref()) }
    }

    /// Get the underlying `T` by taking ownership over this `NoDrop<T>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn into_inner(self) -> T {
        ManuallyDrop::into_inner(self.0)
    }

    /// This will explicitly forget the underlying `T` by taking ownership
    /// over this `NoDrop<T>`.
    #[inline(always)]
    pub(crate) const fn forget(self) {}
}

impl<T, E> NoDrop<Result<T, E>> {
    /// Maps the underlying `Result<T, E>` into a `Result<NoDrop<T>, NoDrop<E>>`.
    #[inline(always)]
    pub(crate) const fn transpose(self) -> Result<NoDrop<T>, NoDrop<E>> {
        // NOTE: Why not use a simple `match`? As of writing, Rust has some limitations
        //       on how far the drop checker can work in const, a simple match on a result
        //       is unfortunately one of them.
        //
        // SAFETY: References are always valid for reads, and we know that `self`
        //         is never touched again after this point.
        match self.as_ref() {
            Ok(ok) => Ok(NoDrop::new(unsafe { ptr::read(ok) })),
            Err(err) => Err(NoDrop::new(unsafe { ptr::read(err) })),
        }
    }
}

impl<T> NoDrop<Option<T>> {
    /// Maps the underlying `Option<T>` into a `Option<NoDrop<T>>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn transpose(self) -> Option<NoDrop<T>> {
        // NOTE: Why not use a simple `match`? As of writing, Rust has some limitations
        //       on how far the drop checker can work in const, a simple match on a result
        //       is unfortunately one of them.
        //
        // SAFETY: References are always valid for reads, and we know that `self`
        //         is never touched again after this point.
        match self.as_ref() {
            Some(some) => Some(NoDrop::new(unsafe { ptr::read(some) })),
            None => None,
        }
    }
}
