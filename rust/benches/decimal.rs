use core::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fixed_decimal::Decimal;
use rust_decimal::Decimal as RustDecimal;

fn bench(c: &mut Criterion) {
    let a = Decimal::<9>::from_str("123.456789").unwrap();
    let b = Decimal::<9>::from_str("987.654321").unwrap();
    let fa = 123.456789_f64;
    let fb = 987.654321_f64;
    let ra = RustDecimal::from_str("123.456789").unwrap();
    let rb = RustDecimal::from_str("987.654321").unwrap();

    // Fixed-scale multiply: both sides multiply and rescale the result back to 9
    // decimal places. rust_decimal's bare `*` keeps a growing scale (no rescale),
    // so the fair comparison rounds it back with round_dp(9).
    let mut mul = c.benchmark_group("mul-rescale-9dp");
    mul.bench_function("fixed-decimal", |bn| {
        bn.iter(|| black_box(a).mul(black_box(b)))
    });
    mul.bench_function("f64", |bn| bn.iter(|| black_box(fa) * black_box(fb)));
    mul.bench_function("rust_decimal", |bn| {
        bn.iter(|| (black_box(ra) * black_box(rb)).round_dp(9))
    });
    mul.finish();

    let mut parse = c.benchmark_group("parse");
    parse.bench_function("fixed-decimal", |bn| {
        bn.iter(|| Decimal::<9>::from_str(black_box("123.456789")).unwrap())
    });
    parse.bench_function("rust_decimal", |bn| {
        bn.iter(|| RustDecimal::from_str(black_box("123.456789")).unwrap())
    });
    parse.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
