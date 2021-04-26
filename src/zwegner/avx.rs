#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_add_epi32, _mm256_and_si256, _mm256_andnot_si256, _mm256_lddqu_si256,
    _mm256_load_si256, _mm256_movemask_epi8, _mm256_or_si256, _mm256_set1_epi8, _mm256_setr_epi8,
    _mm256_setzero_si256, _mm256_shuffle_epi8, _mm256_slli_epi16, _mm256_srli_epi16,
    _mm256_testz_si256,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_add_epi32, _mm256_and_si256, _mm256_andnot_si256, _mm256_lddqu_si256,
    _mm256_load_si256, _mm256_movemask_epi8, _mm256_or_si256, _mm256_set1_epi8, _mm256_setr_epi8,
    _mm256_setzero_si256, _mm256_shuffle_epi8, _mm256_slli_epi16, _mm256_srli_epi16,
    _mm256_testz_si256, _mm256_loadu_si256
};

/// This function is yoinked from unstable Rust.
fn wrapping_offset_from<T>(destination: *const T, origin: *const T) -> isize
where
    T: Sized,
{
    let pointee_size = core::mem::size_of::<T>();
    assert!(0 < pointee_size && pointee_size <= isize::max_value() as usize);

    let d = isize::wrapping_sub(destination as _, origin as _);
    d.wrapping_div(pointee_size as _)
}

type size_t = libc::c_ulong;
type uint8_t = libc::c_uchar;
type uint32_t = libc::c_uint;
type uint64_t = libc::c_ulong;
type intptr_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
struct _result_t_avx2 {
    lookup_error: __m256i,
    cont_error: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
struct _state_t_avx2 {
    bytes: __m256i,
    shifted_bytes: __m256i,
    carry_req: uint32_t,
}
#[inline]
unsafe extern "C" fn init_state_avx2(mut state: *mut _state_t_avx2) {
    (*state).carry_req = 0 as libc::c_int as uint32_t;
}
#[inline]
unsafe extern "C" fn load_next_avx2(mut state: *mut _state_t_avx2, mut data: *const libc::c_char) {
    (*state).shifted_bytes =
        _mm256_lddqu_si256(data.offset(-(1 as libc::c_int as isize)) as *mut __m256i);
    (*state).bytes = _mm256_load_si256(data as *mut __m256i);
}
// Validate one vector's worth of input bytes
#[inline]
unsafe extern "C" fn z_validate_vec_avx2(
    mut bytes: __m256i,
    mut shifted_bytes: __m256i,
    mut carry_req: *mut uint32_t,
) -> _result_t_avx2 {
    let mut result: _result_t_avx2 = _result_t_avx2 {
        lookup_error: _mm256_setzero_si256(),
        cont_error: 0,
    };
    // Add error masks as locals
    let error_1: __m256i = _mm256_setr_epi8(
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x3 as libc::c_int as libc::c_char,
        0x1 as libc::c_int as libc::c_char,
        0xd as libc::c_int as libc::c_char,
        0x79 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x80 as libc::c_int as libc::c_char,
        0x3 as libc::c_int as libc::c_char,
        0x1 as libc::c_int as libc::c_char,
        0xd as libc::c_int as libc::c_char,
        0x79 as libc::c_int as libc::c_char,
    );
    let error_2: __m256i = _mm256_setr_epi8(
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xb6 as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xde as libc::c_int as libc::c_char,
        0xfe as libc::c_int as libc::c_char,
        0xfe as libc::c_int as libc::c_char,
        0xfc as libc::c_int as libc::c_char,
        0xe8 as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xb6 as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xbe as libc::c_int as libc::c_char,
        0xde as libc::c_int as libc::c_char,
        0xfe as libc::c_int as libc::c_char,
        0xfe as libc::c_int as libc::c_char,
        0xfc as libc::c_int as libc::c_char,
        0xe8 as libc::c_int as libc::c_char,
    );
    let error_3: __m256i = _mm256_setr_epi8(
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0xd6 as libc::c_int as libc::c_char,
        0xe6 as libc::c_int as libc::c_char,
        0xea as libc::c_int as libc::c_char,
        0xea as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0x53 as libc::c_int as libc::c_char,
        0xd6 as libc::c_int as libc::c_char,
        0xe6 as libc::c_int as libc::c_char,
        0xea as libc::c_int as libc::c_char,
        0xea as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
        0x63 as libc::c_int as libc::c_char,
    );
    // Look up error masks for three consecutive nibbles. Note that we need a
    // special trick for the second nibble (as described in gen_table.py for
    // the MARK_CONT2 bit). There, we invert shifted_bytes and AND with 0x8F
    // with one AND NOT instruction, which zeroes out e_2 for ASCII input.
    let mut e_1: __m256i = _mm256_shuffle_epi8(
        error_1,
        _mm256_and_si256(
            if 4 as libc::c_int != 0 {
                _mm256_srli_epi16(shifted_bytes, 4 as libc::c_int)
            } else {
                shifted_bytes
            },
            _mm256_set1_epi8(0xf as libc::c_int as libc::c_char),
        ),
    );
    let mut e_2: __m256i = _mm256_shuffle_epi8(
        error_2,
        _mm256_andnot_si256(
            shifted_bytes,
            _mm256_set1_epi8(0x8f as libc::c_int as libc::c_char),
        ),
    );
    let mut e_3: __m256i = _mm256_shuffle_epi8(
        error_3,
        _mm256_and_si256(
            if 4 as libc::c_int != 0 {
                _mm256_srli_epi16(bytes, 4 as libc::c_int)
            } else {
                bytes
            },
            _mm256_set1_epi8(0xf as libc::c_int as libc::c_char),
        ),
    );
    // Get error bits common between the first and third nibbles. This is a
    // subexpression used for ANDing all three nibbles, but is also used for
    // finding continuation bytes after the first. The MARK_CONT bit is only
    // set in this mask if both the first and third nibbles correspond to
    // continuation bytes, so the first continuation byte after a leader byte
    // won't be checked.
    let mut e_1_3: __m256i = _mm256_and_si256(e_1, e_3);
    // Create the result vector with any bits set in all three error masks.
    // Note that we use AND NOT here, because the bits in e_2 are inverted--
    // this is needed for ASCII->continuation to trigger the MARK_CONT2 error.
    result.lookup_error = _mm256_andnot_si256(e_2, e_1_3);
    // req is a mask of what bytes are required to be continuation bytes after
    // the first, and cont is a mask of the continuation bytes after the first
    let mut req: uint64_t = *carry_req as uint64_t;
    let mut cont: uint32_t = _mm256_movemask_epi8(if 7 as libc::c_int - 7 as libc::c_int != 0 {
        _mm256_slli_epi16(e_1_3, 7 as libc::c_int - 7 as libc::c_int)
    } else {
        e_1_3
    }) as uint32_t;
    // Compute the continuation byte mask by finding bytes that start with
    // 11x, 111x, and 1111. For each of these prefixes, we get a bitmask
    // and shift it forward by 1, 2, or 3. This loop should be unrolled by
    // the compiler, and the (n == 1) branch inside eliminated.
    let mut leader_3: uint32_t = _mm256_movemask_epi8(if 7 as libc::c_int - 3 as libc::c_int != 0 {
        _mm256_slli_epi16(e_1, 7 as libc::c_int - 3 as libc::c_int)
    } else {
        e_1
    }) as uint32_t;
    // Micro-optimization: use x+x instead of x<<1, it's a tiny bit faster
    let mut leader_4: uint32_t = _mm256_movemask_epi8(
        if 7 as libc::c_int - (6 as libc::c_int + 1 as libc::c_int) != 0 {
            _mm256_slli_epi16(
                _mm256_add_epi32(e_1, e_1),
                7 as libc::c_int - (6 as libc::c_int + 1 as libc::c_int),
            )
        } else {
            _mm256_add_epi32(e_1, e_1)
        },
    ) as uint32_t;
    // We add the shifted mask here instead of ORing it, which would
    // be the more natural operation, so that this line can be done
    // with one lea. While adding could give a different result due
    // to carries, this will only happen for invalid UTF-8 sequences,
    // and in a way that won't cause it to pass validation. Reasoning:
    // Any bits for required continuation bytes come after the bits
    // for their leader bytes, and are all contiguous. For a carry to
    // happen, two of these bit sequences would have to overlap. If
    // this is the case, there is a leader byte before the second set
    // of required continuation bytes (and thus before the bit that
    // will be cleared by a carry). This leader byte will not be
    // in the continuation mask, despite being required. QEDish.
    req = (req as libc::c_ulong).wrapping_add((leader_4 as uint64_t) << 2 as libc::c_int)
        as uint64_t as uint64_t;
    req = (req as libc::c_ulong).wrapping_add((leader_3 as uint64_t) << 1 as libc::c_int)
        as uint64_t as uint64_t;
    // Save continuation bits and input bytes for the next round
    *carry_req = (req >> 32 as libc::c_int) as uint32_t;
    // Check that continuation bytes match. We must cast req from vmask2_t
    // (which holds the carry mask in the upper half) to vmask_t, which
    // zeroes out the upper bits
    result.cont_error = cont ^ req as uint32_t;
    return result;
}
// Validate a chunk of input data which is <UNROLL_COUNT> consecutive vectors.
// This assumes that <data> is aligned to a V_LEN boundary, and that we can read
// one byte before data. We only check for validation failures or ASCII-only
// input once in this function, for entire input chunks.
#[inline]
unsafe extern "C" fn z_validate_unrolled_chunk_avx2(
    mut state: *mut _state_t_avx2,
    mut data: *const libc::c_char,
) -> libc::c_int {
    // Run other validations. Annoyingly, at least one compiler (GCC 8)
    // doesn't optimize v_or(0, x) into x, so manually unroll the first
    // iteration
    load_next_avx2(
        state,
        data.offset((0 as libc::c_int * 32 as libc::c_int) as isize),
    );
    let mut result: _result_t_avx2 = z_validate_vec_avx2(
        (*state).bytes,
        (*state).shifted_bytes,
        &mut (*state).carry_req,
    );
    let mut i: uint32_t = 1 as libc::c_int as uint32_t;
    while i < 6 as libc::c_int as libc::c_uint {
        load_next_avx2(
            state,
            data.offset(i.wrapping_mul(32 as libc::c_int as libc::c_uint) as isize),
        );
        let mut r: _result_t_avx2 = z_validate_vec_avx2(
            (*state).bytes,
            (*state).shifted_bytes,
            &mut (*state).carry_req,
        );
        result.lookup_error = _mm256_or_si256(result.lookup_error, r.lookup_error);
        result.cont_error = result.cont_error | r.cont_error;
        i = i.wrapping_add(1)
    }
    return ((result.cont_error != 0 as libc::c_int as libc::c_uint) as libc::c_int as libc::c_long
        != 0
        || (_mm256_testz_si256(result.lookup_error, result.lookup_error) == 0) as libc::c_int
            as libc::c_long
            != 0) as libc::c_int;
}
// Validate a piece of data too small to fit in a full unrolled chunk. This is
// done at the beginning and end of the input, and is slower because we copy
// input data into a buffer--we need to copy because memory outside the input
// might be unmapped and cause a segfault. This function takes two nontrivial
// parameters: <first> is the character before the input data pointer (which is
// handled specially outside of this code), and <align_at_start>, which decides
// which side of the buffer to align the input data on (the end of the buffer
// for the first part of the data, or the start of the buffer for the last part
// of the data). This is so the transient state such as the carry_req mask works
// consistently across the entire input--it can be thought of like padding the
// input data with NUL bytes on either side.
#[inline]
unsafe extern "C" fn z_validate_small_chunk_avx2(
    mut state: *mut _state_t_avx2,
    mut data: *const libc::c_char,
    mut len: size_t,
    mut first: libc::c_char,
    mut align_at_start: libc::c_int,
) -> libc::c_int {
    // Deal with any bytes remaining. Rather than making a separate scalar path,
    // just fill in a buffer, reading bytes only up to len, and load from that.
    // TODO: In the C code, this is aligned to 32 bytes.
    let mut buffer: [libc::c_char; 33] = [
        0 as libc::c_int as libc::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut offset: size_t = if align_at_start != 0 {
        0 as libc::c_int as libc::c_ulong
    } else {
        (32 as libc::c_int as libc::c_ulong).wrapping_sub(len)
    };
    buffer[offset as usize] = first;
    assert!(len <= 32);

    let mut i: size_t = 0 as libc::c_int as size_t;
    while i < len {
        buffer[offset
            .wrapping_add(i)
            .wrapping_add(1 as libc::c_int as libc::c_ulong) as usize] = *data.offset(i as isize);
        i = i.wrapping_add(1)
    }
    let mut bytes: __m256i =
        _mm256_lddqu_si256(buffer.as_mut_ptr().offset(1 as libc::c_int as isize) as *mut __m256i);
    let mut shifted_bytes: __m256i = _mm256_load_si256(buffer.as_mut_ptr() as *mut __m256i);
    let mut result: _result_t_avx2 =
        z_validate_vec_avx2(bytes, shifted_bytes, &mut (*state).carry_req);
    return ((result.cont_error != 0 as libc::c_int as libc::c_uint) as libc::c_int as libc::c_long
        != 0
        || (_mm256_testz_si256(result.lookup_error, result.lookup_error) == 0) as libc::c_int
            as libc::c_long
            != 0) as libc::c_int;
}
#[no_mangle]
unsafe extern "C" fn z_validate_utf8_avx2(
    mut data: *const libc::c_char,
    mut len: size_t,
) -> libc::c_int {
    let mut state: [_state_t_avx2; 1] = [_state_t_avx2 {
        bytes: _mm256_setzero_si256(),
        shifted_bytes: _mm256_setzero_si256(),
        carry_req: 0,
    }; 1];
    init_state_avx2(state.as_mut_ptr());
    // Get an aligned pointer to our input data. We round up to the next
    // multiple of V_LEN after data + 1, since we need to read one byte before
    // the data pointer for shifted_bytes. This rounding is equivalent to
    // rounding down after adding V_LEN, which is what this does, by clearing
    // the low bits of the pointer (V_LEN is always a power of two).
    let mut aligned_data: *const libc::c_char =
        (data as intptr_t + 32 as libc::c_int as libc::c_long
            & -(32 as libc::c_int) as libc::c_long) as *const libc::c_char;
    if aligned_data >= data.offset(len as isize) {
        // The input wasn't big enough to fill one vector
        if z_validate_small_chunk_avx2(
            state.as_mut_ptr(),
            data,
            len,
            '\u{0}' as i32 as libc::c_char,
            0 as libc::c_int,
        ) != 0
        {
            return 0 as libc::c_int;
        }
    } else {
        // Validate the start section between data and aligned_data
        if z_validate_small_chunk_avx2(
            state.as_mut_ptr(),
            data,
            wrapping_offset_from(aligned_data, data) as libc::c_long as size_t,
            '\u{0}' as i32 as libc::c_char,
            0 as libc::c_int,
        ) != 0
        {
            return 0 as libc::c_int;
        }
        // Get the size of the aligned inner section of data
        let mut aligned_len: size_t =
            len.wrapping_sub(
                wrapping_offset_from(aligned_data, data) as libc::c_long as libc::c_ulong
            );
        // Subtract from the aligned_len any bytes at the end of the input that
        // don't fill an entire UNROLL_SIZE-byte chunk
        aligned_len = aligned_len.wrapping_sub(
            aligned_len.wrapping_rem((6 as libc::c_int * 32 as libc::c_int) as libc::c_ulong),
        );
        // Validate the main inner part of the input, in UNROLL_SIZE-byte chunks
        let mut offset: size_t = 0 as libc::c_int as size_t;
        while offset < aligned_len {
            if z_validate_unrolled_chunk_avx2(
                state.as_mut_ptr(),
                aligned_data.offset(offset as isize),
            ) != 0
            {
                return 0 as libc::c_int;
            }
            offset = (offset as libc::c_ulong)
                .wrapping_add((6 as libc::c_int * 32 as libc::c_int) as libc::c_ulong)
                as size_t as size_t
        }
        // Validate the end section. This might be multiple vector's worth of
        // data, due to the main loop being unrolled
        let mut end_data: *const libc::c_char = aligned_data.offset(aligned_len as isize);
        let mut end_len: size_t =
            wrapping_offset_from(data.offset(len as isize), end_data) as libc::c_long as size_t;
        let mut offset_0: size_t = 0 as libc::c_int as size_t;
        while offset_0 < end_len {
            let mut l: size_t = end_len.wrapping_sub(offset_0);
            l = if l > 32 as libc::c_int as libc::c_ulong {
                32 as libc::c_int as libc::c_ulong
            } else {
                l
            };
            if z_validate_small_chunk_avx2(
                state.as_mut_ptr(),
                end_data.offset(offset_0 as isize),
                l,
                *end_data.offset(offset_0.wrapping_sub(1 as libc::c_int as libc::c_ulong) as isize),
                1 as libc::c_int,
            ) != 0
            {
                return 0 as libc::c_int;
            }
            offset_0 = (offset_0 as libc::c_ulong).wrapping_add(32 as libc::c_int as libc::c_ulong)
                as size_t as size_t
        }
    }
    // Micro-optimization compensation! We have to double check for a multi-byte
    // sequence that starts on the last byte, since we check for the first
    // continuation byte using error masks, which are shifted one byte forward
    // in the data stream. Thus, a leader byte in the last position will be
    // ignored if it's also the last byte of a vector.
    if len > 0 as libc::c_int as libc::c_ulong
        && *data.offset(len.wrapping_sub(1 as libc::c_int as libc::c_ulong) as isize) as uint8_t
            as libc::c_int
            >= 0xc0 as libc::c_int
    {
        return 0 as libc::c_int;
    }
    // The input is valid if we don't have any more expected continuation bytes
    return !((*state.as_mut_ptr()).carry_req != 0 as libc::c_int as libc::c_uint) as libc::c_int;
}
// Undefine all macros

pub fn is_utf8(bytes: &[u8]) -> bool {
    let mut data = bytes.as_ptr();
    let mut len = bytes.len();
    unsafe { z_validate_utf8_avx2(data as *const i8, len as size_t) != 0 }
}
