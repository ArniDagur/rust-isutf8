//! # AVX implementation of Lemire's algorithm
//!
//! ## Target specific intrinsics used:
//! ### AVX
//! * _mm256_loadu_si256
//! * _mm256_set1_epi8
//! * _mm256_set_epi8
//! * _mm256_setr_epi8
//! * _mm256_setzero_si256
//! * _mm256_testz_si256
//!
//! ### AVX2
//! * _mm256_add_epi8
//! * _mm256_alignr_epi8
//! * _mm256_and_si256
//! * _mm256_cmpeq_epi8
//! * _mm256_cmpgt_epi8
//! * _mm256_or_si256
//! * _mm256_permute2x128_si256
//! * _mm256_shuffle_epi8
//! * _mm256_srli_epi16
//! * _mm256_subs_epu8
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
    rawbytes: __m256i,
    high_nibbles: __m256i,
    carried_continuations: __m256i,
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

impl ProcessedUtfBytes {
    #[inline]
    fn count_nibbles(&mut self, bytes: __m256i) {
        self.rawbytes = bytes;
        self.high_nibbles =
            unsafe { _mm256_and_si256(_mm256_srli_epi16(bytes, 4), _mm256_set1_epi8(0xF)) };
    }
}

#[derive(Clone)]
struct State {
    previous: ProcessedUtfBytes,
    has_error: __m256i,
}

impl Default for State {
    fn default() -> Self {
        State {
            previous: ProcessedUtfBytes::default(),
            has_error: unsafe { _mm256_setzero_si256() },
        }
    }
}

impl State {
    // check whether the current bytes are valid UTF-8
    // at the end of the function, previous gets updated
    fn check_bytes(&mut self, current_bytes: __m256i) {
        let mut pb = ProcessedUtfBytes::default();
        pb.count_nibbles(current_bytes);
        self.check_smaller_than_0xf4(current_bytes);
        let initial_lengths = continuation_lengths(pb.high_nibbles);
        pb.carried_continuations =
            carry_continuations(initial_lengths, self.previous.carried_continuations);
        self.check_continuations(initial_lengths, pb.carried_continuations);
        let off1_current_bytes = push_last_byte_of_a_to_b(self.previous.rawbytes, pb.rawbytes);
        self.check_first_continuation_max(current_bytes, off1_current_bytes);
        self.check_overlong(
            current_bytes,
            off1_current_bytes,
            pb.high_nibbles,
            self.previous.high_nibbles,
        );
        self.previous = pb;
    }

    // check whether the current bytes are valid UTF-8
    // at the end of the function, previous gets updated
    fn check_bytes_ascii_path(&mut self, current_bytes: __m256i) {
        if no_most_significant_bits(current_bytes) {
            unsafe {
                // Fast ascii path
                self.has_error = _mm256_or_si256(
                    _mm256_cmpgt_epi8(
                        self.previous.carried_continuations,
                        _mm256_setr_epi8(
                            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                            9, 9, 9, 9, 9, 9, 9, 1,
                        ),
                    ),
                    self.has_error,
                );
                return;
            }
        }
        // Slow non-ascii path
        self.check_bytes(current_bytes);
    }

    #[inline]
    fn check_continuations(&mut self, initial_lengths: __m256i, carries: __m256i) {
        unsafe {
            // overlap || underlap
            // carry > length && length > 0 || !(carry > length) && !(length > 0)
            // (carries > length) == (lengths > 0)
            let overunder = _mm256_cmpeq_epi8(
                _mm256_cmpgt_epi8(carries, initial_lengths),
                _mm256_cmpgt_epi8(initial_lengths, _mm256_setzero_si256()),
            );
            self.has_error = _mm256_or_si256(self.has_error, overunder);
        }
    }

    // when 0xED is found, next byte must be no larger than 0x9F
    // when 0xF4 is found, next byte must be no larger than 0x8F
    // next byte must be continuation, ie sign bit is set, so signed < is ok
    #[inline]
    fn check_first_continuation_max(
        &mut self,
        current_bytes: __m256i,
        off1_current_bytes: __m256i,
    ) {
        unsafe {
            let mask_ed = _mm256_cmpeq_epi8(off1_current_bytes, _mm256_set1_epi8(0xEDi32 as i8));
            let mask_f4 = _mm256_cmpeq_epi8(off1_current_bytes, _mm256_set1_epi8(0xF4i32 as i8));
            let bad_follow_ed = _mm256_and_si256(
                _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(0x9Fi32 as i8)),
                mask_ed,
            );
            let bad_follow_f4 = _mm256_and_si256(
                _mm256_cmpgt_epi8(current_bytes, _mm256_set1_epi8(0x8Fi32 as i8)),
                mask_f4,
            );
            self.has_error = _mm256_or_si256(
                self.has_error,
                _mm256_or_si256(bad_follow_ed, bad_follow_f4),
            );
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
        current_bytes: __m256i,
        off1_current_bytes: __m256i,
        hibits: __m256i,
        previous_hibits: __m256i,
    ) {
        unsafe {
            let off1_hibits = push_last_byte_of_a_to_b(previous_hibits, hibits);
            let initial_mins = _mm256_shuffle_epi8(
                _mm256_setr_epi8(
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
            let initial_under = _mm256_cmpgt_epi8(initial_mins, off1_current_bytes);
            let second_mins = _mm256_shuffle_epi8(
                _mm256_setr_epi8(
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
            let second_under = _mm256_cmpgt_epi8(second_mins, current_bytes);
            self.has_error = _mm256_or_si256(
                self.has_error,
                _mm256_and_si256(initial_under, second_under),
            );
        }
    }

    // all byte values must be no larger than 0xF4
    #[inline]
    fn check_smaller_than_0xf4(&mut self, current_bytes: __m256i) {
        unsafe {
            // unsigned, saturates to 0 below max
            self.has_error = _mm256_or_si256(
                self.has_error,
                _mm256_subs_epu8(current_bytes, _mm256_set1_epi8(0xF4i32 as i8)),
            );
        }
    }

    #[inline]
    fn is_erroneous(&mut self) -> bool {
        unsafe { _mm256_testz_si256(self.has_error, self.has_error) != 0 }
    }
}

/// Return `true` if none of the bytes given have their most significant bit
/// set to `1`.
#[inline]
fn no_most_significant_bits(bytes: __m256i) -> bool {
    unsafe { _mm256_testz_si256(bytes, _mm256_set1_epi8(0x80i32 as i8)) != 0 }
}

#[inline]
fn push_last_byte_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe {
        return _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 15);
    }
}

#[inline]
fn push_last_2bytes_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe {
        return _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 14);
    }
}

#[inline]
fn continuation_lengths(high_nibbles: __m256i) -> __m256i {
    unsafe {
        return _mm256_shuffle_epi8(
            _mm256_setr_epi8(
                1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 3, 4, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
                2, 2, 3, 4,
            ),
            high_nibbles,
        );
    }
}

#[inline]
fn carry_continuations(initial_lengths: __m256i, previous_carries: __m256i) -> __m256i {
    unsafe {
        let right1 = _mm256_subs_epu8(
            push_last_byte_of_a_to_b(previous_carries, initial_lengths),
            _mm256_set1_epi8(1),
        );
        let sum = _mm256_add_epi8(initial_lengths, right1);
        let right2 = _mm256_subs_epu8(
            push_last_2bytes_of_a_to_b(previous_carries, sum),
            _mm256_set1_epi8(2),
        );
        return _mm256_add_epi8(sum, right2);
    }
}

pub fn is_utf8_ascii_path(bytes: &[u8]) -> bool {
    let len = bytes.len();
    let mut i = 0;

    let mut state = State::default();

    if len >= 32 {
        while i <= len - 32 {
            let current_bytes =
                unsafe { _mm256_loadu_si256(bytes.as_ptr().offset(i as isize) as *const __m256i) };
            state.check_bytes_ascii_path(current_bytes);
            i += 32;
        }
    }
    // last part
    if i < len {
        unsafe {
            let mut buffer = [0; 32];
            ptr::write_bytes(buffer.as_mut_ptr(), 0, 32);
            ptr::copy_nonoverlapping(
                bytes.as_ptr().offset(i as isize),
                buffer.as_mut_ptr(),
                len - i,
            );
            let current_bytes_0 = _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
            state.check_bytes(current_bytes_0);
        }
    } else {
        unsafe {
            state.has_error = _mm256_or_si256(
                _mm256_cmpgt_epi8(
                    state.previous.carried_continuations,
                    _mm256_setr_epi8(
                        9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                        9, 9, 9, 9, 9, 9, 1,
                    ),
                ),
                state.has_error,
            )
        }
    }

    state.is_erroneous()
}

pub fn is_utf8(bytes: &[u8]) -> bool {
    let len = bytes.len();
    let mut i = 0;

    let mut state = State::default();

    if len >= 32 {
        while i <= len - 32 {
            let current_bytes =
                unsafe { _mm256_loadu_si256(bytes.as_ptr().offset(i as isize) as *const __m256i) };
            state.check_bytes(current_bytes);
            i += 32
        }
    }
    // last part
    if i < len {
        let mut buffer = [0; 32];
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr().offset(i as isize),
                buffer.as_mut_ptr(),
                len - i,
            );
            let current_bytes_0 = _mm256_loadu_si256(buffer.as_mut_ptr() as *const __m256i);
            state.check_bytes(current_bytes_0);
        }
    } else {
        unsafe {
            state.has_error = _mm256_or_si256(
                _mm256_cmpgt_epi8(
                    state.previous.carried_continuations,
                    _mm256_setr_epi8(
                        9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                        9, 9, 9, 9, 9, 9, 1,
                    ),
                ),
                state.has_error,
            )
        }
    }

    state.is_erroneous()
}
