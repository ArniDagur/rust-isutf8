// vim: fdm=marker:et:ts=4:sts=4:sw=4
//
// lib.rs
// Copyright (c) 2019 Árni Dagur <arni@dagur.eu> MIT license
//
#![no_std]
#![feature(stdsimd)]
#![feature(doc_cfg)]
pub mod lemire;

#[cfg(test)]
mod tests {
    macro_rules! create_tests {
        ($test_function:ident) => {
            assert!($test_function(&[0xc0, 0x80]).is_err());
            assert!($test_function(&[0xc0, 0xae]).is_err());
            assert!($test_function(&[0xe0, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xe0, 0x80, 0xaf]).is_err());
            assert!($test_function(&[0xe0, 0x81, 0x81]).is_err());
            assert!($test_function(&[0xf0, 0x82, 0x82, 0xac]).is_err());
            assert!($test_function(&[0xf4, 0x90, 0x80, 0x80]).is_err());

            // deny surrogates
            assert!($test_function(&[0xED, 0xA0, 0x80]).is_err());
            assert!($test_function(&[0xED, 0xBF, 0xBF]).is_err());

            assert!($test_function(&[0xC2, 0x80]).is_ok());
            assert!($test_function(&[0xDF, 0xBF]).is_ok());
            assert!($test_function(&[0xE0, 0xA0, 0x80]).is_ok());
            assert!($test_function(&[0xED, 0x9F, 0xBF]).is_ok());
            assert!($test_function(&[0xEE, 0x80, 0x80]).is_ok());
            assert!($test_function(&[0xEF, 0xBF, 0xBF]).is_ok());
            assert!($test_function(&[0xF0, 0x90, 0x80, 0x80]).is_ok());
            assert!($test_function(&[0xF4, 0x8F, 0xBF, 0xBF]).is_ok());

            // from: http://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt
            assert!($test_function("κόσμε".as_bytes()).is_ok());

            // 2.1 First possible sequence of a certain length: 1 to 6 bytes
            assert!($test_function(&[0]).is_ok());
            assert!($test_function(&[0xC2, 0x80]).is_ok());
            assert!($test_function(&[0xE0, 0xA0, 0x80]).is_ok());
            assert!($test_function(&[0xF0, 0x90, 0x80, 0x80]).is_ok());
            assert!($test_function(&[0xF8, 0x88, 0x80, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xFC, 0x84, 0x80, 0x80, 0x80, 0x80]).is_err());

            // 2.2 Last possible sequence of a certain length: 1 to 6 bytes
            assert!($test_function(&[0x7F]).is_ok());
            assert!($test_function(&[0xDF, 0xBF]).is_ok());
            assert!($test_function(&[0xEF, 0xBF, 0xBF]).is_ok());
            assert!($test_function(&[0xF7, 0xBF, 0xBF, 0xBF]).is_err());
            assert!($test_function(&[0xFB, 0xBF, 0xBF, 0xBF, 0xBF]).is_err());
            assert!($test_function(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF]).is_err());

            // 2.3 Other boundary conditions
            assert!($test_function(&[0xED, 0x9F, 0xBF]).is_ok());
            assert!($test_function(&[0xEE, 0x80, 0x80]).is_ok());
            assert!($test_function(&[0xEF, 0xBF, 0xBD]).is_ok());
            assert!($test_function(&[0xF4, 0x8F, 0xBF, 0xBF]).is_ok());
            assert!($test_function(&[0xF4, 0x90, 0x80, 0x80]).is_err());

            // 3.1  Unexpected continuation bytes
            assert!($test_function(&[0x80]).is_err());
            assert!($test_function(&[0xbf]).is_err());
            assert!($test_function(&[0x80, 0xBF]).is_err());
            assert!($test_function(&[0x80, 0xBF, 0x80]).is_err());
            assert!($test_function(&[0x80, 0xBF, 0x80, 0xBF]).is_err());
            assert!($test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80]).is_err());
            assert!($test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80, 0xBF]).is_err());
            assert!($test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80, 0xBF, 0x80]).is_err());

            // 3.1.9 Sequence of all 64 possible continuation bytes (0x80-0xbf):
            #[cfg_attr(rustfmt, rustfmt_skip)]
            let continuation_bytes = [
                0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
                0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
                0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
                0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
                0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
                0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
                0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
                0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
            ];
            assert!($test_function(&continuation_bytes).is_err());
            for &b in continuation_bytes.iter() {
                assert!($test_function(&[b]).is_err());
            }

            // 3.2  Lonely start characters
            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_2 = [
                0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7,
                0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
                0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7,
                0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
            ];
            assert!($test_function(&lonely_start_characters_2).is_err());
            for &b in &lonely_start_characters_2 {
                assert!($test_function(&[b]).is_err());
            }

            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_3 = [
                0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7,
                0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
            ];
            assert!($test_function(&lonely_start_characters_3).is_err());
            for &b in &lonely_start_characters_3 {
                assert!($test_function(&[b]).is_err());
            }

            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_4 = [
                0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
            ];
            assert!($test_function(&lonely_start_characters_4).is_err());
            for &b in &lonely_start_characters_4 {
                assert!($test_function(&[b]).is_err());
            }

            let lonely_start_characters_5 = [0xF8, 0xF9, 0xFA, 0xFB];
            assert!($test_function(&lonely_start_characters_5).is_err());
            for &b in &lonely_start_characters_5 {
                assert!($test_function(&[b]).is_err());
            }

            let lonely_start_characters_6 = [0xFC, 0xFD];
            assert!($test_function(&lonely_start_characters_6).is_err());
            for &b in &lonely_start_characters_6 {
                assert!($test_function(&[b]).is_err());
            }

            // 3.3 Sequences with last continuation byte missing
            assert!($test_function(&[0xC0]).is_err());
            assert!($test_function(&[0xE0, 0x80]).is_err());
            assert!($test_function(&[0xF0, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xF8, 0x80, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xDF]).is_err());
            assert!($test_function(&[0xEF, 0xBF]).is_err());
            assert!($test_function(&[0xF7, 0xBF, 0xBF]).is_err());
            assert!($test_function(&[0xFB, 0xBF, 0xBF, 0xBF]).is_err());
            assert!($test_function(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF]).is_err());

            // 3.4 Concatenation of incomplete sequences
            #[cfg_attr(rustfmt, rustfmt_skip)]
            let incomplete = [
                0xC0,
                0xE0, 0x80,
                0xF0, 0x80, 0x80,
                0xF8, 0x80, 0x80, 0x80,
                0xFC, 0x80, 0x80, 0x80, 0x80,
                0xDF,
                0xEF, 0xBF,
                0xF7, 0xBF, 0xBF,
                0xFB, 0xBF, 0xBF, 0xBF,
                0xFD, 0xBF, 0xBF, 0xBF, 0xBF];
            assert!($test_function(&incomplete).is_err());

            // 3.5 Impossible bytes
            assert!($test_function(&[0xFE]).is_err());
            assert!($test_function(&[0xFF]).is_err());
            assert!($test_function(&[0xFE, 0xFE, 0xFF, 0xFF]).is_err());

            // 4. Overlong sequences
            assert!($test_function(&[0xC0, 0xAF]).is_err());
            assert!($test_function(&[0xE0, 0x80, 0xAF]).is_err());
            assert!($test_function(&[0xF0, 0x80, 0x80, 0xAF]).is_err());
            assert!($test_function(&[0xF8, 0x80, 0x80, 0x80, 0xAF]).is_err());
            assert!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0xAF]).is_err());

            assert!($test_function(&[0xC0, 0x80]).is_err());
            assert!($test_function(&[0xE0, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xF0, 0x80, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xF8, 0x80, 0x80, 0x80, 0x80]).is_err());
            assert!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80]).is_err());

            // 5. Illegal code positions
            assert!($test_function(&[0xed, 0xa0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xad, 0xbf]).is_err());
            assert!($test_function(&[0xed, 0xae, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xaf, 0xbf]).is_err());
            assert!($test_function(&[0xed, 0xb0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xbe, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xbf, 0xbf]).is_err());

            assert!($test_function(&[0xed, 0xa0, 0x80, 0xed, 0xb0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xa0, 0x80, 0xed, 0xbf, 0xbf]).is_err());
            assert!($test_function(&[0xed, 0xad, 0xbf, 0xed, 0xb0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xad, 0xbf, 0xed, 0xbf, 0xbf]).is_err());
            assert!($test_function(&[0xed, 0xae, 0x80, 0xed, 0xb0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xae, 0x80, 0xed, 0xbf, 0xbf]).is_err());
            assert!($test_function(&[0xed, 0xaf, 0xbf, 0xed, 0xb0, 0x80]).is_err());
            assert!($test_function(&[0xed, 0xaf, 0xbf, 0xed, 0xbf, 0xbf]).is_err());
        }
    }

    #[test]
    fn test_lemire_avx() {
        use super::lemire::avx::validate_utf8_fast;
        create_tests!(validate_utf8_fast);
    }

    #[test]
    fn test_lemire_sse() {
        use super::lemire::sse::validate_utf8_fast;
        create_tests!(validate_utf8_fast);
    }
}
