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
    use super::lemire::avx;
    use super::lemire::sse;

    #[test]
    fn valid_sequences_sse() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(sse::validate_utf8_fast(&arr), true);
    }

    #[test]
    fn invalid_sequences_sse() {
        let arr = [0xfe, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(sse::validate_utf8_fast(&arr), false);
    }

    #[test]
    fn valid_sequences_avx() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 10, 11];
        assert_eq!(avx::validate_utf8_fast(&arr), true);
    }
}
