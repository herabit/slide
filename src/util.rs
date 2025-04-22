use core::{
    hint,
    ptr::{self, NonNull},
};

/// Computes `(lhs - rhs) / size_of::<T>>()`.
///
/// # Safety
///
/// The caller must ensure that `lhs >= rhs`, as well that all of the invariants of
/// [`core::ptr::NonNull::offset_from`] are upheld.
///
/// # Panics
///
/// Panics if `T` is zero-sized.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn unchecked_sub<T>(lhs: *const T, rhs: *const T) -> usize {
    match unsafe { lhs.offset_from(rhs) } {
        diff @ 0.. => diff as usize,
        ..0 if cfg!(debug_assertions) => panic!("undefined behavior: `lhs >= rhs` must be true"),
        ..0 => unsafe { hint::unreachable_unchecked() },
    }
}

/// Create a [`NonNull`] slice pointer.
#[inline(always)]
#[must_use]
pub const fn nonnull_slice<T>(ptr: NonNull<T>, len: usize) -> NonNull<[T]> {
    NonNull::new(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len)).unwrap()
}
