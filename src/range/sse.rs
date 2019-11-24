/*
 * Process 2x16 bytes in each iteration.
 * Comments removed for brevity. See range-sse.c for details.
 */
#[cfg(target_arch = "x86")]
pub use core::arch::x86::{
    __m128i, _mm_add_epi8, _mm_adds_epu8, _mm_alignr_epi8, _mm_and_si128, _mm_cmpgt_epi8,
    _mm_cmplt_epi8, _mm_extract_epi32, _mm_lddqu_si128, _mm_or_si128, _mm_set1_epi8, _mm_set_epi8,
    _mm_setzero_si128, _mm_shuffle_epi8, _mm_srli_epi16, _mm_sub_epi8, _mm_subs_epu8,
    _mm_testz_si128,
};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{
    __m128i, _mm_add_epi8, _mm_adds_epu8, _mm_alignr_epi8, _mm_and_si128, _mm_cmpgt_epi8,
    _mm_cmplt_epi8, _mm_extract_epi32, _mm_lddqu_si128, _mm_or_si128, _mm_set1_epi8, _mm_set_epi8,
    _mm_setzero_si128, _mm_shuffle_epi8, _mm_srli_epi16, _mm_sub_epi8, _mm_subs_epu8,
    _mm_testz_si128,
};

use crate::libcore;

static FIRST_LEN_TABLE: [i8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3];
static FIRST_RANGE_TABLE: [i8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8];
static RANGE_MIN_TABLE: [i8; 16] = [
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
static RANGE_MAX_TABLE: [i8; 16] = [
    0x7Fi32 as i8,
    0xbFi32 as i8,
    0xbFi32 as i8,
    0xbFi32 as i8,
    0xbFi32 as i8,
    0x9Fi32 as i8,
    0xbFi32 as i8,
    0x8Fi32 as i8,
    0xF4i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
    0x80i32 as i8,
];
static DF_EE_TABLE: [i8; 16] = [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0];
static EF_FE_TABLE: [i8; 16] = [0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub fn is_utf8(bytes: &[u8]) -> bool {
    let mut data = bytes.as_ptr();
    let mut len = bytes.len();
    unsafe {
        if len >= 32 {
            let mut prev_input = _mm_setzero_si128();
            let mut prev_first_len = _mm_setzero_si128();
            let mut error = _mm_setzero_si128();

            // This should be safe as long as the tables are all 16 bytes long
            let first_len_table = _mm_lddqu_si128(FIRST_LEN_TABLE.as_ptr() as *const __m128i);
            let first_range_table = _mm_lddqu_si128(FIRST_RANGE_TABLE.as_ptr() as *const __m128i);
            let range_min_table = _mm_lddqu_si128(RANGE_MIN_TABLE.as_ptr() as *const __m128i);
            let range_max_table = _mm_lddqu_si128(RANGE_MAX_TABLE.as_ptr() as *const __m128i);
            let df_ee_table = _mm_lddqu_si128(DF_EE_TABLE.as_ptr() as *const __m128i);
            let ef_fe_table = _mm_lddqu_si128(EF_FE_TABLE.as_ptr() as *const __m128i);

            while len >= 32 {
                /* **************************** block 1 ****************************/
                let input = _mm_lddqu_si128(data as *const __m128i);
                let mut high_nibbles = _mm_and_si128(_mm_srli_epi16(input, 4), _mm_set1_epi8(0xF));
                let first_len = _mm_shuffle_epi8(first_len_table, high_nibbles);
                let mut range = _mm_shuffle_epi8(first_range_table, high_nibbles);
                range = _mm_or_si128(range, _mm_alignr_epi8(first_len, prev_first_len, 15));
                let mut tmp1 = _mm_subs_epu8(first_len, _mm_set1_epi8(1));
                let mut tmp2 = _mm_subs_epu8(prev_first_len, _mm_set1_epi8(1));
                range = _mm_or_si128(range, _mm_alignr_epi8(tmp1, tmp2, 14));
                tmp1 = _mm_subs_epu8(first_len, _mm_set1_epi8(2));
                tmp2 = _mm_subs_epu8(prev_first_len, _mm_set1_epi8(2));
                range = _mm_or_si128(range, _mm_alignr_epi8(tmp1, tmp2, 13));
                let mut shift1 = _mm_alignr_epi8(input, prev_input, 15);
                let mut pos = _mm_sub_epi8(shift1, _mm_set1_epi8(0xEFi32 as i8));
                tmp1 = _mm_subs_epu8(pos, _mm_set1_epi8(240i32 as i8));
                let mut range2 = _mm_shuffle_epi8(df_ee_table, tmp1);
                tmp2 = _mm_adds_epu8(pos, _mm_set1_epi8(112));
                range2 = _mm_add_epi8(range2, _mm_shuffle_epi8(ef_fe_table, tmp2));
                range = _mm_add_epi8(range, range2);
                let mut minv: __m128i = _mm_shuffle_epi8(range_min_table, range);
                let mut maxv: __m128i = _mm_shuffle_epi8(range_max_table, range);
                error = _mm_or_si128(error, _mm_cmplt_epi8(input, minv));
                error = _mm_or_si128(error, _mm_cmpgt_epi8(input, maxv));
                /* **************************** block 2 ****************************/
                let _input = _mm_lddqu_si128(data.offset(16 as isize) as *const __m128i);
                high_nibbles = _mm_and_si128(_mm_srli_epi16(_input, 4), _mm_set1_epi8(0xF));
                let mut _first_len = _mm_shuffle_epi8(first_len_table, high_nibbles);
                let mut _range = _mm_shuffle_epi8(first_range_table, high_nibbles);
                _range = _mm_or_si128(_range, _mm_alignr_epi8(_first_len, first_len, 15));
                tmp1 = _mm_subs_epu8(_first_len, _mm_set1_epi8(1));
                tmp2 = _mm_subs_epu8(first_len, _mm_set1_epi8(1));
                _range = _mm_or_si128(_range, _mm_alignr_epi8(tmp1, tmp2, 14));
                tmp1 = _mm_subs_epu8(_first_len, _mm_set1_epi8(2));
                tmp2 = _mm_subs_epu8(first_len, _mm_set1_epi8(2));
                _range = _mm_or_si128(_range, _mm_alignr_epi8(tmp1, tmp2, 13));
                let mut _range2 = _mm_setzero_si128();
                shift1 = _mm_alignr_epi8(_input, input, 15);
                pos = _mm_sub_epi8(shift1, _mm_set1_epi8(0xEFi32 as i8));
                tmp1 = _mm_subs_epu8(pos, _mm_set1_epi8(240i32 as i8));
                _range2 = _mm_shuffle_epi8(df_ee_table, tmp1);
                tmp2 = _mm_adds_epu8(pos, _mm_set1_epi8(112));
                _range2 = _mm_add_epi8(_range2, _mm_shuffle_epi8(ef_fe_table, tmp2));
                _range = _mm_add_epi8(_range, _range2);
                minv = _mm_shuffle_epi8(range_min_table, _range);
                maxv = _mm_shuffle_epi8(range_max_table, _range);
                error = _mm_or_si128(error, _mm_cmplt_epi8(_input, minv));
                error = _mm_or_si128(error, _mm_cmpgt_epi8(_input, maxv));
                /* *********************** next iteration **************************/
                prev_input = _input;
                prev_first_len = _first_len;
                data = data.offset(32 as isize);
                len -= 32
            }
            if _mm_testz_si128(error, error) == 0 {
                return false;
            }
            let mut token4: i32 = _mm_extract_epi32(prev_input, 3);
            let token: *const i8 = &mut token4 as *mut i32 as *const i8;
            let mut lookahead = 0;
            if *token.offset(3 as isize) as i32 > 0xBF as i32 as i8 as i32 {
                lookahead = 1
            } else if *token.offset(2 as isize) as i32 > 0xBF as i32 as i8 as i32 {
                lookahead = 2
            } else if *token.offset(1 as isize) as i32 > 0xBF as i32 as i8 as i32 {
                lookahead = 3
            }
            data = data.offset(-(lookahead as isize));
            len += lookahead
        }
        return libcore::is_utf8(core::slice::from_raw_parts(data, len));
    }
}
