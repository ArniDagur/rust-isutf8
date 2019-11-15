#[cfg(target_arch = "x86")]
pub use core::arch::x86::{
    __m256i, _mm256_add_epi8, _mm256_alignr_epi8, _mm256_and_si256, _mm256_cmpeq_epi8,
    _mm256_cmpgt_epi8, _mm256_loadu_si256, _mm256_or_si256, _mm256_permute2x128_si256,
    _mm256_set1_epi8, _mm256_set_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
    _mm256_srli_epi16, _mm256_subs_epu8, _mm256_testz_si256,
};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{
    __m256i, _mm256_add_epi8, _mm256_alignr_epi8, _mm256_and_si256, _mm256_cmpeq_epi8,
    _mm256_cmpgt_epi8, _mm256_loadu_si256, _mm256_or_si256, _mm256_permute2x128_si256,
    _mm256_set1_epi8, _mm256_set_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
    _mm256_srli_epi16, _mm256_subs_epu8, _mm256_testz_si256,
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
pub struct __loadu_si256 {
    pub __v: __m256i,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessedUtfBytes {
    pub rawbytes: __m256i,
    pub high_nibbles: __m256i,
    pub carried_continuations: __m256i,
}

impl Default for ProcessedUtfBytes {
    fn default() -> Self {
        unsafe {
            ProcessedUtfBytes {
                rawbytes: _mm256_setzero_si256(),
                high_nibbles: _mm256_setzero_si256(),
                carried_continuations: _mm256_setzero_si256(),
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
#[inline]
unsafe fn push_last_byte_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    return _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21i32), 15i32);
}

#[inline]
unsafe fn push_last_2bytes_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    return _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21i32), 14i32);
}

// all byte values must be no larger than 0xF4
#[inline]
unsafe fn check_smaller_than_0xf4(current_bytes: __m256i, has_error: *mut __m256i) {
    // unsigned, saturates to 0 below max
    *has_error = _mm256_or_si256(
        *has_error,
        _mm256_subs_epu8(current_bytes, _mm256_set1_epi8(0xf4i32 as libc::c_char)),
    );
}

#[inline]
unsafe fn continuation_lengths(high_nibbles: __m256i) -> __m256i {
    return _mm256_shuffle_epi8(
        _mm256_setr_epi8(
            1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 0i8, 0i8, 0i8, 0i8, 2i8, 2i8, 3i8, 4i8, 1i8,
            1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 1i8, 0i8, 0i8, 0i8, 0i8, 2i8, 2i8, 3i8, 4i8,
        ),
        high_nibbles,
    );
}

#[inline]
unsafe fn carry_continuations(initial_lengths: __m256i, previous_carries: __m256i) -> __m256i {
    let right1: __m256i = _mm256_subs_epu8(
        push_last_byte_of_a_to_b(previous_carries, initial_lengths),
        _mm256_set1_epi8(1i8),
    );
    let sum: __m256i = _mm256_add_epi8(initial_lengths, right1);
    let right2: __m256i = _mm256_subs_epu8(
        push_last_2bytes_of_a_to_b(previous_carries, sum),
        _mm256_set1_epi8(2i8),
    );
    return _mm256_add_epi8(sum, right2);
}

#[inline]
unsafe fn check_continuations(initial_lengths: __m256i, carries: __m256i, has_error: *mut __m256i) {
    // overlap || underlap
    // carry > length && length > 0 || !(carry > length) && !(length > 0)
    // (carries > length) == (lengths > 0)
    let overunder: __m256i = _mm256_cmpeq_epi8(
        _mm256_cmpgt_epi8(carries, initial_lengths),
        _mm256_cmpgt_epi8(initial_lengths, _mm256_setzero_si256()),
    );
    *has_error = _mm256_or_si256(*has_error, overunder);
}

// when 0xED is found, next byte must be no larger than 0x9F
// when 0xF4 is found, next byte must be no larger than 0x8F
// next byte must be continuation, ie sign bit is set, so signed < is ok
#[inline]
unsafe fn check_first_continuation_max(
    current_bytes: __m256i,
    off1_current_bytes: __m256i,
    has_error: *mut __m256i,
) {
    let mask_ed: __m256i = _mm256_cmpeq_epi8(
        off1_current_bytes,
        _mm256_set1_epi8(0xedi32 as libc::c_char),
    );
    let mask_f4: __m256i = _mm256_cmpeq_epi8(
        off1_current_bytes,
        _mm256_set1_epi8(0xf4i32 as libc::c_char),
    );
    let bad_follow_ed: __m256i = _mm256_and_si256(
        _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(0x9fi32 as libc::c_char)),
        mask_ed,
    );
    let bad_follow_f4: __m256i = _mm256_and_si256(
        _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(0x8fi32 as libc::c_char)),
        mask_f4,
    );
    *has_error = _mm256_or_si256(*has_error, _mm256_or_si256(bad_follow_ed, bad_follow_f4));
}

// map off1_hibits => error condition
// hibits     off1    cur
// C       => < C2 && true
// E       => < E1 && < A0
// F       => < F1 && < 90
// else      false && false
#[inline]
unsafe fn check_overlong(
    current_bytes: __m256i,
    off1_current_bytes: __m256i,
    hibits: __m256i,
    previous_hibits: __m256i,
    has_error: *mut __m256i,
) {
    let off1_hibits: __m256i = push_last_byte_of_a_to_b(previous_hibits, hibits);
    let initial_mins: __m256i = _mm256_shuffle_epi8(
        _mm256_setr_epi8(
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
    let initial_under: __m256i = _mm256_cmpgt_epi8(initial_mins, off1_current_bytes);
    let second_mins: __m256i = _mm256_shuffle_epi8(
        _mm256_setr_epi8(
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
    let second_under: __m256i = _mm256_cmpgt_epi8(second_mins, current_bytes);
    *has_error = _mm256_or_si256(*has_error, _mm256_and_si256(initial_under, second_under));
}

#[inline]
unsafe fn count_nibbles(bytes: __m256i, mut answer: *mut ProcessedUtfBytes) {
    (*answer).rawbytes = bytes;
    (*answer).high_nibbles =
        _mm256_and_si256(_mm256_srli_epi16(bytes, 4i32), _mm256_set1_epi8(0xfi8));
}

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
pub unsafe fn check_utf8_bytes(
    current_bytes: __m256i,
    previous: *mut ProcessedUtfBytes,
    has_error: *mut __m256i,
) -> ProcessedUtfBytes {
    let mut pb: ProcessedUtfBytes = ProcessedUtfBytes::default();
    count_nibbles(current_bytes, &mut pb);
    check_smaller_than_0xf4(current_bytes, has_error);
    let initial_lengths: __m256i = continuation_lengths(pb.high_nibbles);
    pb.carried_continuations =
        carry_continuations(initial_lengths, (*previous).carried_continuations);
    check_continuations(initial_lengths, pb.carried_continuations, has_error);
    let off1_current_bytes: __m256i = push_last_byte_of_a_to_b((*previous).rawbytes, pb.rawbytes);
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

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
pub unsafe fn check_utf8_bytes_ascii_path(
    current_bytes: __m256i,
    previous: *mut ProcessedUtfBytes,
    has_error: *mut __m256i,
) -> ProcessedUtfBytes {
    if _mm256_testz_si256(current_bytes, _mm256_set1_epi8(0x80i32 as libc::c_char)) != 0 {
        // fast ascii path
        *has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                (*previous).carried_continuations,
                _mm256_setr_epi8(
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8,
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 1i8,
                ),
            ),
            *has_error,
        );
        return *previous;
    }
    let mut pb = ProcessedUtfBytes::default();
    count_nibbles(current_bytes, &mut pb);
    check_smaller_than_0xf4(current_bytes, has_error);
    let initial_lengths: __m256i = continuation_lengths(pb.high_nibbles);
    pb.carried_continuations =
        carry_continuations(initial_lengths, (*previous).carried_continuations);
    check_continuations(initial_lengths, pb.carried_continuations, has_error);
    let off1_current_bytes: __m256i = push_last_byte_of_a_to_b((*previous).rawbytes, pb.rawbytes);
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

pub unsafe fn validate_utf8_fast_ascii_path(src: *const libc::c_char, len: size_t) -> bool {
    let mut i: size_t = 0u64;
    let mut has_error: __m256i = _mm256_setzero_si256();
    let mut previous = ProcessedUtfBytes::default();
    if len >= 32u64 {
        while i <= len.wrapping_sub(32u64) {
            let current_bytes: __m256i =
                _mm256_loadu_si256(src.offset(i as isize) as *const __m256i);
            previous = check_utf8_bytes_ascii_path(current_bytes, &mut previous, &mut has_error);
            i = (i).wrapping_add(32u64)
        }
    }
    // last part
    if i < len {
        let mut buffer: [libc::c_char; 32] = [0; 32];
        memset(buffer.as_mut_ptr() as *mut libc::c_void, 0i32, 32u64);
        memcpy(
            buffer.as_mut_ptr() as *mut libc::c_void,
            src.offset(i as isize) as *const libc::c_void,
            len.wrapping_sub(i),
        );
        let current_bytes_0: __m256i = _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
        check_utf8_bytes(current_bytes_0, &mut previous, &mut has_error);
    } else {
        has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                previous.carried_continuations,
                _mm256_setr_epi8(
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8,
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 1i8,
                ),
            ),
            has_error,
        )
    }
    return _mm256_testz_si256(has_error, has_error) != 0;
}

pub unsafe fn validate_utf8_fast(src: *const libc::c_char, len: size_t) -> bool {
    let mut i: size_t = 0u64;
    let mut has_error: __m256i = _mm256_setzero_si256();
    let mut previous = ProcessedUtfBytes::default();
    if len >= 32u64 {
        while i <= len.wrapping_sub(32u64) {
            let current_bytes: __m256i =
                _mm256_loadu_si256(src.offset(i as isize) as *const __m256i);
            previous = check_utf8_bytes(current_bytes, &mut previous, &mut has_error);
            i = (i).wrapping_add(32u64)
        }
    }
    // last part
    if i < len {
        let mut buffer: [libc::c_char; 32] = [0; 32];
        memset(buffer.as_mut_ptr() as *mut libc::c_void, 0i32, 32u64);
        memcpy(
            buffer.as_mut_ptr() as *mut libc::c_void,
            src.offset(i as isize) as *const libc::c_void,
            len.wrapping_sub(i),
        );
        let current_bytes_0: __m256i = _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
        check_utf8_bytes(current_bytes_0, &mut previous, &mut has_error);
    } else {
        has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                previous.carried_continuations,
                _mm256_setr_epi8(
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8,
                    9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 9i8, 1i8,
                ),
            ),
            has_error,
        )
    }
    return _mm256_testz_si256(has_error, has_error) != 0;
}
