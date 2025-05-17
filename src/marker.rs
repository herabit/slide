//! # Where a lot of these ideas come from:
//!
//! For [`TypeEq`] and the [`type_fn`] module, I'm borrowing a lot from:
//!
//! 1. The [typewit](https://github.com/rodrimati1992/typewit/) crate.
//! 2. The [type-equalities](https://github.com/WorldSEnder/type-equalities-rs) crate.
//!
//! For the [`invariance`] module, it comes from the
//! [Phantom Variance Markers](https://github.com/rust-lang/rust/issues/135806) rust issue,
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

/// Marker type that acts as a proof that `Src == Dest`.
#[repr(transparent)]
pub(crate) struct TypeEq<Src: ?Sized, Dest: ?Sized>(Invariant<Src>, Invariant<Dest>);

impl<Src: ?Sized, Dest: ?Sized> TypeEq<Src, Dest> {
    /// Create a proof that `Src == Dest`.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that `Src == Dest`. Failure to do so
    /// is considered undefined behavior as it permits transmutes between
    /// `Src` and `Dest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn new_unchecked() -> Self {
        // SAFETY: The caller ensures that `Src == Dest`.
        Self(Invariant::new(), Invariant::new())
    }

    /// Create a proof that `Src == NewDest` given that `Src == Dest && Dest == NewDest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn join<NewDest: ?Sized>(
        self,
        _: TypeEq<Dest, NewDest>,
    ) -> TypeEq<Src, NewDest> {
        // SAFETY: `Src == Dest && Dest == NewDest` imples `Src == NewDest`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `Dest == Src`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn flip(self) -> TypeEq<Dest, Src> {
        // SAFETY: `Src == Dest` implies `Dest == Src`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `Call<F, Src> == Call<F, Dest>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn lift<F: Func<Src> + Func<Dest> + ?Sized>(
        self,
    ) -> TypeEq<Call<F, Src>, Call<F, Dest>> {
        // SAFETY: `Src == Dest` implies `Call<F, Src> == Call<F, Dest>`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Create a proof that `Uncall<F, Src> == Uncall<F, Dest>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unlift<F: RevFunc<Src> + RevFunc<Dest> + ?Sized>(
        self,
    ) -> TypeEq<Uncall<F, Src>, Uncall<F, Dest>> {
        self.lift::<Rev<F>>()
    }

    /// Create a proof that `Call<F, Src> == Call<F, Dest>` given
    /// some `F`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn apply<F: Func<Src> + Func<Dest>>(
        self,
        func: F,
    ) -> TypeEq<Call<F, Src>, Call<F, Dest>> {
        core::mem::forget(func);

        self.lift::<F>()
    }

    /// Create a proof that `Uncall<F, Src> == Uncall<F, Dest>` given
    /// some `F`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unapply<F: RevFunc<Src> + RevFunc<Dest>>(
        self,
        func: F,
    ) -> TypeEq<Uncall<F, Src>, Uncall<F, Dest>> {
        core::mem::forget(func);

        self.unlift::<F>()
    }

    /// Create new proof that `Call<NewDest::Func, Src> == NewDest` given that `Call<NewDest::Func, Dest> == NewDest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn project<NewDest: ?Sized>(self) -> TypeEq<Call<NewDest::Func, Src>, NewDest>
    where
        NewDest: HasFunc<Arg = Dest>,
        NewDest::Func: Func<Src>,
    {
        self.lift::<NewDest::Func>()
    }

    /// Create a new proof that `Uncall<Dest::Func, Src> == Uncall<Dest::Func, Dest>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn unproject(self) -> TypeEq<Uncall<Dest::Func, Src>, Uncall<Dest::Func, Dest>>
    where
        Dest: HasFunc,
        Dest::Func: RevFunc<Src>,
    {
        self.unlift::<Dest::Func>()
    }

    /// Create a proof that `&'a Src == &'a Dest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ref<'a>(self) -> TypeEq<&'a Src, &'a Dest> {
        self.project()
    }

    /// Create a proof that `&'a mut Src == &'a mut Dest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_mut<'a>(self) -> TypeEq<&'a mut Src, &'a mut Dest> {
        self.project()
    }

    /// Create a proof that `*const Src == *const Dest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr(self) -> TypeEq<*const Src, *const Dest> {
        self.project()
    }

    /// Create a proof that `*mut Src == *mut Dest`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr_mut(self) -> TypeEq<*mut Src, *mut Dest> {
        self.project()
    }

    /// Create a proof that `NonNull<Src> == NonNull<Dest>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_nonnull(self) -> TypeEq<NonNull<Src>, NonNull<Dest>> {
        self.project()
    }

    /// Coerce a `&'a Src` into a `&'a Dest`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ref<'a>(self, src: &'a Src) -> &'a Dest {
        self.wrap_ref().coerce(src)
    }

    /// Coerce a `&'a mut Src` into a `&'a mut Dest`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_mut<'a>(self, src: &'a mut Src) -> &'a mut Dest {
        self.wrap_mut().coerce(src)
    }

    /// Coerce a `*const Src` into a `*const Dest`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ptr(self, src: *const Src) -> *const Dest {
        self.wrap_ptr().coerce(src)
    }

    /// Coerce a `*mut Src` into a `*mut Dest`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_ptr_mut(self, src: *mut Src) -> *mut Dest {
        self.wrap_ptr_mut().coerce(src)
    }

    /// Coerce a `NonNull<Src>` into a `NonNull<Dest>`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_nonnull(self, src: NonNull<Src>) -> NonNull<Dest> {
        self.wrap_nonnull().coerce(src)
    }

    /// Uncoerce a `&'a Dest` back into a `&'a Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_ref<'a>(self, dest: &'a Dest) -> &'a Src {
        self.wrap_ref().uncoerce(dest)
    }

    /// Uncoerce a `&'a mut Dest` back into a `&'a mut Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_mut<'a>(self, dest: &'a mut Dest) -> &'a mut Src {
        self.wrap_mut().uncoerce(dest)
    }

    /// Uncoerce a `*const Dest` back into a `*const Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_ptr(self, dest: *const Dest) -> *const Src {
        self.wrap_ptr().uncoerce(dest)
    }

    /// Uncoerce a `*mut Dest` back into a `*mut Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_ptr_mut(self, dest: *mut Dest) -> *mut Src {
        self.wrap_ptr_mut().uncoerce(dest)
    }

    /// Uncoerce a `NonNull<Dest>` back into a `NonNull<Src>`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_nonnull(self, dest: NonNull<Dest>) -> NonNull<Src> {
        self.wrap_nonnull().uncoerce(dest)
    }
}

impl<Src: ?Sized> TypeEq<Src, Src> {
    /// Create a proof that `Src == Src`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        // SAFETY `Src == Src`.
        unsafe { Self::new_unchecked() }
    }
}

impl<Src, Dest> TypeEq<Src, Dest> {
    /// Provide the compiler some hints.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn compiler_hints<T>(self, value: T) -> T {
        // SAFETY: `Src == Dest`, therefore they have the same size, alignment, and layout niches.
        unsafe {
            assert_layout_unchecked!(
                Src,
                Dest,
                "`Src` and `Dest` must have the same memory layout"
            );
            assert_layout_unchecked!(
                Option<Src>,
                Option<Dest>,
                "`Src` and `Dest` must have the same memory layout"
            );
        }

        value
    }

    /// Create a proof that `[Src] == [Dest]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn wrap_slice(self) -> TypeEq<[Src], [Dest]> {
        self.compiler_hints(self.project())
    }

    /// Create a proof that `Option<Src> == Option<Dest>`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn wrap_option(self) -> TypeEq<Option<Src>, Option<Dest>> {
        self.compiler_hints(self.project())
    }

    /// Coerce a `Src` into a `Dest`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce(self, src: Src) -> Dest {
        // SAFETY: `Src == Dest`.
        self.compiler_hints(unsafe { crate::mem::transmute_unchecked(src) })
    }

    /// Coerce a `&'a [Src]` into a `&'a [Dest]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_slice<'a>(self, src: &'a [Src]) -> &'a [Dest] {
        self.wrap_slice().coerce_ref(src)
    }

    /// Coerce a `&'a mut [Src]` into a `&'a mut [Dest]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn coerce_slice_mut<'a>(self, src: &'a mut [Src]) -> &'a mut [Dest] {
        self.wrap_slice().coerce_mut(src)
    }

    /// Uncoerce a `Dest` back into a `Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce(self, dest: Dest) -> Src {
        self.flip().coerce(dest)
    }

    /// Uncoerce a `&'a [Dest]` back into a `&'a [Src]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_slice<'a>(self, dest: &'a [Dest]) -> &'a [Src] {
        self.wrap_slice().uncoerce_ref(dest)
    }

    /// Uncoerce a `&'a mut [Dest]` back into a `&'a mut [Src]`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn uncoerce_slice_mut<'a>(self, dest: &'a mut [Dest]) -> &'a mut [Src] {
        self.wrap_slice().uncoerce_mut(dest)
    }

    /// Swap a `Src` and a `Dest`.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn swap(self, x: &mut Src, y: &mut Dest) {
        core::mem::swap(self.coerce_mut(x), y)
    }

    /// Replace a `Dest` with some `Src`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn replace(self, src: Src, dest: &mut Dest) -> Dest {
        core::mem::replace(dest, self.coerce(src))
    }
}

impl<Src: ?Sized, Dest: ?Sized> Clone for TypeEq<Src, Dest> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Src: ?Sized, Dest: ?Sized> Copy for TypeEq<Src, Dest> {}
