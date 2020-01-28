//! # SSE implementation of Lemire's algorithm
//!
//! ## Target specific intrinsics used:
//! ### SSE2
//! * _mm_add_epi8
//! * _mm_and_si128
//! * _mm_cmpeq_epi8
//! * _mm_cmpgt_epi8
//! * _mm_loadu_si128
//! * _mm_or_si128
//! * _mm_set1_epi8
//! * _mm_set_epi8
//! * _mm_setr_epi8
//! * _mm_setzero_si128
//! * _mm_srli_epi16
//! * _mm_subs_epu8
//!
//! ### SSSE3
//! * _mm_alignr_epi8
//! * _mm_shuffle_epi8
//!
//! ### SSE4.1
//! * _mm_testz_si128

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
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::default::Default;
use core::ptr;

#[derive(Copy, Clone)]
struct ProcessedUtfBytes {
    rawbytes: __m128i,
    high_nibbles: __m128i,
    carried_continuations: __m128i,
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

impl ProcessedUtfBytes {
    #[inline]
    fn count_nibbles(&mut self, bytes: __m128i) {
        self.rawbytes = bytes;
        self.high_nibbles = unsafe { _mm_and_si128(_mm_srli_epi16(bytes, 4), _mm_set1_epi8(0xF)) };
    }
}

#[derive(Clone)]
struct State {
    previous: ProcessedUtfBytes,
    has_error: __m128i,
}

impl Default for State {
    fn default() -> Self {
        State {
            previous: ProcessedUtfBytes::default(),
            has_error: unsafe { _mm_setzero_si128() },
        }
    }
}

impl State {
    // check whether the current bytes are valid UTF-8
    // at the end of the function, previous gets updated
    fn check_bytes(&mut self, current_bytes: __m128i) {
        let mut pb = ProcessedUtfBytes::default();
        pb.count_nibbles(current_bytes);
        self.check_smaller_than_0xf4(current_bytes);
        let initial_lengths = continuation_lengths(pb.high_nibbles);
        pb.carried_continuations =
            carry_continuations(initial_lengths, self.previous.carried_continuations);
        self.check_continuations(initial_lengths, pb.carried_continuations);
        let current_bytes_off_by_one =
            unsafe { _mm_alignr_epi8(pb.rawbytes, self.previous.rawbytes, 16 - 1) };
        self.check_first_continuation_max(current_bytes, current_bytes_off_by_one);
        self.check_overlong(
            current_bytes,
            current_bytes_off_by_one,
            pb.high_nibbles,
            self.previous.high_nibbles,
        );
        self.previous = pb;
    }

    // all byte values must be no larger than 0xF4
    #[inline]
    fn check_smaller_than_0xf4(&mut self, current_bytes: __m128i) {
        unsafe {
            // unsigned, saturates to 0 below max
            self.has_error = _mm_or_si128(
                self.has_error,
                _mm_subs_epu8(current_bytes, _mm_set1_epi8(0xF4i32 as i8)),
            );
        }
    }

    // when 0xED is found, next byte must be no larger than 0x9F
    // when 0xF4 is found, next byte must be no larger than 0x8F
    // next byte must be continuation, ie sign bit is set, so signed < is ok
    #[inline]
    fn check_first_continuation_max(
        &mut self,
        current_bytes: __m128i,
        off1_current_bytes: __m128i,
    ) {
        unsafe {
            let mask_ed = _mm_cmpeq_epi8(off1_current_bytes, _mm_set1_epi8(0xEDi32 as i8));
            let mask_f4 = _mm_cmpeq_epi8(off1_current_bytes, _mm_set1_epi8(0xF4i32 as i8));
            let bad_follow_ed = _mm_and_si128(
                _mm_cmpgt_epi8(current_bytes, _mm_set1_epi8(0x9Fi32 as i8)),
                mask_ed,
            );
            let bad_follow_f4 = _mm_and_si128(
                _mm_cmpgt_epi8(current_bytes, _mm_set1_epi8(0x8Fi32 as i8)),
                mask_f4,
            );
            self.has_error =
                _mm_or_si128(self.has_error, _mm_or_si128(bad_follow_ed, bad_follow_f4));
        }
    }

    #[inline]
    fn check_continuations(&mut self, initial_lengths: __m128i, carries: __m128i) {
        unsafe {
            let overunder = _mm_cmpeq_epi8(
                _mm_cmpgt_epi8(carries, initial_lengths),
                _mm_cmpgt_epi8(initial_lengths, _mm_setzero_si128()),
            );
            self.has_error = _mm_or_si128(self.has_error, overunder);
        }
    }

    // map off1_hibits => error condition
    // hibits     off1    cur
    // C       => < C2 && true
    // E       => < E1 && < A0
    // F       => < F1 && < 90
    // else      false && false
    #[inline]
    fn check_overlong(
        &mut self,
        current_bytes: __m128i,
        off1_current_bytes: __m128i,
        hibits: __m128i,
        previous_hibits: __m128i,
    ) {
        unsafe {
            let off1_hibits = _mm_alignr_epi8(hibits, previous_hibits, 16 - 1);
            let initial_mins = _mm_shuffle_epi8(
                _mm_setr_epi8(
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    0xC2i32 as i8,
                    -128,
                    0xE1i32 as i8,
                    0xF1i32 as i8,
                ),
                off1_hibits,
            );
            let initial_under = _mm_cmpgt_epi8(initial_mins, off1_current_bytes);
            let second_mins = _mm_shuffle_epi8(
                _mm_setr_epi8(
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    -128,
                    127,
                    127,
                    0xA0i32 as i8,
                    0x90i32 as i8,
                ),
                off1_hibits,
            );
            let second_under = _mm_cmpgt_epi8(second_mins, current_bytes);
            self.has_error =
                _mm_or_si128(self.has_error, _mm_and_si128(initial_under, second_under));
        }
    }

    fn is_erroneous(self) -> bool {
        unsafe { _mm_testz_si128(self.has_error, self.has_error) != 0 }
    }
}

#[inline]
fn continuation_lengths(high_nibbles: __m128i) -> __m128i {
    unsafe {
        _mm_shuffle_epi8(
            _mm_setr_epi8(1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4),
            high_nibbles,
        )
    }
}

#[inline]
fn carry_continuations(initial_lengths: __m128i, previous_carries: __m128i) -> __m128i {
    unsafe {
        let right1 = _mm_subs_epu8(
            _mm_alignr_epi8(initial_lengths, previous_carries, 16 - 1),
            _mm_set1_epi8(1),
        );
        let sum = _mm_add_epi8(initial_lengths, right1);
        let right2 = _mm_subs_epu8(
            _mm_alignr_epi8(sum, previous_carries, 16 - 2),
            _mm_set1_epi8(2),
        );
        _mm_add_epi8(sum, right2)
    }
}

pub fn is_utf8(bytes: &[u8]) -> bool {
    let len = bytes.len();
    let mut i = 0;

    let mut state = State::default();

    if len >= 16 {
        while i <= len - 16 {
            let current_bytes =
                unsafe { _mm_loadu_si128(bytes.as_ptr().offset(i as isize) as *const __m128i) };
            state.check_bytes(current_bytes);
            i += 16
        }
    }
    // last part
    if i < len {
        let mut buffer = [0; 16];
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr().offset(i as isize),
                buffer.as_mut_ptr(),
                len - i,
            );
            let current_bytes_0 = _mm_loadu_si128(buffer.as_mut_ptr() as *const __m128i);
            state.check_bytes(current_bytes_0);
        }
    } else {
        unsafe {
            state.has_error = _mm_or_si128(
                _mm_cmpgt_epi8(
                    state.previous.carried_continuations,
                    _mm_setr_epi8(9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 1),
                ),
                state.has_error,
            )
        }
    }

    state.is_erroneous()
}
