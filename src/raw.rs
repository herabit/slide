#![allow(dead_code)]

use core::{hint, ptr::NonNull};

use data::{Data, DataMut, DataRef};
use non_zst::NonZst;
use zst::Zst;

use crate::{Direction, LEFT, RIGHT, util::nonnull_slice};

mod data;
mod non_zst;
mod zst;

/// A struct used for implementing slides.
#[repr(C)]
pub(crate) struct RawSlide<T> {
    start: NonNull<T>,
    data: Data<T>,
}

impl<T> RawSlide<T> {
    #[inline]
    #[must_use]
    #[track_caller]
    const unsafe fn new_unchecked(slice: NonNull<[T]>, offset: usize) -> RawSlide<T> {
        debug_assert!(offset <= slice.len(), "undefined behavior: out of bounds");

        RawSlide {
            start: slice.cast(),
            data: match size_of::<T>() {
                0 => Data {
                    zst: unsafe { Zst::new_unchecked(slice, offset) },
                },
                1.. => Data {
                    non_zst: unsafe { NonZst::new_unchecked(slice, offset) },
                },
            },
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_slice_offset(slice: &[T], offset: usize) -> Option<RawSlide<T>> {
        if offset <= slice.len() {
            let slice = NonNull::new(slice as *const [T] as *mut [T]).unwrap();

            Some(unsafe { RawSlide::new_unchecked(slice, offset) })
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_slice(slice: &[T]) -> RawSlide<T> {
        RawSlide::from_slice_offset(slice, 0).unwrap()
    }

    #[inline]
    #[must_use]
    pub const fn from_slice_mut_offset(slice: &mut [T], offset: usize) -> Option<RawSlide<T>> {
        if offset <= slice.len() {
            let slice = NonNull::new(slice as *mut [T]).unwrap();

            Some(unsafe { RawSlide::new_unchecked(slice, offset) })
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_slice_mut(slice: &mut [T]) -> RawSlide<T> {
        RawSlide::from_slice_mut_offset(slice, 0).unwrap()
    }

    /// Determines the offset of the cursor, and the ***total length*** of the
    /// slide.
    #[inline]
    #[must_use]
    pub const fn offset_len(&self) -> (usize, usize) {
        let (offset, len) = match self.data.borrow() {
            DataRef::Zst(data) => unsafe { data.offset_len(self.start) },
            DataRef::NonZst(data) => unsafe { data.offset_len(self.start) },
        };

        unsafe { hint::assert_unchecked(offset <= len) };

        (offset, len)
    }

    /// Assists the compiler.
    #[inline]
    pub const fn compiler_hints(&self) {
        let _ = self.offset_len();
    }

    /// Get a pointer to the source data.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> NonNull<[T]> {
        let (_, len) = self.offset_len();

        nonnull_slice(self.start, len)
    }

    /// Get a pointer to the consumed data.
    #[inline]
    #[must_use]
    pub const fn consumed(&self) -> NonNull<[T]> {
        let (len, _) = self.offset_len();

        nonnull_slice(self.start, len)
    }

    /// Get a pointer to the remaining data.
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> NonNull<[T]> {
        let (offset, len) = self.offset_len();

        nonnull_slice(
            unsafe { self.start.add(offset) },
            len.checked_sub(offset).unwrap(),
        )
    }
}

// impl<T> RawSlide<MaybeUninit<T>> {
//     /// Assume that the slide is initialized.
//     #[inline]
//     #[must_use]
//     pub const unsafe fn assume_init(self) -> RawSlide<T> {
//         unsafe { core::mem::transmute(self) }
//     }

//     /// Assume that this slide is initialized.
//     #[inline]
//     #[must_use]
//     pub const unsafe fn assume_init_ref(&self) -> &RawSlide<T> {
//         unsafe { &*(&raw const *self).cast() }
//     }

//     /// Assume that this slide is initialized mutably.
//     #[inline]
//     #[must_use]
//     pub const unsafe fn assume_init_mut(&mut self) -> &mut RawSlide<T> {
//         unsafe { &mut *(&raw mut *self).cast() }
//     }
// }

impl<T> RawSlide<T> {
    /// Set the offset for the cursor without any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `offset <= self.source().len()`.
    #[inline]
    #[track_caller]
    pub const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe { hint::assert_unchecked(offset <= self.source().len()) };

        match self.data.borrow_mut() {
            DataMut::Zst(data) => data.offset = offset,
            DataMut::NonZst(data) => data.cursor = unsafe { self.start.add(offset) },
        }
    }

    /// Set the offset for the cursor.
    ///
    /// # Returns
    ///
    /// Returns `None` if `offset > self.source().len()`.
    #[inline]
    #[track_caller]
    pub const fn set_offset_checked(&mut self, offset: usize) -> Option<()> {
        if offset <= self.source().len() {
            Some(unsafe { self.set_offset_unchecked(offset) })
        } else {
            None
        }
    }

    /// Set the offset for the cursor.
    ///
    /// # Panics
    ///
    /// Panics if `offset > self.source().len()`.
    #[inline]
    #[track_caller]
    pub const fn set_offset(&mut self, offset: usize) {
        self.set_offset_checked(offset).expect("out of bounds")
    }
}

impl<T> RawSlide<T> {
    /// Slide the cursor over in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure that `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure that `n <= self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const unsafe fn slide_unchecked(&mut self, dir: Direction, n: usize) {
        match dir {
            Direction::Right => {
                unsafe { hint::assert_unchecked(n <= self.remaining().len()) };

                match self.data.borrow_mut() {
                    DataMut::Zst(data) => data.offset = unsafe { data.offset.unchecked_add(n) },
                    DataMut::NonZst(data) => data.cursor = unsafe { data.cursor.add(n) },
                }
            }
            Direction::Left => {
                unsafe { hint::assert_unchecked(n <= self.consumed().len()) };

                match self.data.borrow_mut() {
                    DataMut::Zst(data) => data.offset = unsafe { data.offset.unchecked_sub(n) },
                    DataMut::NonZst(data) => data.cursor = unsafe { data.cursor.sub(n) },
                }
            }
        }
    }

    /// Slide the cursor over in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `n > self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const fn slide_checked(&mut self, dir: Direction, n: usize) -> Option<()> {
        match dir {
            Direction::Right if n <= self.remaining().len() => {
                Some(unsafe { self.slide_unchecked(RIGHT, n) })
            }
            Direction::Left if n <= self.consumed().len() => {
                Some(unsafe { self.slide_unchecked(LEFT, n) })
            }
            _ => None,
        }
    }

    /// Slide the cursor over in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `n > self.consumed().len()`.
    #[inline]
    #[track_caller]
    pub const fn slide(&mut self, dir: Direction, n: usize) {
        self.slide_checked(dir, n).expect("out of bounds")
    }
}

impl<T> RawSlide<T> {
    /// Peek `n` elements in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `n <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `n <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_slice_unchecked(&self, dir: Direction, n: usize) -> NonNull<[T]> {
        match dir {
            Direction::Right => {
                unsafe { hint::assert_unchecked(n <= self.remaining().len()) };

                nonnull_slice(self.remaining().cast(), n)
            }
            Direction::Left => {
                unsafe { hint::assert_unchecked(n <= self.consumed().len()) };

                // NOTE: The start of `self.remaining()` is equivalent to
                //       the cursor pointer for non-ZSTs.
                //
                //       LLVM is able to optimize
                //       the pointer math to calculate the pointer into
                //       a no-op read of the cursor pointer.
                //
                //       So rather than calculating the new pointer in terms of `consumed`,
                //       which starts at the `start` pointer, we utilize `remaining` instead
                //       to open more opportunities for further optimization.
                //
                //       I hope this makes sense.
                //
                //       Hopefully.
                //
                //       It's kinda hard to explain :/
                nonnull_slice(unsafe { self.remaining().cast().sub(n) }, n)
            }
        }
    }

    /// Peek `N` elements as an array in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `N <= self.remaining().len()`.
    /// - [`Direction::Left`]: The caller must ensure `N <= self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_array_unchecked<const N: usize>(
        &self,
        dir: Direction,
    ) -> NonNull<[T; N]> {
        unsafe { self.peek_slice_unchecked(dir, N) }.cast()
    }

    /// Peek the first element in a given direction without any checks.
    ///
    /// # Safety
    ///
    /// - [`Direction::Right`]: The caller must ensure `!self.remaining().is_empty()`.
    /// - [`Direction::Left`]: The caller must ensure `!self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn peek_unchecked(&self, dir: Direction) -> NonNull<T> {
        unsafe { self.peek_slice_unchecked(dir, 1) }.cast()
    }

    //// Peek `n` elements in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice_checked(&self, dir: Direction, n: usize) -> Option<NonNull<[T]>> {
        match dir {
            Direction::Right if n <= self.remaining().len() => {
                Some(unsafe { self.peek_slice_unchecked(RIGHT, n) })
            }
            Direction::Left if n <= self.consumed().len() => {
                Some(unsafe { self.peek_slice_unchecked(LEFT, n) })
            }
            _ => None,
        }
    }

    /// Peek `N` elements as an array in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Returns `None` if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array_checked<const N: usize>(
        &self,
        dir: Direction,
    ) -> Option<NonNull<[T; N]>> {
        match self.peek_slice_checked(dir, N) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    /// Peek the first element in a given direction.
    ///
    /// # Returns
    ///
    /// - [`Direction::Right`]: Returns `None` if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Returns `None` if `self.remaining().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_checked(&self, dir: Direction) -> Option<NonNull<T>> {
        match self.peek_slice_checked(dir, 1) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    /// Peek `n` elements in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `n > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `n > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_slice(&self, dir: Direction, n: usize) -> NonNull<[T]> {
        self.peek_slice_checked(dir, n).expect("out of bounds")
    }

    /// Peek `N` elements as an array in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `N > self.remaining().len()`.
    /// - [`Direction::Left`]: Panics if `N > self.consumed().len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek_array<const N: usize>(&self, dir: Direction) -> NonNull<[T; N]> {
        self.peek_slice(dir, N).cast()
    }

    /// Peek the first element in a given direction.
    ///
    /// # Panics
    ///
    /// - [`Direction::Right`]: Panics if `self.remaining().is_empty()`.
    /// - [`Direction::Left`]: Panics if `self.consumed().is_empty()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub const fn peek(&self, dir: Direction) -> NonNull<T> {
        self.peek_slice(dir, 1).cast()
    }
}

// NOTE: It doesn't make a whole lot of sense to take from the consumed buffer,
//       as it being consumed implies it was already taken.
//
//       We're unsure of whether implementing these methods in a direction agnostic
//       manner makes all that much sense with that taken into consideration.
// impl<T> RawSlide<T> {
//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const unsafe fn take_slice_unchecked(&mut self, n: usize) -> NonNull<[T]> {
//         let slice = unsafe { self.peek_slice_unchecked(RIGHT, n) };
//         unsafe { self.slide_unchecked(RIGHT, n) };

//         slice
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const unsafe fn take_array_unchecked<const N: usize>(&mut self) -> NonNull<[T; N]> {
//         unsafe { self.take_slice_unchecked(N) }.cast()
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const unsafe fn take_unchecked(&mut self) -> NonNull<T> {
//         unsafe { self.take_slice_unchecked(1) }.cast()
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take_slice_checked(&mut self, n: usize) -> Option<NonNull<[T]>> {
//         if n <= self.remaining().len() {
//             Some(unsafe { self.take_slice_unchecked(n) })
//         } else {
//             None
//         }
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take_array_checked<const N: usize>(&mut self) -> Option<NonNull<[T; N]>> {
//         match self.take_slice_checked(N) {
//             Some(ptr) => Some(ptr.cast()),
//             None => None,
//         }
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take_checked(&mut self) -> Option<NonNull<T>> {
//         match self.take_slice_checked(1) {
//             Some(ptr) => Some(ptr.cast()),
//             None => None,
//         }
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take_slice(&mut self, n: usize) -> NonNull<[T]> {
//         self.take_slice_checked(n).expect("out of bounds")
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take_array<const N: usize>(&mut self) -> NonNull<[T; N]> {
//         self.take_slice(N).cast()
//     }

//     #[inline]
//     #[must_use]
//     #[track_caller]
//     pub const fn take(&mut self) -> NonNull<T> {
//         self.take_slice(1).cast()
//     }
// }

impl<T> Clone for RawSlide<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RawSlide<T> {}
