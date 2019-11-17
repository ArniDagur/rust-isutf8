// vim: fdm=marker:et:ts=4:sts=4:sw=4
//
// lib.rs
// Copyright (c) 2019 Árni Dagur <arni@dagur.eu> MIT license
//
#![no_std]
#![feature(doc_cfg)]

pub mod lemire;
pub mod libcore;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    static UTF8_SAMPLE_OK: &'static str = include_str!("../props/utf8_sample_ok.txt");
    static ASCII_SAMPLE_OK: &'static str = include_str!("../props/ascii_sample_ok.txt");
    static MOSTLY_ASCII_SAMPLE_OK: &'static str =
        include_str!("../props/mostly_ascii_sample_ok.txt");
    static ALL_UTF8_CHARACTERS: &'static str =
        include_str!("../props/utf8-characters-0-0x10ffff.txt");
    static ALL_UTF8_CHARACTERS_WITH_GARBAGE: &'static [u8; 4644508] =
        include_bytes!("../props/utf8-characters-0-0x10ffff-with-garbage.bin");
    static RANDOM_BYTES: &'static [u8; 524288] = include_bytes!("../props/random_bytes.bin");

    #[cfg_attr(rustfmt, rustfmt_skip)]
    macro_rules! create_tests {
        // Adapted test suite from gnzlbg:
        // https://github.com/gnzlbg/is_utf8/blob/f34c49e5b041bbc49f17a4110799980411e9ccb3/src/lib.rs
        ($test_function:ident) => {
            // deny overlong encodings
            assert_eq!($test_function(&[0xc0, 0x80]), false);
            assert_eq!($test_function(&[0xc0, 0xae]), false);
            assert_eq!($test_function(&[0xe0, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xe0, 0x80, 0xaf]), false);
            assert_eq!($test_function(&[0xe0, 0x81, 0x81]), false);
            assert_eq!($test_function(&[0xf0, 0x82, 0x82, 0xac]), false);
            assert_eq!($test_function(&[0xf4, 0x90, 0x80, 0x80]), false);

            // deny surrogates
            assert_eq!($test_function(&[0xED, 0xA0, 0x80]), false);
            assert_eq!($test_function(&[0xED, 0xBF, 0xBF]), false);

            assert_eq!($test_function(&[0xC2, 0x80]), true);
            assert_eq!($test_function(&[0xDF, 0xBF]), true);
            assert_eq!($test_function(&[0xE0, 0xA0, 0x80]), true);
            assert_eq!($test_function(&[0xED, 0x9F, 0xBF]), true);
            assert_eq!($test_function(&[0xEE, 0x80, 0x80]), true);
            assert_eq!($test_function(&[0xEF, 0xBF, 0xBF]), true);
            assert_eq!($test_function(&[0xF0, 0x90, 0x80, 0x80]), true);
            assert_eq!($test_function(&[0xF4, 0x8F, 0xBF, 0xBF]), true);

            // from: http://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt
            assert_eq!($test_function("κόσμε".as_bytes()), true);

            // 2.1 First possible sequence of a certain length: 1 to 6 bytes
            assert_eq!($test_function(&[0]), true);
            assert_eq!($test_function(&[0xC2, 0x80]), true);
            assert_eq!($test_function(&[0xE0, 0xA0, 0x80]), true);
            assert_eq!($test_function(&[0xF0, 0x90, 0x80, 0x80]), true);
            assert_eq!($test_function(&[0xF8, 0x88, 0x80, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xFC, 0x84, 0x80, 0x80, 0x80, 0x80]), false);

            // 2.2 Last possible sequence of a certain length: 1 to 6 bytes
            assert_eq!($test_function(&[0x7F]), true);
            assert_eq!($test_function(&[0xDF, 0xBF]), true);
            assert_eq!($test_function(&[0xEF, 0xBF, 0xBF]), true);
            assert_eq!($test_function(&[0xF7, 0xBF, 0xBF, 0xBF]), false);
            assert_eq!($test_function(&[0xFB, 0xBF, 0xBF, 0xBF, 0xBF]), false);
            assert_eq!($test_function(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF]), false);

            // 2.3 Other boundary conditions
            assert_eq!($test_function(&[0xED, 0x9F, 0xBF]), true);
            assert_eq!($test_function(&[0xEE, 0x80, 0x80]), true);
            assert_eq!($test_function(&[0xEF, 0xBF, 0xBD]), true);
            assert_eq!($test_function(&[0xF4, 0x8F, 0xBF, 0xBF]), true);
            assert_eq!($test_function(&[0xF4, 0x90, 0x80, 0x80]), false);

            // 3.1  Unexpected continuation bytes
            assert_eq!($test_function(&[0x80]), false);
            assert_eq!($test_function(&[0xbf]), false);
            assert_eq!($test_function(&[0x80, 0xBF]), false);
            assert_eq!($test_function(&[0x80, 0xBF, 0x80]), false);
            assert_eq!($test_function(&[0x80, 0xBF, 0x80, 0xBF]), false);
            assert_eq!($test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80]), false);
            assert_eq!($test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80, 0xBF]), false);
            assert_eq!(
                $test_function(&[0x80, 0xBF, 0x80, 0xBF, 0x80, 0xBF, 0x80]),
                false
            );

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
            assert_eq!($test_function(&continuation_bytes), false);
            for &b in continuation_bytes.iter() {
                assert_eq!($test_function(&[b]), false);
            }

            // 3.2  Lonely start characters
            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_2 = [
                0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7,
                0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
                0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7,
                0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
            ];
            assert_eq!($test_function(&lonely_start_characters_2), false);
            for &b in &lonely_start_characters_2 {
                assert_eq!($test_function(&[b]), false);
            }

            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_3 = [
                0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7,
                0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
            ];
            assert_eq!($test_function(&lonely_start_characters_3), false);
            for &b in &lonely_start_characters_3 {
                assert_eq!($test_function(&[b]), false);
            }

            #[cfg_attr(rustfmt, rustfmt_skip)]
            let lonely_start_characters_4 = [
                0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7,
            ];
            assert_eq!($test_function(&lonely_start_characters_4), false);
            for &b in &lonely_start_characters_4 {
                assert_eq!($test_function(&[b]), false);
            }

            let lonely_start_characters_5 = [0xF8, 0xF9, 0xFA, 0xFB];
            assert_eq!($test_function(&lonely_start_characters_5), false);
            for &b in &lonely_start_characters_5 {
                assert_eq!($test_function(&[b]), false);
            }

            let lonely_start_characters_6 = [0xFC, 0xFD];
            assert_eq!($test_function(&lonely_start_characters_6), false);
            for &b in &lonely_start_characters_6 {
                assert_eq!($test_function(&[b]), false);
            }

            // 3.3 Sequences with last continuation byte missing
            assert_eq!($test_function(&[0xC0]), false);
            assert_eq!($test_function(&[0xE0, 0x80]), false);
            assert_eq!($test_function(&[0xF0, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xF8, 0x80, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xDF]), false);
            assert_eq!($test_function(&[0xEF, 0xBF]), false);
            assert_eq!($test_function(&[0xF7, 0xBF, 0xBF]), false);
            assert_eq!($test_function(&[0xFB, 0xBF, 0xBF, 0xBF]), false);
            assert_eq!($test_function(&[0xFD, 0xBF, 0xBF, 0xBF, 0xBF]), false);

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
            assert_eq!($test_function(&incomplete), false);

            // 3.5 Impossible bytes
            assert_eq!($test_function(&[0xFE]), false);
            assert_eq!($test_function(&[0xFF]), false);
            assert_eq!($test_function(&[0xFE, 0xFE, 0xFF, 0xFF]), false);

            // 4. Overlong sequences
            assert_eq!($test_function(&[0xC0, 0xAF]), false);
            assert_eq!($test_function(&[0xE0, 0x80, 0xAF]), false);
            assert_eq!($test_function(&[0xF0, 0x80, 0x80, 0xAF]), false);
            assert_eq!($test_function(&[0xF8, 0x80, 0x80, 0x80, 0xAF]), false);
            assert_eq!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0xAF]), false);

            assert_eq!($test_function(&[0xC0, 0x80]), false);
            assert_eq!($test_function(&[0xE0, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xF0, 0x80, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xF8, 0x80, 0x80, 0x80, 0x80]), false);
            assert_eq!($test_function(&[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80]), false);

            // 5. Illegal code positions
            assert_eq!($test_function(&[0xed, 0xa0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xad, 0xbf]), false);
            assert_eq!($test_function(&[0xed, 0xae, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xaf, 0xbf]), false);
            assert_eq!($test_function(&[0xed, 0xb0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xbe, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xbf, 0xbf]), false);

            assert_eq!($test_function(&[0xed, 0xa0, 0x80, 0xed, 0xb0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xa0, 0x80, 0xed, 0xbf, 0xbf]), false);
            assert_eq!($test_function(&[0xed, 0xad, 0xbf, 0xed, 0xb0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xad, 0xbf, 0xed, 0xbf, 0xbf]), false);
            assert_eq!($test_function(&[0xed, 0xae, 0x80, 0xed, 0xb0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xae, 0x80, 0xed, 0xbf, 0xbf]), false);
            assert_eq!($test_function(&[0xed, 0xaf, 0xbf, 0xed, 0xb0, 0x80]), false);
            assert_eq!($test_function(&[0xed, 0xaf, 0xbf, 0xed, 0xbf, 0xbf]), false);

            // Test data from files in the `../props` directory.
            assert_eq!($test_function(UTF8_SAMPLE_OK.as_bytes()), true);
            assert_eq!($test_function(ASCII_SAMPLE_OK.as_bytes()), true);
            assert_eq!($test_function(MOSTLY_ASCII_SAMPLE_OK.as_bytes()), true);
            assert_eq!($test_function(ALL_UTF8_CHARACTERS.as_bytes()), true);
            assert_eq!($test_function(ALL_UTF8_CHARACTERS_WITH_GARBAGE), false);
            assert_eq!($test_function(RANDOM_BYTES), false);
        };
    }

    #[test]
    fn test_lemire_avx() {
        use super::lemire::avx::is_utf8;
        create_tests!(is_utf8);
    }

    #[test]
    fn test_lemire_avx_ascii() {
        use super::lemire::avx::is_utf8_ascii_path;
        create_tests!(is_utf8_ascii_path);
    }

    #[test]
    fn test_lemire_sse() {
        use super::lemire::sse::is_utf8;
        create_tests!(is_utf8);
    }

    #[test]
    fn test_libcore() {
        use super::libcore::is_utf8;
        create_tests!(is_utf8);
    }
}
