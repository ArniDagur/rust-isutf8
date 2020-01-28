//! Range algorithm
//!
//! The algoritm's implementation is adapted from Yibo Cai's UTF-8 library
//! which can be found [on GitHub][1].
//!
//! ## Warning:
//! This module is expertimental. There may be breaking API changes.
//!
//! [1]: https://github.com/cyb70289/utf8
#[cfg(any(all(target_feature = "avx", target_feature = "avx2"), dox))]
#[cfg_attr(dox, doc(cfg(all(target_feature = "avx", target_feature = "avx2"))))]
pub mod avx;
#[cfg(any(all(target_feature = "sse4.1"), dox))]
#[cfg_attr(dox, doc(cfg(all(target_feature = "sse4.1"))))]
pub mod sse;
