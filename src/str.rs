use core::ops::Range;

use crate::macros::{assert_unchecked, unreachable_unchecked};

/// Returns whether a given byte is a UTF-8 character boundary.
///
/// # Correctness
///
/// For sources that aren't UTF-8, this may return false positives,
/// but will never return false negatives.
#[inline(always)]
#[must_use]
pub const fn is_utf8_char_boundary(byte: u8) -> bool {
    // Is it ascii (`0x00..=0x7F`) or are the leading two bits set to one (`0xC0..=0xFF`).
    matches!(byte, 0x00..=0x7F | 0xC0..=0xFF)
}

/// Why? Why not? Mainly just to flex my stupidity when, in reality, it does not matter like, at all.
#[inline(always)]
#[must_use]
pub const fn floor_char_boundary(s: &str, index: usize) -> usize {
    let new_index = if index >= s.len() {
        s.len()
    } else {
        // NOTE: We need to find the rightmost byte in the range `index.saturating_sub(3)..=index` is a UTF-8
        //       character boundary.
        //
        //       We use the range `index.saturating_sub(3)..index + 1` in the code, but they're equivalent. The
        //       inclusive range just better descrives what we're doing.

        let range = index.saturating_sub(3)..index.checked_add(1).unwrap();
        let Range { start, mut end } = {
            // SAFETY: We know that `index.saturating_sub(3) <= index`, and that `index < s.len()`.
            let start = unsafe { s.as_ptr().add(range.start) };
            // SAFETY: We know that `index + 1 <= s.len()`.
            let end = unsafe { s.as_ptr().add(range.end) };

            // SAFETY: We know that `start < end`.
            start..end
        };

        // SAFETY: UTF-8 characters occupy at *most* 4 bytes, and since we know `s` to be valid UTF-8,
        //         we will only need to scan at *most* 4 bytes. This is used as a hint *just in case*
        //         LLVM forgets this, in order to give it more wiggle room with optimizations.
        unsafe { assert_unchecked!(end.offset_from_unsigned(start) <= 4, "`end - start > 4`") };

        'block: {
            // SAFETY: We know that `start` and `end` are derived from the same allocated object. We also ensure
            //         that `start <= end` is always upheld, so there is *zero* chance for overflow.
            while unsafe { end.offset_from_unsigned(start) > 0 } {
                // SAFETY: We know that `end - start >= 1`.
                end = unsafe { end.sub(1) };
                // SAFETY: Since we're using an exclusive range, `end - 1` is the actual address of the *last* byte
                //         in the range. We just decremented it.
                let value = unsafe { end.read() };

                if is_utf8_char_boundary(value) {
                    // SAFETY: We know that `start <= end` is always true. So we can soundly compute the relative
                    //         offset for the current byte from `start`.
                    let offset = unsafe { end.offset_from_unsigned(start) };

                    // SAFETY: Calculating the index for the current value from the index of `start` will never result in
                    //         an overflow, since `offset < range.len()` always holds true.
                    let index = unsafe { range.start.unchecked_add(offset) };

                    // NOTE: If we fail to break out of `'block`, then we'll hit undefined behavior.
                    break 'block index;
                }
            }

            // SAFETY: Unless `s` is ill-formed UTF-8, which is by definition undefined behavior, we know we have breaked out from the loop
            //         into `'block` already. This is because in valid UTF-8, there is always at least one character boundary within the preceding
            //         4 or less bytes before a given index, provided the index is in bounds for the string.
            //
            //         So, if we fail to exit the loop before this, then `s` is ill-formed, and the existence of it is undefined behavior anyways.
            unsafe { unreachable_unchecked!("`s` contains ill-formed UTF-8") }
        }
    };

    // SAFETY: We know that `new_index` is never greater than the length of `s`.
    unsafe { assert_unchecked!(new_index <= s.len(), "`new_index > s.len()`") };
    // SAFETY: We know that `new_index` is never greater than `index`. It is at *most* equal to it.
    unsafe { assert_unchecked!(new_index <= index, "`new_index > index`") };

    new_index
}

// #[unsafe(no_mangle)]
// fn _floor_char_boundary_pseudocode(s: &str, index: usize) -> usize {
//     if index >= s.len() {
//         s.len()
//     } else {
//         let lower_bound = index.saturating_sub(3);

//         let new_index = s.as_bytes()[lower_bound..=index]
//             .iter()
//             .rposition(|b| is_utf8_char_boundary(*b))
//             .unwrap();

//         // SAFETY: we know that the character boundary will be within four bytes

//         lower_bound + new_index
//     }
// }
