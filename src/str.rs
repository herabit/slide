#![allow(dead_code)]

use core::{ops::ControlFlow, ptr::NonNull};

use crate::{
    macros::assert_unchecked,
    slice,
    util::{post_inc, pre_dec, repeat_u8, sum_bytes_usize, unlikely},
};

pub(crate) mod pos;

#[inline(always)]
#[must_use]
pub(crate) const unsafe fn next_code_point(ptr: &mut NonNull<u8>) -> u32 {
    let x = unsafe { post_inc(ptr, 1).read() };

    if x < 128 {
        return x as u32;
    }

    let init = utf8_first_byte(x, 2);
    let y = unsafe { post_inc(ptr, 1).read() };
    let mut ch = utf8_acc_cont_byte(init, y);

    if x >= 0xE0 {
        let z = unsafe { post_inc(ptr, 1).read() };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);

        ch = init << 12 | y_z;

        if x >= 0xF0 {
            let w = unsafe { post_inc(ptr, 1).read() };
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    ch
}

#[inline(always)]
#[must_use]
pub(crate) const unsafe fn last_code_point(ptr: &mut NonNull<u8>) -> u32 {
    let w = unsafe { pre_dec(ptr, 1).read() };

    if w < 128 {
        return w as u32;
    }

    let mut ch;

    let z = unsafe { pre_dec(ptr, 1).read() };

    ch = utf8_first_byte(z, 2);

    if utf8_is_cont(z) {
        let y = unsafe { pre_dec(ptr, 1).read() };
        ch = utf8_first_byte(y, 3);

        if utf8_is_cont(y) {
            let x = unsafe { pre_dec(ptr, 1).read() };

            ch = utf8_first_byte(x, 4);
            ch = utf8_acc_cont_byte(ch, y);
        }

        ch = utf8_acc_cont_byte(ch, z);
    }

    ch = utf8_acc_cont_byte(ch, w);

    ch
}

#[inline(always)]
#[must_use]
pub(crate) const fn count_chars(s: &str) -> usize {
    #[inline(never)]
    #[track_caller]
    const unsafe fn count_chars_slow(s: &[u8]) -> usize {
        unsafe {
            assert_unchecked!(cfg!(feature = "opt_size") || s.len() <= USIZE_SIZE * UNROLL_INNER)
        };

        count_chars_general(s)
    }

    /// Small wrapper that exists to prevent inlining.
    #[inline(never)]
    #[track_caller]
    #[unsafe(no_mangle)]
    const unsafe fn count_chars_fast(s: &[u8]) -> usize {
        unsafe {
            assert_unchecked!(
                cfg!(not(feature = "opt_size")) && s.len() > USIZE_SIZE * UNROLL_INNER
            )
        };

        count_chars_chunked(s)
    }

    // If we're optimizing for file size, or if we consider `s` to be too small,
    // compute it the "slow way".
    //
    // Otherwise, compute it the "fast way".
    let count = if cfg!(feature = "opt_size") || s.len() <= USIZE_SIZE * UNROLL_INNER {
        unsafe { count_chars_slow(s.as_bytes()) }
    } else {
        unsafe { count_chars_fast(s.as_bytes()) }
    };

    // SAFETY: The character count of a string will never exceed its
    //         size in memory.
    unsafe { assert_unchecked!(count <= s.len(), "character count larger than string") };

    count
}

/// This processes character counts in an element-by-element fashion.
#[inline(always)]
#[must_use]
const fn count_chars_general(s: &[u8]) -> usize {
    let mut total = 0_usize;
    let mut bytes = s;

    // FIXME: There's likely a faster way of handling this.
    while let Some(byte) = slice::next(&mut bytes) {
        let count = !utf8_is_cont(*byte) as usize;

        // SAFETY: The UTF-8 char count of a string cannot exceed its
        //         size in memory.
        total = unsafe { total.unchecked_add(count) };
    }

    total
}

// FIXME: Replace with `usize` once `align_to` works in const.
type Word = [u8; USIZE_SIZE];

// FIXME: Use `align_to` once available in const.
#[inline(always)]
#[must_use]
const fn align_to_words(s: &[u8]) -> (&[u8], &[Word], &[u8]) {
    let (body, tail) = slice::as_chunks::<USIZE_SIZE, _>(s);

    (&[], body, tail)
}

// FIXME: Replace with a `usize` read once `align_to` works in const.
#[inline(always)]
#[must_use]
const fn load_word(w: &Word) -> usize {
    usize::from_ne_bytes(*w)
}

/// This processes character counts in a chunked, heavily loop-unrolled fashion.
#[inline(always)]
#[must_use]
const fn count_chars_chunked(s: &[u8]) -> usize {
    // FIXME: Use `align_to` whenever it becomes available in const.
    let (head, mut body, tail) = align_to_words(s);

    if unlikely(body.is_empty() || head.len() > USIZE_SIZE || tail.len() > USIZE_SIZE) {
        return count_chars_general(s);
    }

    let head_total = count_chars_general(head);
    let tail_total = count_chars_general(tail);

    // SAFETY: The character count of a string cannot exceed its size
    //         in memory.
    let mut total = unsafe { head_total.unchecked_add(tail_total) };

    // FIXME: There may be a faster way of going over chunks,
    //        but this should suffice.
    while let Some(chunk) = slice::next_chunk(&mut body, CHUNK_SIZE) {
        let result = chunk_char_counts(chunk);
        let count = match result {
            ControlFlow::Continue(counts) | ControlFlow::Break(counts) => sum_bytes_usize(counts),
        };

        // SAFETY: The character count of a string cannot exceed its size
        //         in memory.
        total = unsafe { total.unchecked_add(count) };

        if let ControlFlow::Break(_) = result {
            break;
        }
    }

    total
}

#[inline(always)]
#[must_use]
#[track_caller]
const fn chunk_char_counts(chunk: &[Word]) -> ControlFlow<usize, usize> {
    assert!(chunk.len() <= CHUNK_SIZE);

    // This is essentially a `[u8; size_of::<usize>()]` of byte counts.
    let mut counts = 0_usize;
    // let mut i = 0_usize;

    let (mut unrolled_chunks, mut remainder) = slice::as_chunks::<UNROLL_INNER, _>(chunk);

    // We want the compiler to unroll some loops for us.
    while let Some(unrolled) = slice::next(&mut unrolled_chunks) {
        let mut unrolled = unrolled as &[_];

        while let Some(word) = slice::next(&mut unrolled) {
            let count = contains_non_utf8_cont(load_word(word));

            // SAFETY: `CHUNK_SIZE < 256`, so `counts` cannot ever possibly overflow.
            counts = unsafe { counts.unchecked_add(count) };
        }
    }

    if remainder.is_empty() {
        ControlFlow::Continue(counts)
    } else {
        while let Some(word) = slice::next(&mut remainder) {
            let count = contains_non_utf8_cont(load_word(word));

            // SAFETY: `CHUNK_SIZE < 256`, so `counts` cannot ever possibly overflow.
            counts = unsafe { counts.unchecked_add(count) };
        }

        ControlFlow::Break(counts)
    }
}

/// Returns whether a [`u8`] is a UTF-8 continuation byte (starts with bits `10`).
#[inline(always)]
#[must_use]
#[track_caller]
pub(crate) const fn utf8_is_cont(byte: u8) -> bool {
    (byte as i8) < -64
}

/// Checks whether each byte of `x` is the start of some UTF-8 character
/// sequence.
///
/// Bytes in `x` that are continuation byte are left as `0x00` (false),
/// any bytes tat are non-continuation bytes are left as `0x01` (true).
#[inline(always)]
#[must_use]
pub(crate) const fn contains_non_utf8_cont(x: usize) -> usize {
    const LSB: usize = repeat_u8(0x01);

    ((!x >> 7) | (x >> 6)) & LSB
}

const UNROLL_INNER: usize = 4;
const USIZE_SIZE: usize = size_of::<usize>();

// For correctness and safety, `CHUNK_SIZE` must be:
//
// - Less than or equal to 255 to avoid integer overflows in `counts`.
// - A multiple of `UNROLL_INNER`.
//
// I could write the performance info, but the most important thing for
// me is safety.
const CHUNK_SIZE: usize = 192;

const _: () = assert!(CHUNK_SIZE < 256, "`CHUNK_SIZE` must be less than 256");
const _: () = assert!(
    CHUNK_SIZE % UNROLL_INNER == 0,
    "`CHUNK_SIZE` must be a multiple of `UNROLL_INNER`"
);

const CONT_MASK: u8 = 0b0011_1111;
const UTF8_CHAR_WIDTH: &[u8; 256] = &{
    let mut widths = [0_u8; 256];
    let mut index = 0_usize;

    while index < 256 {
        widths[index] = match index as u8 {
            0x00..=0x7F => 1,
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => 0,
        };

        index += 1;
    }

    widths
};

#[inline(always)]
#[must_use]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

#[inline(always)]
#[must_use]
pub(crate) const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

#[inline(always)]
#[must_use]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}
