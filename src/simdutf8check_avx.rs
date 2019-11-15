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
pub type size_t = libc::c_ulong;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct __loadu_si256 {
    pub __v: __m256i,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct avx_processed_utf_bytes {
    pub rawbytes: __m256i,
    pub high_nibbles: __m256i,
    pub carried_continuations: __m256i,
}

impl Default for avx_processed_utf_bytes {
    fn default() -> Self {
        unsafe {
            avx_processed_utf_bytes {
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
unsafe fn push_last_byte_of_a_to_b(mut a: __m256i, mut b: __m256i) -> __m256i {
    return _mm256_alignr_epi8(
        b,
        _mm256_permute2x128_si256(a, b, 0x21 as libc::c_int),
        15 as libc::c_int,
    );
}

#[inline]
unsafe fn push_last_2bytes_of_a_to_b(mut a: __m256i, mut b: __m256i) -> __m256i {
    return _mm256_alignr_epi8(
        b,
        _mm256_permute2x128_si256(a, b, 0x21 as libc::c_int),
        14 as libc::c_int,
    );
}

// all byte values must be no larger than 0xF4
#[inline]
unsafe fn avxcheckSmallerThan0xF4(mut current_bytes: __m256i, mut has_error: *mut __m256i) {
    // unsigned, saturates to 0 below max
    *has_error = _mm256_or_si256(
        *has_error,
        _mm256_subs_epu8(
            current_bytes,
            _mm256_set1_epi8(0xf4 as libc::c_int as libc::c_char),
        ),
    );
}

#[inline]
unsafe fn avxcontinuationLengths(mut high_nibbles: __m256i) -> __m256i {
    return _mm256_shuffle_epi8(
        _mm256_setr_epi8(
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            2 as libc::c_int as libc::c_char,
            2 as libc::c_int as libc::c_char,
            3 as libc::c_int as libc::c_char,
            4 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            1 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            0 as libc::c_int as libc::c_char,
            2 as libc::c_int as libc::c_char,
            2 as libc::c_int as libc::c_char,
            3 as libc::c_int as libc::c_char,
            4 as libc::c_int as libc::c_char,
        ),
        high_nibbles,
    );
}

#[inline]
unsafe fn avxcarryContinuations(
    mut initial_lengths: __m256i,
    mut previous_carries: __m256i,
) -> __m256i {
    let mut right1: __m256i = _mm256_subs_epu8(
        push_last_byte_of_a_to_b(previous_carries, initial_lengths),
        _mm256_set1_epi8(1 as libc::c_int as libc::c_char),
    );
    let mut sum: __m256i = _mm256_add_epi8(initial_lengths, right1);
    let mut right2: __m256i = _mm256_subs_epu8(
        push_last_2bytes_of_a_to_b(previous_carries, sum),
        _mm256_set1_epi8(2 as libc::c_int as libc::c_char),
    );
    return _mm256_add_epi8(sum, right2);
}

#[inline]
unsafe fn avxcheckContinuations(
    mut initial_lengths: __m256i,
    mut carries: __m256i,
    mut has_error: *mut __m256i,
) {
    // overlap || underlap
    // carry > length && length > 0 || !(carry > length) && !(length > 0)
    // (carries > length) == (lengths > 0)
    let mut overunder: __m256i = _mm256_cmpeq_epi8(
        _mm256_cmpgt_epi8(carries, initial_lengths),
        _mm256_cmpgt_epi8(initial_lengths, _mm256_setzero_si256()),
    );
    *has_error = _mm256_or_si256(*has_error, overunder);
}

// when 0xED is found, next byte must be no larger than 0x9F
// when 0xF4 is found, next byte must be no larger than 0x8F
// next byte must be continuation, ie sign bit is set, so signed < is ok
#[inline]
unsafe fn avxcheckFirstContinuationMax(
    mut current_bytes: __m256i,
    mut off1_current_bytes: __m256i,
    mut has_error: *mut __m256i,
) {
    let mut maskED: __m256i = _mm256_cmpeq_epi8(
        off1_current_bytes,
        _mm256_set1_epi8(0xed as libc::c_int as libc::c_char),
    );
    let mut maskF4: __m256i = _mm256_cmpeq_epi8(
        off1_current_bytes,
        _mm256_set1_epi8(0xf4 as libc::c_int as libc::c_char),
    );
    let mut badfollowED: __m256i = _mm256_and_si256(
        _mm256_cmpgt_epi8(
            current_bytes,
            _mm256_set1_epi8(0x9f as libc::c_int as libc::c_char),
        ),
        maskED,
    );
    let mut badfollowF4: __m256i = _mm256_and_si256(
        _mm256_cmpgt_epi8(
            current_bytes,
            _mm256_set1_epi8(0x8f as libc::c_int as libc::c_char),
        ),
        maskF4,
    );
    *has_error = _mm256_or_si256(*has_error, _mm256_or_si256(badfollowED, badfollowF4));
}

// map off1_hibits => error condition
// hibits     off1    cur
// C       => < C2 && true
// E       => < E1 && < A0
// F       => < F1 && < 90
// else      false && false
#[inline]
unsafe fn avxcheckOverlong(
    mut current_bytes: __m256i,
    mut off1_current_bytes: __m256i,
    mut hibits: __m256i,
    mut previous_hibits: __m256i,
    mut has_error: *mut __m256i,
) {
    let mut off1_hibits: __m256i = push_last_byte_of_a_to_b(previous_hibits, hibits);
    let mut initial_mins: __m256i = _mm256_shuffle_epi8(
        _mm256_setr_epi8(
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            0xc2 as libc::c_int as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            0xe1 as libc::c_int as libc::c_char,
            0xf1 as libc::c_int as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            0xc2 as libc::c_int as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            0xe1 as libc::c_int as libc::c_char,
            0xf1 as libc::c_int as libc::c_char,
        ),
        off1_hibits,
    );
    let mut initial_under: __m256i = _mm256_cmpgt_epi8(initial_mins, off1_current_bytes);
    let mut second_mins: __m256i = _mm256_shuffle_epi8(
        _mm256_setr_epi8(
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            127 as libc::c_int as libc::c_char,
            127 as libc::c_int as libc::c_char,
            0xa0 as libc::c_int as libc::c_char,
            0x90 as libc::c_int as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            -(128 as libc::c_int) as libc::c_char,
            127 as libc::c_int as libc::c_char,
            127 as libc::c_int as libc::c_char,
            0xa0 as libc::c_int as libc::c_char,
            0x90 as libc::c_int as libc::c_char,
        ),
        off1_hibits,
    );
    let mut second_under: __m256i = _mm256_cmpgt_epi8(second_mins, current_bytes);
    *has_error = _mm256_or_si256(*has_error, _mm256_and_si256(initial_under, second_under));
}

#[inline]
unsafe fn avx_count_nibbles(mut bytes: __m256i, mut answer: *mut avx_processed_utf_bytes) {
    (*answer).rawbytes = bytes;
    (*answer).high_nibbles = _mm256_and_si256(
        _mm256_srli_epi16(bytes, 4 as libc::c_int),
        _mm256_set1_epi8(0xf as libc::c_int as libc::c_char),
    );
}

// check whether the current bytes are valid UTF-8
// at the end of the function, previous gets updated
pub unsafe fn avxcheckUTF8Bytes(
    mut current_bytes: __m256i,
    mut previous: *mut avx_processed_utf_bytes,
    mut has_error: *mut __m256i,
) -> avx_processed_utf_bytes {
    let mut pb: avx_processed_utf_bytes = avx_processed_utf_bytes::default();
    avx_count_nibbles(current_bytes, &mut pb);
    avxcheckSmallerThan0xF4(current_bytes, has_error);
    let mut initial_lengths: __m256i = avxcontinuationLengths(pb.high_nibbles);
    pb.carried_continuations =
        avxcarryContinuations(initial_lengths, (*previous).carried_continuations);
    avxcheckContinuations(initial_lengths, pb.carried_continuations, has_error);
    let mut off1_current_bytes: __m256i =
        push_last_byte_of_a_to_b((*previous).rawbytes, pb.rawbytes);
    avxcheckFirstContinuationMax(current_bytes, off1_current_bytes, has_error);
    avxcheckOverlong(
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
pub unsafe fn avxcheckUTF8Bytes_asciipath(
    mut current_bytes: __m256i,
    mut previous: *mut avx_processed_utf_bytes,
    mut has_error: *mut __m256i,
) -> avx_processed_utf_bytes {
    if _mm256_testz_si256(
        current_bytes,
        _mm256_set1_epi8(0x80 as libc::c_int as libc::c_char),
    ) != 0
    {
        // fast ascii path
        *has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                (*previous).carried_continuations,
                _mm256_setr_epi8(
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    1 as libc::c_int as libc::c_char,
                ),
            ),
            *has_error,
        );
        return *previous;
    }
    let mut pb: avx_processed_utf_bytes = avx_processed_utf_bytes {
        rawbytes: _mm256_setzero_si256(),
        high_nibbles: _mm256_setzero_si256(),
        carried_continuations: _mm256_setzero_si256(),
    };
    avx_count_nibbles(current_bytes, &mut pb);
    avxcheckSmallerThan0xF4(current_bytes, has_error);
    let mut initial_lengths: __m256i = avxcontinuationLengths(pb.high_nibbles);
    pb.carried_continuations =
        avxcarryContinuations(initial_lengths, (*previous).carried_continuations);
    avxcheckContinuations(initial_lengths, pb.carried_continuations, has_error);
    let mut off1_current_bytes: __m256i =
        push_last_byte_of_a_to_b((*previous).rawbytes, pb.rawbytes);
    avxcheckFirstContinuationMax(current_bytes, off1_current_bytes, has_error);
    avxcheckOverlong(
        current_bytes,
        off1_current_bytes,
        pb.high_nibbles,
        (*previous).high_nibbles,
        has_error,
    );
    return pb;
}

pub unsafe fn validate_utf8_fast_avx_asciipath(
    mut src: *const libc::c_char,
    mut len: size_t,
) -> bool {
    let mut i: size_t = 0 as libc::c_int as size_t;
    let mut has_error: __m256i = _mm256_setzero_si256();
    let mut previous: avx_processed_utf_bytes = {
        let mut init = avx_processed_utf_bytes::default();
        init
    };
    if len >= 32 as libc::c_int as libc::c_ulong {
        while i <= len.wrapping_sub(32 as libc::c_int as libc::c_ulong) {
            let mut current_bytes: __m256i =
                _mm256_loadu_si256(src.offset(i as isize) as *const __m256i);
            previous = avxcheckUTF8Bytes_asciipath(current_bytes, &mut previous, &mut has_error);
            i = (i as libc::c_ulong).wrapping_add(32 as libc::c_int as libc::c_ulong) as size_t
                as size_t
        }
    }
    // last part
    if i < len {
        let mut buffer: [libc::c_char; 32] = [0; 32];
        memset(
            buffer.as_mut_ptr() as *mut libc::c_void,
            0 as libc::c_int,
            32 as libc::c_int as libc::c_ulong,
        );
        memcpy(
            buffer.as_mut_ptr() as *mut libc::c_void,
            src.offset(i as isize) as *const libc::c_void,
            len.wrapping_sub(i),
        );
        let mut current_bytes_0: __m256i =
            _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
        previous = avxcheckUTF8Bytes(current_bytes_0, &mut previous, &mut has_error)
    } else {
        has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                previous.carried_continuations,
                _mm256_setr_epi8(
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    1 as libc::c_int as libc::c_char,
                ),
            ),
            has_error,
        )
    }
    return _mm256_testz_si256(has_error, has_error) != 0;
}

pub unsafe fn validate_utf8_fast_avx(mut src: *const libc::c_char, mut len: size_t) -> bool {
    let mut i: size_t = 0 as libc::c_int as size_t;
    let mut has_error: __m256i = _mm256_setzero_si256();
    let mut previous: avx_processed_utf_bytes = {
        let mut init = avx_processed_utf_bytes::default();
        init
    };
    if len >= 32 as libc::c_int as libc::c_ulong {
        while i <= len.wrapping_sub(32 as libc::c_int as libc::c_ulong) {
            let mut current_bytes: __m256i =
                _mm256_loadu_si256(src.offset(i as isize) as *const __m256i);
            previous = avxcheckUTF8Bytes(current_bytes, &mut previous, &mut has_error);
            i = (i as libc::c_ulong).wrapping_add(32 as libc::c_int as libc::c_ulong) as size_t
                as size_t
        }
    }
    // last part
    if i < len {
        let mut buffer: [libc::c_char; 32] = [0; 32];
        memset(
            buffer.as_mut_ptr() as *mut libc::c_void,
            0 as libc::c_int,
            32 as libc::c_int as libc::c_ulong,
        );
        memcpy(
            buffer.as_mut_ptr() as *mut libc::c_void,
            src.offset(i as isize) as *const libc::c_void,
            len.wrapping_sub(i),
        );
        let mut current_bytes_0: __m256i =
            _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
        previous = avxcheckUTF8Bytes(current_bytes_0, &mut previous, &mut has_error)
    } else {
        has_error = _mm256_or_si256(
            _mm256_cmpgt_epi8(
                previous.carried_continuations,
                _mm256_setr_epi8(
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    9 as libc::c_int as libc::c_char,
                    1 as libc::c_int as libc::c_char,
                ),
            ),
            has_error,
        )
    }
    return _mm256_testz_si256(has_error, has_error) != 0;
}
