#![allow(dead_code)]

use core::{
    ops::Range,
    ptr::{self, NonNull},
};

use private::{IndexKind, SliceKind};

use crate::macros::{assert_unchecked, unreachable_unchecked};

/// The implementation details of the [`Slice`] trait.
pub(crate) mod private;

/// Trait for the various supported slices.
pub trait Slice: private::Slice {
    /// Returns the length of this slice.
    #[inline(always)]
    #[must_use]
    fn len(&self) -> usize {
        len(self)
    }
}

impl<S: private::Slice + ?Sized> Slice for S {}

/// Trait for the various supported slice indexes.
pub trait SliceIndex<S: Slice + ?Sized>: private::SliceIndex<S> {}

impl<I, S> SliceIndex<S> for I
where
    I: private::SliceIndex<S> + ?Sized,
    S: Slice + ?Sized,
{
}

/// Returns the length of a given slice.
#[inline(always)]
#[must_use]
pub const fn len<S: Slice + ?Sized>(slice: *const S) -> usize {
    match S::WITNESS.0 {
        SliceKind::Slice(conv) => conv.coerce_ptr(slice).len(),
        SliceKind::Str(conv) => (conv.coerce_ptr(slice) as *const [u8]).len(),
    }
}

#[inline(always)]
#[must_use]
pub const fn from_parts<S: Slice + ?Sized>(data: *const S::Item, len: usize) -> *const S {
    match S::WITNESS.0 {
        SliceKind::Slice(conv) => conv.uncoerce_ptr(ptr::slice_from_raw_parts(data, len)),
        SliceKind::Str(conv) => {
            conv.uncoerce_ptr(ptr::slice_from_raw_parts(data, len) as *const str)
        }
    }
}

#[inline(always)]
#[must_use]
pub const fn from_parts_mut<S: Slice + ?Sized>(data: *mut S::Item, len: usize) -> *mut S {
    from_parts::<S>(data, len).cast_mut()
}

#[inline(always)]
#[must_use]
pub const fn from_parts_nonnull<S: Slice + ?Sized>(
    data: NonNull<S::Item>,
    len: usize,
) -> NonNull<S> {
    NonNull::new(from_parts_mut(data.as_ptr(), len)).unwrap()
}

#[inline(always)]
pub const unsafe fn from_raw_parts<'a, S: Slice + ?Sized>(
    data: *const S::Item,
    len: usize,
) -> &'a S {
    unsafe { &*from_parts::<S>(data, len) }
}

#[inline(always)]
pub const unsafe fn from_raw_parts_mut<'a, S: Slice + ?Sized>(
    data: *mut S::Item,
    len: usize,
) -> &'a mut S {
    unsafe { &mut *from_parts_mut(data, len) }
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn get_unchecked<S: Slice + ?Sized, I: SliceIndex<S>>(
    slice: *const S,
    index: I,
) -> *const I::Output {
    match S::WITNESS.0 {
        SliceKind::Slice(..) => {
            if let IndexKind::Index(i, o) = I::WITNESS.0 {
                let index = i.coerce(index);

                // SAFETY: The caller ensures that the index is in bounds.
                unsafe { assert_unchecked!(index <= len(slice), "`index > len`") };

                // SAFETY: The caller ensures that the index is in bounds.
                let ptr = unsafe { slice.cast::<S::Item>().add(index) };

                o.uncoerce_ptr(ptr)
            } else {
                // Calculate the range.
                let range = I::WITNESS.0.into_range(len(slice), index);

                // SAFETY: The caller ensures that the index is in bounds.
                unsafe {
                    assert_unchecked!(range.start <= range.end, "`start > end`");
                    assert_unchecked!(range.end <= len(slice), "`end > len`");
                }

                // SAFETY: The caller ensures that the index is in bounds.
                let ptr = unsafe { slice.cast::<S::Item>().add(range.start) };
                let len = unsafe { range.end.unchecked_sub(range.start) };

                let slice = from_parts(ptr, len);

                I::WITNESS.0.output().uncoerce_ptr(slice)
            }
        }
        SliceKind::Str(s) => {
            // Calculate the range.
            let range = I::WITNESS.0.into_range(len(slice), index);

            // SAFETY: The caller ensures this is safe.
            let slice = unsafe { get_unchecked(s.coerce_ptr(slice) as *const [u8], range) };
            let slice = s.uncoerce_ptr(slice as *const str);

            I::WITNESS.0.output().uncoerce_ptr(slice)
        }
    }
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn get_unchecked_mut<S: Slice + ?Sized, I: SliceIndex<S>>(
    slice: *mut S,
    index: I,
) -> *mut I::Output {
    unsafe { get_unchecked(slice, index) }.cast_mut()
}

#[inline(always)]
#[must_use]
#[track_caller]
pub const unsafe fn get_unchecked_nonnull<S: Slice + ?Sized, I: SliceIndex<S>>(
    slice: NonNull<S>,
    index: I,
) -> NonNull<I::Output> {
    unsafe { NonNull::new_unchecked(get_unchecked_mut(slice.as_ptr(), index)) }
}

#[unsafe(no_mangle)]
unsafe fn get(s: NonNull<str>, r: Range<usize>) -> NonNull<str> {
    unsafe { get_unchecked_nonnull(s, r) }
}
