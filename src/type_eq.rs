#![allow(dead_code)]

use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    Slice,
    macros::{assert_layout_unchecked, assert_unchecked},
    raw::{Chunk, Pos},
    util,
};

/// Marker struct whose instantiation proves that `L == R`, and inversely
/// `R == L`.
#[repr(transparent)]
pub struct TypeEq<L: ?Sized, R: ?Sized>(
    /// We need to be invariant over `L`.
    PhantomData<fn(PhantomData<L>) -> PhantomData<L>>,
    /// We need to be invariant over `R`.
    PhantomData<fn(PhantomData<R>) -> PhantomData<R>>,
);

impl<T: ?Sized> TypeEq<T, T> {
    /// Create a new [`TypeEq`].
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        // SAFETY: We know that `A == A`.
        unsafe { Self::new_unchecked() }
    }
}

impl<L: ?Sized, R: ?Sized> TypeEq<L, R> {
    /// Unsafely create a new [`TypeEq`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `L == R` (which implies `L == R`).
    ///
    /// Failure to do so is immediate undefined behavior.
    #[inline(always)]
    #[must_use]
    pub(crate) const unsafe fn new_unchecked() -> Self {
        // SAFETY: The caller ensures that `L == R`.
        Self(PhantomData, PhantomData)
    }

    /// Flip `L` and `R`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn flip(self) -> TypeEq<R, L> {
        // SAFETY: We know that `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert a `TypeEq<L, R>` into an `TypeEq<L, U>` given
    /// a `TypeEq<R, U>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn join<U: ?Sized>(self, with: TypeEq<R, U>) -> TypeEq<L, U> {
        let _ = with;
        // SAFETY: Since `L == R` and `R == U`, then `L == U`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Converts a `TypeEq<L, R>` to a `TypeEq<&'a L, &'a R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ref<'a>(self) -> TypeEq<&'a L, &'a R> {
        // SAFETY: Since `L == R`, then `&'a L == &'a R`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Converts a `TypeEq<L, R>` to a `TypeEq<&'a mut L, &'a mut R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_mut<'a>(self) -> TypeEq<&'a mut L, &'a mut R> {
        // SAFETY: Since `L == R`, then `&'a mut L == &'a mut R`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Converts a `TypeEq<L, R>` to a `TypeEq<*const L, *const R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr(self) -> TypeEq<*const L, *const R> {
        // SAFETY: Since `L == R`, then `*const L == *const R`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Converts a `TypeEq<L, R>` to a `TypeEq<*mut L, *mut R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_ptr_mut(self) -> TypeEq<*mut L, *mut R> {
        // SAFETY: Since `L == R`, then `*mut L == *mut R`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Converts a `TypeEq<L, R>` to a `TypeEq<NonNull<L>, NonNull<R>>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_nonnull(self) -> TypeEq<NonNull<L>, NonNull<R>> {
        // SAFETY: Since `L == R`, then `NonNull<L> == NonNull<R>`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert a `&L` to a `&R`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn as_right<'a>(self, left: &'a L) -> &'a R {
        self.wrap_ref().into_right(left)
    }

    /// Convert a `&mut L` to a `&mut R`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn as_right_mut<'a>(self, left: &'a mut L) -> &'a mut R {
        self.wrap_mut().into_right(left)
    }

    /// Convert a `&R` to a `&L`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn as_left<'a>(self, right: &'a R) -> &'a L {
        self.wrap_ref().into_left(right)
    }

    /// Convert a `&mut R` to a `&mut L`.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn as_left_mut<'a>(self, right: &'a mut R) -> &'a mut L {
        self.wrap_mut().into_left(right)
    }
}

impl<L, R> TypeEq<L, R> {
    /// Provide hints to the compiler.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn compiler_hints<T>(self, x: T) -> T {
        unsafe {
            // SAFETY: We know `L == R`.
            assert_layout_unchecked!(L, R, "`L` and `R` must have the same layout");

            // SAFETY: `L == R` implies that `Option<L> == Option<R>`.
            assert_layout_unchecked!(
                Option<L>,
                Option<R>,
                "`Option<L>` and `Option<R>` must have the same layout"
            );
        }

        x
    }

    /// Cast a given `L` into an `R`.
    ///
    /// This operation is a no-op.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn into_right(self, left: L) -> R {
        // SAFETY: We know `L == R`.
        self.compiler_hints(unsafe { util::transmute_unchecked(left) })
    }

    /// Cast a given `R` into an `R`.
    ///
    /// This operation is a no-op.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn into_left(self, right: R) -> L {
        // SAFETY: We know `L == R`.
        self.compiler_hints(unsafe { util::transmute_unchecked(right) })
    }

    /// Convert a `TypeEq<L, R>` to a `TypeEq<[L], [R]>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_slice(self) -> TypeEq<[L], [R]> {
        // SAFETY: Since `L == R`, then `[L] == [R]`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<'a, L: ?Sized, R: ?Sized> TypeEq<&'a L, &'a R> {
    /// Convert a `TypeEq<&'a L, &'a R>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_ref(self) -> TypeEq<L, R> {
        // SAFETY: Since `&'a L == &'a R`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<'a, L: ?Sized, R: ?Sized> TypeEq<&'a mut L, &'a mut R> {
    /// Convert a `TypeEq<&'a mut L, &'a mut R>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_mut(self) -> TypeEq<L, R> {
        // SAFETY: Since `&'a mut L == &'a mut R`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: ?Sized, R: ?Sized> TypeEq<*const L, *const R> {
    /// Convert a `TypeEq<*const L, *const R>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_ptr(self) -> TypeEq<L, R> {
        // SAFETY: Since `*const L == *const R`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: ?Sized, R: ?Sized> TypeEq<*mut L, *mut R> {
    /// Convert a `TypeEq<*mut L, *mut R>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_ptr_mut(self) -> TypeEq<L, R> {
        // SAFETY: Since `*mut L == *mut R`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: ?Sized, R: ?Sized> TypeEq<NonNull<L>, NonNull<R>> {
    /// Convert a `TypeEq<NonNull<L>, NonNull<R>>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_nonnull(self) -> TypeEq<L, R> {
        // SAFETY: Since `NonNull<L> == NonNull<R>`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L, R> TypeEq<[L], [R]> {
    /// Convert a `TypeEq<[L], [R]>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_slice(self) -> TypeEq<L, R> {
        // SAFETY: Since `[L] == [R]`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: Slice + ?Sized, R: Slice + ?Sized> TypeEq<L, R> {
    /// Convert a `TypeEq<L, R>` to a `TypeEq<Pos<L>, Pos<R>>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_pos(self) -> TypeEq<Pos<L>, Pos<R>> {
        // SAFETY: Since `L == R`, `Pos<L> == Pos<R>`.
        unsafe { TypeEq::new_unchecked() }
    }

    /// Convert a `TypeEq<L, R>` to a `TypeEq<Chunk<L>, Chunk<R>>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn wrap_chunk(self) -> TypeEq<Chunk<L>, Chunk<R>> {
        // SAFETY: Since `L == R`, `Chunk<L> == Chunk<R>`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: Slice + ?Sized, R: Slice + ?Sized> TypeEq<Pos<L>, Pos<R>> {
    /// Convert a `TypeEq<Pos<L>, Pos<R>>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peel_pos(self) -> TypeEq<L, R> {
        // SAFETY: Since `Pos<L> == Pos<R>`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: Slice + ?Sized, R: Slice + ?Sized> TypeEq<Chunk<L>, Chunk<R>> {
    /// Convert a `TypeEq<Chunk<L>, Chunk<R>>` to a `TypeEq<L, R>`.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn peek_chunk(self) -> TypeEq<L, R> {
        // SAFETY: Since `Chunk<L> == Chunk<R>`, `L == R`.
        unsafe { TypeEq::new_unchecked() }
    }
}

impl<L: ?Sized, R: ?Sized> Clone for TypeEq<L, R> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: ?Sized, R: ?Sized> Copy for TypeEq<L, R> {}
