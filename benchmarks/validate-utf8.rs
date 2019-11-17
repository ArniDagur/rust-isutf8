//! Benchmarks written by bluss
//!
//! cf <https://gist.github.com/bluss/bf45e07e711238e22b7a>

#[macro_use]
extern crate criterion;
use criterion::{Benchmark, Criterion, Throughput};

macro_rules! bench {
    ($name:ident, $path:expr) => {
        fn $name(c: &mut Criterion) {
            let bytes = include_bytes!($path);

            c.bench(
                stringify!($name),
                Benchmark::new("libcore", move |b| {
                    b.iter(|| ::is_utf8::libcore::is_utf8(bytes))
                })
                .with_function("lemire_sse", move |b| {
                    b.iter(|| ::is_utf8::lemire::sse::is_utf8(bytes))
                })
                .with_function("lemire_avx", move |b| {
                    b.iter(|| ::is_utf8::lemire::avx::is_utf8(bytes))
                })
                .with_function("lemire_avx_ascii_path", move |b| {
                    b.iter(|| ::is_utf8::lemire::avx::is_utf8_ascii_path(bytes))
                })
                .throughput(Throughput::Bytes(bytes.len() as u64)),
            );
        }
    };
}

bench!(random_bytes, "../props/random_bytes.bin");
bench!(mostly_ascii, "../props/mostly_ascii_sample_ok.txt");
bench!(ascii, "../props/ascii_sample_ok.txt");
bench!(utf8, "../props/utf8_sample_ok.txt");
bench!(all_utf8, "../props/utf8-characters-0-0x10ffff.txt");
bench!(all_utf8_with_garbage, "../props/utf8-characters-0-0x10ffff-with-garbage.bin");

criterion_group!(
    benches,
    random_bytes,
    mostly_ascii,
    ascii,
    utf8,
    all_utf8,
    all_utf8_with_garbage
);
criterion_main!(benches);
