use core::{
    ops::Range,
    ptr::{self, NonNull},
};

/// Asserts that a given condition must be true whilst forcing the compiler to optimize as such.
macro_rules! assert_unchecked {
    ($cond:expr, $($arg:tt)+) => {{
        #[cfg(debug_assertions)]
        {
            ::core::assert!(
                $crate::util::needs_unsafe($cond),
                $($arg)+
            )
        }

        #[cfg(not(debug_assertions))]
        {
            ::core::hint::assert_unchecked($cond)
        }
        // if ::core::cfg!(debug_assertions) {
        //     ::core::assert!(
        //         $crate::util::needs_unsafe($cond),
        //         $($arg)+
        //     )
        // } else {
        //     ::core::hint::assert_unchecked($cond)
        // }
    }};

    ($cond:expr $(,)?) => {{ $crate::util::assert_unchecked!($cond, "undefined behavior: assertion must always be true") }};
}

pub(crate) use assert_unchecked;

/// Computes the length of a given pointer range.
///
/// # Safety
///
/// The caller must ensure:
///
/// - `start <= end`.
///
/// - That `start` and `end` either point to the same address OR
///   be derived from the same allocated object.
///
/// - The distance, in bytes between `start` and `end` is an exact
///   multiple of `T`.
///
/// - Likely other things, don't fuck shit up.
///
/// # Panics
///
/// Panics if `T` is zero-sized.
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const unsafe fn ptr_len<T>(Range { start, end }: Range<*const T>) -> usize {
    assert!(size_of::<T>() != 0, "size must be nonzero");

    let len = unsafe { end.offset_from(start) };

    unsafe { assert_unchecked!(len >= 0, "undefined behavior: `start > end`") };

    len as usize
}

/// Create a [`NonNull`] slice pointer.
#[inline(always)]
#[must_use]
pub(crate) const fn nonnull_slice<T>(ptr: NonNull<T>, len: usize) -> NonNull<[T]> {
    NonNull::new(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len)).unwrap()
}

/// Create a [`NonNull`] pointer from a reference.
#[inline(always)]
#[must_use]
pub(crate) const fn nonnull_from_ref<T: ?Sized>(x: &T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(ptr::from_ref(x).cast_mut()) }
}

/// Create a [`NonNull`] pointer from a mutable reference.
#[inline(always)]
#[must_use]
pub(crate) const fn nonnull_from_mut<T: ?Sized>(x: &mut T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(ptr::from_mut(x)) }
}

/// A function that does nothing but assert that something needs to be unsafe.
#[inline(always)]
pub(crate) const unsafe fn needs_unsafe<T>(x: T) -> T {
    x
}
