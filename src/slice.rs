#![allow(dead_code)]

use private::SliceKind;

use crate::mem::NoDrop;

/// The implementation details of the [`Slice`] trait.
pub(crate) mod private;

/// Trait for the various supported slices.
pub unsafe trait Slice: private::Slice {
    /// Returns the length of this slice.
    #[inline(always)]
    #[must_use]
    fn len(&self) -> usize {
        len(self)
    }

    #[inline(always)]
    #[must_use]
    fn from_elems(elems: &[Self::Item]) -> Result<&Self, Self::Error> {
        from_elems(elems)
    }

    #[inline(always)]
    #[must_use]
    fn from_elems_mut(elems: &mut [Self::Item]) -> Result<&mut Self, Self::Error> {
        from_elems_mut(elems)
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_unchecked(elems: &[Self::Item]) -> &Self {
        unsafe { from_elems_unchecked(elems) }
    }

    #[inline(always)]
    #[must_use]
    #[track_caller]
    unsafe fn from_elems_mut_unchecked(elems: &mut [Self::Item]) -> &mut Self {
        unsafe { from_elems_mut_unchecked(elems) }
    }
}

unsafe impl<S: private::Slice + ?Sized> Slice for S {}

/// Trait for the various supported slice indexes.
pub unsafe trait SliceIndex<S: Slice + ?Sized>: private::SliceIndex<S> {}

unsafe impl<I, S> SliceIndex<S> for I
where
    I: private::SliceIndex<S> + ?Sized,
    S: Slice + ?Sized,
{
}

/// Returns the length of a given slice pointer.
#[inline(always)]
#[must_use]
pub const fn len<S: Slice + ?Sized>(slice: *const S) -> usize {
    match S::WITNESS.0 {
        SliceKind::Slice { this: conv, .. } => conv.coerce_ptr(slice).len(),
        SliceKind::Str { this: conv, .. } => (conv.coerce_ptr(slice) as *const [u8]).len(),
    }
}

/// Given an error returned by attempting to create a slice, return an error string.
#[inline(always)]
#[must_use]
pub const fn error_string<S: Slice + ?Sized>(err: S::Error) -> &'static str {
    let err = NoDrop::new(err);

    match S::WITNESS.0 {
        #[allow(unreachable_code)]
        SliceKind::Slice { error, .. } => match error.coerce(err.into_inner()) {},
        SliceKind::Str { .. } => "invalid utf-8",
    }
}

/// Attempts to create a slice from a slice of its elements.
#[inline(always)]
#[must_use]
pub const fn from_elems<S: Slice + ?Sized>(elems: &[S::Item]) -> Result<&S, S::Error> {
    match S::WITNESS.0 {
        SliceKind::Slice { this, .. } => Ok(this.uncoerce_ref(elems)),
        SliceKind::Str { this, items, error } => this
            .wrap_ref()
            .wrap_result(error)
            .uncoerce(core::str::from_utf8(items.coerce_ref(elems))),
    }
}

/// Attempts to create a mutable slice from a mutable slice of its elements.
#[inline(always)]
#[must_use]
pub const fn from_elems_mut<S: Slice + ?Sized>(elems: &mut [S::Item]) -> Result<&mut S, S::Error> {
    match S::WITNESS.0 {
        SliceKind::Slice { this, .. } => Ok(this.uncoerce_mut(elems)),
        SliceKind::Str { this, error, items } => this
            .wrap_mut()
            .wrap_result(error)
            .uncoerce(core::str::from_utf8_mut(items.coerce_mut(elems))),
    }
}

/// Creates a slice from a slice of its elements without any checks.
///
/// # Safety
///
/// The caller must ensure that the provided element slice is indeed valid
/// according to the invariants of `S`.
///
/// Failure to do so will likely result in undefined behavior.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_elems_unchecked<S: Slice + ?Sized>(elems: &[S::Item]) -> &S {
    match S::WITNESS.0 {
        SliceKind::Slice { this, .. } => this.uncoerce_ref(elems),
        SliceKind::Str { this, items, .. } => {
            this.uncoerce_ref(unsafe { core::str::from_utf8_unchecked(items.coerce_ref(elems)) })
        }
    }
}

/// Creates a mutable slice from a mutable slice of its elements without any checks.
///
/// # Safety
///
/// The caller must ensure that the provided element slice is indeed valid
/// according to the invariants of `S`.
///
/// Failure to do so will likely result in undefined behavior.
#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn from_elems_mut_unchecked<S: Slice + ?Sized>(elems: &mut [S::Item]) -> &mut S {
    match S::WITNESS.0 {
        SliceKind::Slice { this, .. } => this.uncoerce_mut(elems),
        SliceKind::Str { this, items, .. } => this
            .uncoerce_mut(unsafe { core::str::from_utf8_unchecked_mut(items.coerce_mut(elems)) }),
    }
}
