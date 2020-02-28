#![no_main]
use is_utf8::{lemire, libcore, range};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let lemire_avx_ascii = lemire::avx::is_utf8_ascii_path(data);
    let lemire_avx = lemire::avx::is_utf8(data);
    let lemire_sse = lemire::sse::is_utf8(data);
    let range_avx = range::avx::is_utf8(data);
    let range_sse = range::sse::is_utf8(data);
    let libcore = libcore::is_utf8(data);

    // Make sure that each implementation is in agreement with the others. I
    // chain the assert statements together like since to hopefully return a
    // more readable error in case of a panic.
    assert_eq!(lemire_avx_ascii, lemire_avx);
    assert_eq!(lemire_avx, lemire_sse);
    assert_eq!(lemire_sse, range_avx);
    assert_eq!(range_avx, range_sse);
    assert_eq!(range_sse, libcore);
});
