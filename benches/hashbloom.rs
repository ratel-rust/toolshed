#![feature(test)]
extern crate test;
extern crate fxhash;
extern crate toolshed;

use toolshed::bloom::Bloom;
use test::{Bencher, black_box};

macro_rules! generate_benches {
    ($($fx:ident, $bloom:ident, $s:expr),* $(,)*) => (
        $(
            #[bench]
            fn $fx(b: &mut Bencher) {
                let s = black_box($s);
                b.iter(|| {
                    fxhash::hash(&s)
                })
            }

            #[bench]
            fn $bloom(b: &mut Bencher) {
                let s = black_box($s);
                b.iter(|| {
                    s.bloom()
                })
            }
        )*
    )
}

generate_benches!(
    bench_fx_003, bench_bloom_003, "123",
    bench_fx_004, bench_bloom_004, "1234",
    bench_fx_011, bench_bloom_011, "12345678901",
    bench_fx_012, bench_bloom_012, "123456789012",
    bench_fx_023, bench_bloom_023, "12345678901234567890123",
    bench_fx_024, bench_bloom_024, "123456789012345678901234",
    bench_fx_068, bench_bloom_068, "11234567890123456789012345678901234567890123456789012345678901234567",
    bench_fx_132, bench_bloom_132, "112345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901",
);
