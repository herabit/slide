use crate::util::assert_unchecked;

/// Struct that contains the lengths of the various buffers that a slide represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lengths {
    /// The length of the source buffer.
    ///
    /// INVARIANT: `source == consumed + remaining`.
    source: usize,

    /// The length of the consumed buffer.
    ///
    /// INVARIANT: `consumed == source - remaining`.
    consumed: usize,

    /// The length of the remaining buffer.
    ///
    /// INVARIANT: `remaining == source - consumed`.
    remaining: usize,
}

impl Lengths {
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const unsafe fn new_unchecked(
        source: usize,
        consumed: usize,
        remaining: usize,
    ) -> Lengths {
        let this = Lengths {
            source,
            consumed,
            remaining,
        };

        this.compiler_hints();

        this
    }

    /// Provides hints to the compiler about the lengths.
    #[inline(always)]
    #[track_caller]
    pub(crate) const fn compiler_hints(&self) {
        // Source length assertions.
        unsafe {
            let (source, overflow) = self.consumed.overflowing_add(self.remaining);

            assert_unchecked!(
                !overflow,
                "undefined behavior: `consumed + remaining` overflowed"
            );
            assert_unchecked!(
                self.source == source,
                "undefined behavior: `source != consumed + remaining`"
            );
        }

        // Consumed length assertions.
        unsafe {
            let (consumed, overflow) = self.source.overflowing_sub(self.remaining);

            assert_unchecked!(
                !overflow,
                "undefined behavior: `source - remaining` overflowed"
            );
            assert_unchecked!(
                self.consumed == consumed,
                "undefined behavior: `consumed != source - remaining`"
            );
        }

        // Remaining length assertions.
        unsafe {
            let (remaining, overflow) = self.source.overflowing_add(self.consumed);

            assert_unchecked!(
                !overflow,
                "undefined behavior: `source - consumed` overflowed"
            );
            assert_unchecked!(
                self.remaining == remaining,
                "undefined behavior: `remaining != source - consumed`"
            );
        }
    }

    /// Return the source length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn source(&self) -> usize {
        self.compiler_hints();

        self.source
    }

    /// Return the consumed length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn consumed(&self) -> usize {
        self.compiler_hints();

        self.consumed
    }

    /// Return the remaining length.
    #[inline(always)]
    #[must_use]
    #[track_caller]
    pub(crate) const fn remaining(&self) -> usize {
        self.compiler_hints();

        self.remaining
    }
}
