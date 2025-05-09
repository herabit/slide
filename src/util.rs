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

/// Marks some code path as cold.
#[inline(always)]
#[cold]
#[allow(dead_code)]
pub(crate) const fn cold<T>(x: T) -> T {
    x
}

/// Marks some condition as likely.
#[inline(always)]
#[must_use]
#[allow(dead_code)]
pub(crate) const fn likely(cond: bool) -> bool {
    if !cond {
        cold(());
    }

    cond
}

/// Marks some condition as unlikely.
#[inline(always)]
#[must_use]
#[allow(dead_code)]
pub(crate) const fn unlikely(cond: bool) -> bool {
    if cond {
        cold(());
    }

    cond
}

/// Unchecked signed addition for [`usize`].
#[inline(always)]
#[track_caller]
#[must_use]
pub(crate) const unsafe fn unchecked_add_signed(lhs: usize, rhs: isize) -> usize {
    let (res, overflow) = lhs.overflowing_add_signed(rhs);

    unsafe { assert_unchecked!(!overflow, "undefined behavior: overflow") };

    res
}

/// Overflowing signed difference for [`usize`].
#[inline(always)]
#[must_use]
#[allow(dead_code)]
pub(crate) const fn overflowing_signed_diff(lhs: usize, rhs: usize) -> (isize, bool) {
    // Overflow cases (`lhs` and `rhs` are unsigned, `res` is signed):
    //
    //
    // if `lhs >= rhs && res < 0` {
    //
    //     In this case, `lhs - rhs` does not overflow, but the resulting value
    //     looks negative, when it shouldn't.
    //
    // } else if `lhs < rhs && res >= 0` {
    //
    //     In this case, `lhs - rhs` does overflow, but the resulting value
    //     looks non-negative, when it shouldn't.
    //
    // }
    //
    // This can be represented as the following boolean expression:
    //     `(lhs >= rhs && res < 0) || (lhs < rhs && res >= 0)`
    // which can be simplified as
    //     `(lhs >= rhs) == (res < 0)`
    // .
    let res = lhs.wrapping_sub(rhs) as isize;
    let overflow = (lhs >= rhs) == (res < 0);

    (res, overflow)
}

/// Unchecked signed difference for [`usize`].
#[inline(always)]
#[track_caller]
#[must_use]
#[allow(dead_code)]
pub(crate) const unsafe fn unchecked_signed_diff(lhs: usize, rhs: usize) -> isize {
    let (res, overflow) = overflowing_signed_diff(lhs, rhs);

    unsafe { assert_unchecked!(!overflow, "undefined behavior: overflow") };

    res
}

/// Checked signed difference for [`usize`].
#[inline(always)]
#[track_caller]
#[must_use]
#[allow(dead_code)]
pub(crate) const fn checked_signed_diff(lhs: usize, rhs: usize) -> Option<isize> {
    let (res, overflow) = overflowing_signed_diff(lhs, rhs);

    if !overflow { Some(res) } else { None }
}

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
#[allow(dead_code)]
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

/// Create a [`NonNull`] subslice pointer.
#[inline(always)]
#[track_caller]
#[must_use]
pub(crate) const unsafe fn nonnull_subslice<T>(
    slice: NonNull<[T]>,
    Range { start, end }: Range<usize>,
) -> NonNull<[T]> {
    unsafe {
        assert_unchecked!(start <= end, "undefined behavior: `start > end`");
        assert_unchecked!(
            end <= slice.len(),
            "undefined behavior: `end > slice.len()`"
        );
    }

    let new_len = end.checked_sub(start).unwrap();
    let new_ptr = unsafe { slice.cast::<T>().add(start) };

    nonnull_slice(new_ptr, new_len)
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
