//! Range algorithm
//!
//! The algoritm's implementation is adapted from Yibo Cai's UTF-8 library
//! which can be found [on GitHub][1].
//!
//! [1]: https://github.com/cyb70289/utf8
#[cfg(any(all(target_feature = "sse4.1"), dox))]
#[doc(cfg(all(target_feature = "sse4.1")))]
pub mod sse;
