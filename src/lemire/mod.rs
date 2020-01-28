//! Daniel Lemire's UTF-8 validation algorithm
//!
//! The algorithm is discussed in [a blog post][1] by him.
//!
//! [1]: https://lemire.me/blog/2018/10/19/validating-utf-8-bytes-using-only-0-45-cycles-per-byte-avx-edition/

#[cfg(any(all(target_feature = "avx", target_feature = "avx2"), dox))]
#[cfg_attr(dox, doc(cfg(all(target_feature = "avx", target_feature = "avx2"))))]
pub mod avx;
#[cfg(any(
    all(
        target_feature = "sse2",
        target_feature = "ssse3",
        target_feature = "sse4.1",
    ),
    dox
))]
#[cfg_attr(
    dox,
    doc(cfg(all(
        target_feature = "sse2",
        target_feature = "ssse3",
        target_feature = "sse4.1",
    )))
)]
pub mod sse;
