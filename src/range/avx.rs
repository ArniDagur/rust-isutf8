use crate::libcore;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// Map high nibble of "First Byte" to legal character length minus 1
// 0x00 ~ 0xBF --> 0
// 0xC0 ~ 0xDF --> 1
// 0xE0 ~ 0xEF --> 2
// 0xF0 ~ 0xFF --> 3
static FIRST_LEN_TABLE: [i8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3,
];
// Map "First Byte" to 8-th item of range table (0xC2 ~ 0xF4)
static FIRST_RANGE_TABLE: [i8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8,
];
// Range table, map range index to min and max values
// Index 0    : 00 ~ 7F (First Byte, ascii)
// Index 1,2,3: 80 ~ BF (Second, Third, Fourth Byte)
// Index 4    : A0 ~ BF (Second Byte after E0)
// Index 5    : 80 ~ 9F (Second Byte after ED)
// Index 6    : 90 ~ BF (Second Byte after F0)
// Index 7    : 80 ~ 8F (Second Byte after F4)
// Index 8    : C2 ~ F4 (First Byte, non ascii)
// Index 9~15 : illegal: i >= 127 && i <= -128
static RANGE_MIN_TABLE: [i8; 32] = [
    0,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0xa0i32 as i8,
    0x80i32 as i8,
    0x90i32 as i8,
    0x80i32 as i8,
    0xC2i32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0xa0i32 as i8,
    0x80i32 as i8,
    0x90i32 as i8,
    0x80i32 as i8,
    0xC2i32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
    0x7Fi32 as i8,
];
static RANGE_MAX_TABLE: [i8; 32] = [
    0x7Fi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0x9Fi32 as i8,
    0xBFi32 as i8,
    0x8Fi32 as i8,
    0xF4i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x7Fi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0xBFi32 as i8,
    0x9Fi32 as i8,
    0xBFi32 as i8,
    0x8fi32 as i8,
    0xF4i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
];
/*
 * Tables for fast handling of four special First Bytes(E0,ED,F0,F4), after
 * which the Second Byte are not 80~BF. It contains "range index adjustment".
 * +------------+---------------+------------------+----------------+
 * | First Byte | original range| range adjustment | adjusted range |
 * +------------+---------------+------------------+----------------+
 * | E0         | 2             | 2                | 4              |
 * +------------+---------------+------------------+----------------+
 * | ED         | 2             | 3                | 5              |
 * +------------+---------------+------------------+----------------+
 * | F0         | 3             | 3                | 6              |
 * +------------+---------------+------------------+----------------+
 * | F4         | 4             | 4                | 8              |
 * +------------+---------------+------------------+----------------+
 */
// index1 -> E0, index14 -> ED
static DF_EE_TABLE: [i8; 32] = [
    0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0,
];
// index1 -> F0, index5 -> F4
static EF_FE_TABLE: [i8; 32] = [
    0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/* Define 1 to return index of first error char */
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
fn push_last_3bytes_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe {
        return _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 13);
    }
}

/* 5x faster than naive method */
/* Return 0 - success, -1 - error, >0 - first error char(if RET_ERR_IDX = 1) */
pub fn is_utf8(bytes: &[u8]) -> bool {
    let mut data = bytes.as_ptr();
    let mut len = bytes.len();
    unsafe {
        if len >= 32 {
            let mut prev_input = _mm256_setzero_si256();
            let mut prev_first_len = _mm256_setzero_si256();
            let mut error = _mm256_setzero_si256();

            // This should be safe as long as the tables are all 32 bytes long
            let first_len_tbl = _mm256_lddqu_si256(FIRST_LEN_TABLE.as_ptr() as *const __m256i);
            let first_range_tbl = _mm256_lddqu_si256(FIRST_RANGE_TABLE.as_ptr() as *const __m256i);
            let range_min_tbl = _mm256_lddqu_si256(RANGE_MIN_TABLE.as_ptr() as *const __m256i);
            let range_max_tbl = _mm256_lddqu_si256(RANGE_MAX_TABLE.as_ptr() as *const __m256i);
            let df_ee_tbl = _mm256_lddqu_si256(DF_EE_TABLE.as_ptr() as *const __m256i);
            let ef_fe_tbl = _mm256_lddqu_si256(EF_FE_TABLE.as_ptr() as *const __m256i);

            while len >= 32 {
                let input = _mm256_lddqu_si256(data as *const __m256i);
                /* high_nibbles = input >> 4 */
                let high_nibbles =
                    _mm256_and_si256(_mm256_srli_epi16(input, 4), _mm256_set1_epi8(0xF));
                /* first_len = legal character length minus 1 */
                /* 0 for 00~7F, 1 for C0~DF, 2 for E0~EF, 3 for F0~FF */
                /* first_len = first_len_tbl[high_nibbles] */
                let first_len = _mm256_shuffle_epi8(first_len_tbl, high_nibbles);
                // First Byte: set range index to 8 for bytes within 0xC0 ~ 0xFF
                /* range = first_range_tbl[high_nibbles] */
                let mut range = _mm256_shuffle_epi8(first_range_tbl, high_nibbles);
                // Second Byte: set range index to first_len
                // 0 for 00~7F, 1 for C0~DF, 2 for E0~EF, 3 for F0~FF
                /* range |= (first_len, prev_first_len) << 1 byte */
                range = _mm256_or_si256(range, push_last_byte_of_a_to_b(prev_first_len, first_len));
                // Third Byte: set range index to saturate_sub(first_len, 1)
                // 0 for 00~7F, 0 for C0~DF, 1 for E0~EF, 2 for F0~FF
                /* tmp1 = saturate_sub(first_len, 1) */
                let mut tmp1 = _mm256_subs_epu8(first_len, _mm256_set1_epi8(1));
                /* tmp2 = saturate_sub(prev_first_len, 1) */
                let mut tmp2 = _mm256_subs_epu8(prev_first_len, _mm256_set1_epi8(1));
                /* range |= (tmp1, tmp2) << 2 bytes */
                range = _mm256_or_si256(range, push_last_2bytes_of_a_to_b(tmp2, tmp1));
                // Fourth Byte: set range index to saturate_sub(first_len, 2)
                // 0 for 00~7F, 0 for C0~DF, 0 for E0~EF, 1 for F0~FF
                /* tmp1 = saturate_sub(first_len, 2) */
                tmp1 = _mm256_subs_epu8(first_len, _mm256_set1_epi8(2));
                /* tmp2 = saturate_sub(prev_first_len, 2) */
                tmp2 = _mm256_subs_epu8(prev_first_len, _mm256_set1_epi8(2));
                /* range |= (tmp1, tmp2) << 3 bytes */
                range = _mm256_or_si256(range, push_last_3bytes_of_a_to_b(tmp2, tmp1));
                // Now we have below range indices caluclated
                // Correct cases:
                // - 8 for C0~FF
                // - 3 for 1st byte after F0~FF
                // - 2 for 1st byte after E0~EF or 2nd byte after F0~FF
                // - 1 for 1st byte after C0~DF or 2nd byte after E0~EF or
                //         3rd byte after F0~FF
                // - 0 for others
                // Error cases:
                //   9,10,11 if non ascii First Byte overlaps
                //   E.g., F1 80 C2 90 --> 8 3 10 2, where 10 indicates error
                //
                // Adjust Second Byte range for special First Bytes(E0,ED,F0,F4)
                // Overlaps lead to index 9~15, which are illegal in range table
                // shift1 = (input, prev_input) << 1 byte
                let shift1 = push_last_byte_of_a_to_b(prev_input, input);
                let pos = _mm256_sub_epi8(shift1, _mm256_set1_epi8(0xEFi32 as i8));
                // ---------+---------------+---------------------+---------------+
                // shift1:  | EF  F0 ... FE | FF  00  ... ...  DE | DF  E0 ... EE |
                // pos:     | 0   1      15 | 16  17           239| 240 241    255|
                // pos-240: | 0   0      0  | 0   0            0  | 0   1      15 |
                // pos+112: | 112 113    127|       >= 128        |     >= 128    |
                // ---------+---------------+---------------------+---------------+
                tmp1 = _mm256_subs_epu8(pos, _mm256_set1_epi8(240i32 as i8));
                let mut range2 = _mm256_shuffle_epi8(df_ee_tbl, tmp1);
                tmp2 = _mm256_adds_epu8(pos, _mm256_set1_epi8(112i32 as i8));
                range2 = _mm256_add_epi8(range2, _mm256_shuffle_epi8(ef_fe_tbl, tmp2));
                range = _mm256_add_epi8(range, range2);
                // Load min and max values per calculated range index
                let minv = _mm256_shuffle_epi8(range_min_tbl, range);
                let maxv = _mm256_shuffle_epi8(range_max_tbl, range);
                // Check value range
                error = _mm256_or_si256(error, _mm256_cmpgt_epi8(minv, input));
                error = _mm256_or_si256(error, _mm256_cmpgt_epi8(input, maxv));
                prev_input = input;
                prev_first_len = first_len;
                data = data.offset(32);
                len -= 32
            }
            if _mm256_testz_si256(error, error) == 0 {
                return false;
            }
            // Find previous token (not 80~BF)
            let mut token4: i32 = _mm256_extract_epi32(prev_input, 7);
            let token: *const i8 = &mut token4 as *mut i32 as *const i8;
            let mut lookahead = 0;
            if *token.offset(3) > 0xBFi32 as i8 {
                lookahead = 1
            } else if *token.offset(2) > 0xBFi32 as i8 {
                lookahead = 2
            } else if *token.offset(1) > 0xBFi32 as i8 {
                lookahead = 3
            }
            data = data.offset(-(lookahead as isize));
            len += lookahead
        }
        // Check remaining bytes with naive method
        return libcore::is_utf8(core::slice::from_raw_parts(data, len));
    }
}
