//! # Where a lot of these ideas come from:
//!
//! For [`TypeEq`] and the [`type_fn`] module, I'm borrowing a lot from:
//!
//! 1. The [typewit](https://github.com/rodrimati1992/typewit/) crate.
//! 2. The [type-equalities](https://github.com/WorldSEnder/type-equalities-rs) crate.
//!
//! For [`Covariant`], [`Contravariant`], and [`Invariant`], a lot of this
//! comes from the [Phantom Variance Markers](https://github.com/rust-lang/rust/issues/135806) rust issue,
//! which as of writing is not yet stable.
//!
//! We implement these newtypes instead of directly using [`PhantomData`],
//! as it's kinda hard to reason about variance rules without looking at
//! some table, and these newtypes provide a simpler interface for understanding
//! what the hell is actually being said.

#![allow(dead_code)]

use core::ptr::NonNull;

use type_fn::{Call, Func, HasFunc, Rev, RevFunc, Uncall};
use variance::Invariant;

use crate::macros::assert_layout_unchecked;

/// Module for type level functions.
pub(crate) mod type_fn;
/// Module for variance markers.
pub(crate) mod variance;

/// Marker type that acts as a proof that two types are equivalent.
#[repr(transparent)]
pub(crate) struct TypeEq<T: ?Sized, U: ?Sized>(Invariant<T>, Invariant<U>);

impl<T: ?Sized, U: ?Sized> TypeEq<T, U> {
    /// Create a proof that `T == U`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that `T == U`. Failure to do so
    /// is considered undefined behavior as it permits transmutes between
    /// `T` and `U`.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn new_unchecked() -> Self {
        // SAFETY: The caller ensures that `T == U`.
        Self(Invariant::new(), Invariant::new())
    }

    /// Create a proof that `T == V` given that `T == U && U == V`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn join<V: ?Sized>(self, _: TypeEq<U, V>) -> TypeEq<T, V> {
        // SAFETY: `T == U && U == V` imples `T == V`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `U == T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn flip(self) -> TypeEq<U, T> {
        // SAFETY: `T == U` implies `U == T`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `Call<F, T> == Call<F, U>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn lift<F: Func<T> + Func<U> + ?Sized>(
        self,
    ) -> TypeEq<Call<F, T>, Call<F, U>> {
        // SAFETY: `T == U` implies `Call<F, T> == Call<F, U>`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `Uncall<F, T> == Uncall<F, U>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unlift<F: RevFunc<T> + RevFunc<U> + ?Sized>(
        self,
    ) -> TypeEq<Uncall<F, T>, Uncall<F, U>> {
        self.lift::<Rev<F>>()
    }

    /// Create a proof that `Call<F, T> == Call<F, U>` given
    /// some `F`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn apply<F: Func<T> + Func<U>>(
        self,
        func: F,
    ) -> TypeEq<Call<F, T>, Call<F, U>> {
        core::mem::forget(func);

        self.lift::<F>()
    }

    /// Create a proof that `Uncall<F, T> == Uncall<F, U>` given
    /// some `F`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unapply<F: RevFunc<T> + RevFunc<U>>(
        self,
        func: F,
    ) -> TypeEq<Uncall<F, T>, Uncall<F, U>> {
        core::mem::forget(func);

        self.unlift::<F>()
    }

    /// Create new proof that `A == Call<A::Func, U>` given that `Call<A::Func, T> == A`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn project<A: ?Sized>(self) -> TypeEq<A, Call<A::Func, U>>
    where
        A: HasFunc<Arg = T>,
        A::Func: Func<U>,
    {
        self.lift::<A::Func>()
    }

    /// Create a new proof that `Uncall<T::Func, T> == Uncall<T::Func, U>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unproject(self) -> TypeEq<Uncall<T::Func, T>, Uncall<T::Func, U>>
    where
        T: HasFunc,
        T::Func: RevFunc<U>,
    {
        self.unlift::<T::Func>()
    }

    /// Create a proof that `&'a T == &'a U`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ref<'a>(self) -> TypeEq<&'a T, &'a U> {
        self.project()
    }

    /// Create a proof that `&'a mut T == &'a mut U`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_mut<'a>(self) -> TypeEq<&'a mut T, &'a mut U> {
        self.project()
    }

    /// Create a proof that `*const T == *const U`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr(self) -> TypeEq<*const T, *const U> {
        self.project()
    }

    /// Create a proof that `*mut T == *mut U`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr_mut(self) -> TypeEq<*mut T, *mut U> {
        self.project()
    }

    /// Create a proof that `NonNull<T> == NonNull<U>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_nonnull(self) -> TypeEq<NonNull<T>, NonNull<U>> {
        self.project()
    }

    /// Coerce a `&'a T` into a `&'a U`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ref<'a>(self, value: &'a T) -> &'a U {
        self.wrap_ref().coerce(value)
    }

    /// Coerce a `&'a mut T` into a `&'a mut U`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_mut<'a>(self, value: &'a mut T) -> &'a mut U {
        self.wrap_mut().coerce(value)
    }

    /// Coerce a `*const T` into a `*const U`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ptr(self, value: *const T) -> *const U {
        self.wrap_ptr().coerce(value)
    }

    /// Coerce a `*mut T` into a `*mut U`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ptr_mut(self, value: *mut T) -> *mut U {
        self.wrap_ptr_mut().coerce(value)
    }
}

impl<T: ?Sized> TypeEq<T, T> {
    /// Create a proof that `T == T`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        // SAFETY `T == T`.
        unsafe { Self::new_unchecked() }
    }
}

impl<T, U> TypeEq<T, U> {
    /// Provide the compiler some hints.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn compiler_hints<V>(self, value: V) -> V {
        // SAFETY: `T == U`, therefore they have the same size, alignment, and layout niches.
        unsafe {
            assert_layout_unchecked!(T, U, "`T` and `U` must have the same memory layout");
            assert_layout_unchecked!(
                Option<T>,
                Option<U>,
                "`T` and `U` must have the same memory layout"
            );
        }

        value
    }

    /// Create a proof that `[T] == [U]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn wrap_slice(self) -> TypeEq<[T], [U]> {
        self.compiler_hints(self.project())
    }

    /// Create a proof that `Option<T> == Option<U>`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn wrap_option(self) -> TypeEq<Option<T>, Option<U>> {
        self.compiler_hints(self.project())
    }

    /// Coerce a `T` into a `U`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce(self, value: T) -> U {
        // SAFETY: `T == U`.
        self.compiler_hints(unsafe { crate::mem::transmute_unchecked(value) })
    }

    /// Swap a `T` and a `U`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn swap(self, x: &mut T, y: &mut U) {
        core::mem::swap(self.coerce_mut(x), y)
    }
}

impl<T: ?Sized, U: ?Sized> Clone for TypeEq<T, U> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, U: ?Sized> Copy for TypeEq<T, U> {}
