use crate::macros::assert_unchecked;

#[inline(always)]
#[must_use]
pub(crate) const fn is_utf8_cont(byte: u8) -> bool {
    matches!(byte, 0x80..=0xBF)
}

#[must_use]
pub(crate) const unsafe fn char_count_unchecked(start: *const u8, end: *const u8) -> usize {
    let mut cur = start;
    let mut count = 0_usize;

    // While `start < end`.
    while unsafe { end.offset_from_unsigned(cur) > 0 } {
        let byte = unsafe { cur.read() };

        if !is_utf8_cont(byte) {
            count = unsafe { count.unchecked_add(1) };
        }

        cur = unsafe { cur.add(1) };
    }

    count
}
