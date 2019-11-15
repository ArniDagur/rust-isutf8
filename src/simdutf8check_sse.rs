#[cfg(target_arch = "x86")]
pub use core::arch::x86::{
    __m128i, _mm_add_epi8, _mm_alignr_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8,
    _mm_loadu_si128, _mm_or_si128, _mm_set1_epi8, _mm_set_epi8, _mm_setr_epi8, _mm_setzero_si128,
    _mm_shuffle_epi8, _mm_srli_epi16, _mm_subs_epu8, _mm_testz_si128,
};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{
    __m128i, _mm_add_epi8, _mm_alignr_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8,
    _mm_loadu_si128, _mm_or_si128, _mm_set1_epi8, _mm_set_epi8, _mm_setr_epi8, _mm_setzero_si128,
    _mm_shuffle_epi8, _mm_srli_epi16, _mm_subs_epu8, _mm_testz_si128,
};
use core::default::Default;

extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
}

/* defined(__need_ptrdiff_t) */
/* Always define size_t when modules are available. */
#[allow(non_camel_case_types)]
pub type size_t = libc::c_ulong;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct __loadu_si128 {
    pub __v: __m128i,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessedUtfBytes {
    pub rawbytes: __m128i,
    pub high_nibbles: __m128i,
    pub carried_continuations: __m128i,
}

impl Default for ProcessedUtfBytes {
    fn default() -> Self {
        unsafe {
            ProcessedUtfBytes {
                rawbytes: _mm_setzero_si128(),
                high_nibbles: _mm_setzero_si128(),
                carried_continuations: _mm_setzero_si128(),
            }
        }
    }
}

/*
 * legal utf-8 byte sequence
 * http://www.unicode.org/versions/Unicode6.0.0/ch03.pdf - page 94
 *
 *  Code Points        1st       2s       3s       4s
 * U+0000..U+007F     00..7F
 * U+0080..U+07FF     C2..DF   80..BF
 * U+0800..U+0FFF     E0       A0..BF   80..BF
 * U+1000..U+CFFF     E1..EC   80..BF   80..BF
 * U+D000..U+D7FF     ED       80..9F   80..BF
 * U+E000..U+FFFF     EE..EF   80..BF   80..BF
 * U+10000..U+3FFFF   F0       90..BF   80..BF   80..BF
 * U+40000..U+FFFFF   F1..F3   80..BF   80..BF   80..BF
 * U+100000..U+10FFFF F4       80..8F   80..BF   80..BF
 *
 */

// all byte values must be no larger than 0xF4
#[inline]
unsafe fn check_smaller_than_0xf4(current_bytes: __m128i, has_error: *mut __m128i) {
    // unsigned, saturates to 0 below max
    *has_error = _mm_or_si128(
        *has_error,
        _mm_subs_epu8(current_bytes, _mm_set1_epi8(0xf4i32 as libc::c_char)),
    );
}

#[inline]
unsafe fn continuation_lengths(high_nibbles: __m128i) -> __m128i {
    return _mm_shuffle_epi8(
        _mm_setr_epi8(
            1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 0i8, 0i8, 0i8, 0i8, 2i8, 2i8, 3i8, 4i8,
        ),
        high_nibbles,
    );
}

#[inline]
unsafe fn carry_continuations(initial_lengths: __m128i, previous_carries: __m128i) -> __m128i {
    let right1: __m128i = _mm_subs_epu8(
        _mm_alignr_epi8(initial_lengths, previous_carries, 16i32 - 1i32),
        _mm_set1_epi8(1i8),
    );
    let sum: __m128i = _mm_add_epi8(initial_lengths, right1);
    let right2: __m128i = _mm_subs_epu8(
        _mm_alignr_epi8(sum, previous_carries, 16i32 - 2i32),
        _mm_set1_epi8(2i8),
    );
    return _mm_add_epi8(sum, right2);
}

#[inline]
unsafe fn check_continuations(initial_lengths: __m128i, carries: __m128i, has_error: *mut __m128i) {
    // overlap || underlap
    // carry > length && length > 0 || !(carry > length) && !(length > 0)
    // (carries > length) == (lengths > 0)
    let overunder: __m128i = _mm_cmpeq_epi8(
        _mm_cmpgt_epi8(carries, initial_lengths),
        _mm_cmpgt_epi8(initial_lengths, _mm_setzero_si128()),
    );
    *has_error = _mm_or_si128(*has_error, overunder);
}

// when 0xED is found, next byte must be no larger than 0x9F
// when 0xF4 is found, next byte must be no larger than 0x8F
// next byte must be continuation, ie sign bit is set, so signed < is ok
#[inline]
unsafe fn check_first_continuation_max(
    current_bytes: __m128i,
    off1_current_bytes: __m128i,
    has_error: *mut __m128i,
) {
    let mask_ed: __m128i =
        _mm_cmpeq_epi8(off1_current_bytes, _mm_set1_epi8(0xedi32 as libc::c_char));
    let mask_f4: __m128i =
        _mm_cmpeq_epi8(off1_current_bytes, _mm_set1_epi8(0xf4i32 as libc::c_char));
    let bad_follow_ed: __m128i = _mm_and_si128(
        _mm_cmpgt_epi8(current_bytes, _mm_set1_epi8(0x9fi32 as libc::c_char)),
        mask_ed,
    );
    let bad_follow_f4: __m128i = _mm_and_si128(
        _mm_cmpgt_epi8(current_bytes, _mm_set1_epi8(0x8fi32 as libc::c_char)),
        mask_f4,
    );
    *has_error = _mm_or_si128(*has_error, _mm_or_si128(bad_follow_ed, bad_follow_f4));
}

// map off1_hibits => error condition
// hibits     off1    cur
// C       => < C2 && true
// E       => < E1 && < A0
// F       => < F1 && < 90
// else      false && false
#[inline]
unsafe fn check_overlong(
    current_bytes: __m128i,
    off1_current_bytes: __m128i,
    hibits: __m128i,
    previous_hibits: __m128i,
    has_error: *mut __m128i,
) {
    let off1_hibits: __m128i = _mm_alignr_epi8(hibits, previous_hibits, 16i32 - 1i32);
    let initial_mins: __m128i = _mm_shuffle_epi8(
        _mm_setr_epi8(
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            0xc2i32 as libc::c_char,
            -(128i32) as libc::c_char,
            0xe1i32 as libc::c_char,
            0xf1i32 as libc::c_char,
        ),
        off1_hibits,
    );
    let initial_under: __m128i = _mm_cmpgt_epi8(initial_mins, off1_current_bytes);
    let second_mins: __m128i = _mm_shuffle_epi8(
        _mm_setr_epi8(
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            -(128i32) as libc::c_char,
            127i8,
            127i8,
            0xa0i32 as libc::c_char,
            0x90i32 as libc::c_char,
        ),
        off1_hibits,
    );
    let second_under: __m128i = _mm_cmpgt_epi8(second_mins, current_bytes);
    *has_error = _mm_or_si128(*has_error, _mm_and_si128(initial_under, second_under));
}

#[inline]
unsafe fn count_nibbles(bytes: __m128i, mut answer: *mut ProcessedUtfBytes) {
    (*answer).rawbytes = bytes;
    (*answer).high_nibbles = _mm_and_si128(_mm_srli_epi16(bytes, 4i32), _mm_set1_epi8(0xfi8));
}

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
unsafe fn check_utf8_bytes(
    current_bytes: __m128i,
    previous: *mut ProcessedUtfBytes,
    has_error: *mut __m128i,
) -> ProcessedUtfBytes {
    let mut pb = ProcessedUtfBytes::default();
    count_nibbles(current_bytes, &mut pb);
    check_smaller_than_0xf4(current_bytes, has_error);
    let initial_lengths: __m128i = continuation_lengths(pb.high_nibbles);
    pb.carried_continuations =
        carry_continuations(initial_lengths, (*previous).carried_continuations);
    check_continuations(initial_lengths, pb.carried_continuations, has_error);
    let off1_current_bytes: __m128i =
        _mm_alignr_epi8(pb.rawbytes, (*previous).rawbytes, 16i32 - 1i32);
    check_first_continuation_max(current_bytes, off1_current_bytes, has_error);
    check_overlong(
        current_bytes,
        off1_current_bytes,
        pb.high_nibbles,
        (*previous).high_nibbles,
        has_error,
    );
    return pb;
}

pub unsafe fn validate_utf8_fast(src: *const libc::c_char, len: size_t) -> bool {
    let mut i: size_t = 0u64;
    let mut has_error: __m128i = _mm_setzero_si128();
    let mut previous = ProcessedUtfBytes::default();
    if len >= 16u64 {
        while i <= len.wrapping_sub(16u64) {
            let current_bytes: __m128i = _mm_loadu_si128(src.offset(i as isize) as *const __m128i);
            previous = check_utf8_bytes(current_bytes, &mut previous, &mut has_error);
            i = (i).wrapping_add(16u64)
        }
    }
    // last part
    if i < len {
        let mut buffer: [libc::c_char; 16] = [0; 16];
        memset(buffer.as_mut_ptr() as *mut libc::c_void, 0i32, 16u64);
        memcpy(
            buffer.as_mut_ptr() as *mut libc::c_void,
            src.offset(i as isize) as *const libc::c_void,
            len.wrapping_sub(i),
        );
        let current_bytes_0: __m128i = _mm_loadu_si128(buffer.as_mut_ptr() as *const __m128i);
        check_utf8_bytes(current_bytes_0, &mut previous, &mut has_error);
    } else {
        has_error = _mm_or_si128(
            _mm_cmpgt_epi8(
                previous.carried_continuations,
                _mm_setr_epi8(
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 1i8,
                ),
            ),
            has_error,
        )
    }
    return _mm_testz_si128(has_error, has_error) != 0;
}
