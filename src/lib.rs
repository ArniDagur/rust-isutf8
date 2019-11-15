// vim: fdm=marker:et:ts=4:sts=4:sw=4
//
// lib.rs
// Copyright (c) 2019 Árni Dagur <arni@dagur.eu> MIT license
//
#![no_std]
#![feature(stdsimd)]
pub mod simdutf8check_avx;
pub mod simdutf8check_sse;

#[cfg(test)]
mod tests {
    use super::simdutf8check_avx;
    use super::simdutf8check_sse;

    #[test]
    fn valid_sequences_sse() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(simdutf8check_sse::validate_utf8_fast(&arr), true);
    }

    #[test]
    fn invalid_sequences_sse() {
        let arr = [0xfe, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(simdutf8check_sse::validate_utf8_fast(&arr), false);
    }

    #[test]
    fn valid_sequences_avx() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 10, 11];
        assert_eq!(simdutf8check_avx::validate_utf8_fast(&arr), true);
    }
}
