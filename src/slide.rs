use core::hint::assert_unchecked;

#[derive(PartialEq, Eq)]
pub struct Slide<'a, T> {
    offset: usize,
    source: &'a [T],
}

impl<'a, T> Slide<'a, T> {
    #[inline(always)]
    #[must_use]
    pub const fn from_parts(source: &'a [T], offset: usize) -> Option<Self> {
        if offset <= source.len() {
            Some(Self { offset, source })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn new(source: &'a [T]) -> Self {
        Self::from_parts(source, 0).unwrap()
    }

    #[inline(always)]
    pub const fn compiler_hints(&self) {
        unsafe { assert_unchecked(self.offset <= self.source.len()) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn parts(&self) -> (&'a [T], usize) {
        self.compiler_hints();

        (self.source, self.offset)
    }

    #[inline(always)]
    #[must_use]
    pub const fn split(&self) -> (&'a [T], &'a [T]) {
        let (source, offset) = self.parts();

        source.split_at(offset)
    }

    #[inline(always)]
    #[must_use]
    pub const fn consumed(&self) -> &'a [T] {
        self.split().0
    }

    #[inline(always)]
    #[must_use]
    pub const fn remaining(&self) -> &'a [T] {
        self.split().1
    }

    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &'a [T] {
        self.parts().0
    }

    #[inline(always)]
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.parts().1
    }

    #[inline(always)]
    #[track_caller]
    pub const unsafe fn set_offset_unchecked(&mut self, offset: usize) {
        unsafe { assert_unchecked(offset <= self.source().len()) };

        self.offset = offset;
    }

    #[inline(always)]
    #[must_use]
    pub const fn set_offset_checked(&mut self, offset: usize) -> bool {
        if offset <= self.source().len() {
            unsafe { self.set_offset_unchecked(offset) };

            true
        } else {
            false
        }
    }

    #[inline(always)]
    #[track_caller]
    pub const fn set_offset(&mut self, offset: usize) {
        assert!(
            self.set_offset_checked(offset),
            "offset > self.source().len()"
        )
    }

    #[inline(always)]
    #[must_use]
    pub const unsafe fn peek_unchecked(&self, n: usize) -> &'a [T] {
        unsafe { assert_unchecked(n <= self.remaining().len()) };

        self.remaining().split_at(n).0
    }

    #[inline(always)]
    #[must_use]
    pub const fn peek_checked(&self, n: usize) -> Option<&'a [T]> {
        if n <= self.remaining().len() {
            Some(unsafe { self.peek_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn peek(&self, n: usize) -> &'a [T] {
        self.peek_checked(n).unwrap()
    }

    #[inline(always)]
    #[must_use]
    pub const unsafe fn peek_back_unchecked(&self, n: usize) -> &'a [T] {
        unsafe { assert_unchecked(n <= self.consumed().len()) };

        self.consumed()
            .split_at(self.consumed().len().checked_sub(n).unwrap())
            .1
    }

    #[inline(always)]
    #[must_use]
    pub const fn peek_back_checked(&self, n: usize) -> Option<&'a [T]> {
        if n <= self.consumed().len() {
            Some(unsafe { self.peek_back_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn peek_back(&self, n: usize) -> &'a [T] {
        self.peek_back_checked(n).unwrap()
    }

    #[inline(always)]
    pub const unsafe fn advance_unchecked(&mut self, n: usize) -> &'a [T] {
        unsafe { assert_unchecked(n <= self.remaining().len()) };

        self.offset = unsafe { self.offset.unchecked_add(n) };
        // Omitting the following line results in worse codegen despite it also being within `peek_back_unchecked`.
        unsafe { assert_unchecked(n <= self.consumed().len()) };

        unsafe { self.peek_back_unchecked(n) }
    }

    #[inline(always)]
    pub const fn advance_checked(&mut self, n: usize) -> Option<&'a [T]> {
        if n <= self.remaining().len() {
            Some(unsafe { self.advance_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    pub const fn advance(&mut self, n: usize) -> &'a [T] {
        self.advance_checked(n).unwrap()
    }

    #[inline(always)]
    pub const unsafe fn rewind_unchecked(&mut self, n: usize) -> &'a [T] {
        unsafe { assert_unchecked(n <= self.consumed().len()) };

        self.offset = unsafe { self.offset.unchecked_sub(n) };
        // Omitting the following line results in worse codegen despite it also being within `peek_unchecked`.
        unsafe { assert_unchecked(n <= self.remaining().len()) };

        unsafe { self.peek_unchecked(n) }
    }

    #[inline(always)]
    pub const fn rewind_checked(&mut self, n: usize) -> Option<&'a [T]> {
        if n <= self.consumed().len() {
            Some(unsafe { self.rewind_unchecked(n) })
        } else {
            None
        }
    }

    #[inline(always)]
    pub const fn rewind(&mut self, n: usize) -> &'a [T] {
        self.rewind_checked(n).unwrap()
    }
}

impl<'a, T> Clone for Slide<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Slide<'a, T> {}

// The code whose codegen we care about.
//
// It should be a No-op.
#[unsafe(no_mangle)]
fn peek_back<'a>(s: &mut Slide<'a, ()>) {
    while s.consumed().len() >= 10 {
        s.rewind(10);
    }
}
