use std::fmt::Write as _;

use fixed_decimal::{div_round, Decimal, Notional, ParseError, Price, Qty, Round};

const MODES: [(Round, &str); 7] = [
    (Round::HalfEven, "HalfEven"),
    (Round::HalfUp, "HalfUp"),
    (Round::HalfDown, "HalfDown"),
    (Round::TowardZero, "TowardZero"),
    (Round::AwayFromZero, "AwayFromZero"),
    (Round::Floor, "Floor"),
    (Round::Ceil, "Ceil"),
];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn in_range(&mut self, lo: i64, hi: i64) -> i64 {
        let span = (hi - lo) as u64 + 1;
        lo + (self.next() % span) as i64
    }
}

fn none_or<T: ToString>(v: Option<T>) -> String {
    v.map(|x| x.to_string()).unwrap_or_else(|| "NONE".into())
}

fn parse_outcome(r: Result<Decimal<9>, ParseError>) -> String {
    match r {
        Ok(v) => v.raw().to_string(),
        Err(ParseError::Empty) => "E:Empty".into(),
        Err(ParseError::InvalidChar) => "E:InvalidChar".into(),
        Err(ParseError::Overflow) => "E:Overflow".into(),
        Err(ParseError::TooManyDigits) => "E:TooManyDigits".into(),
    }
}

fn main() {
    let mut rng = Lcg(0x5eed_f1c5_ed05_2026);
    let mut out = String::from("op,mode,sa,sb,a,b,c,expect\n");
    let mut row =
        |op: &str, mode: &str, sa: &str, sb: &str, a: &str, b: &str, c: &str, expect: &str| {
            writeln!(out, "{op},{mode},{sa},{sb},{a},{b},{c},{expect}").unwrap();
        };

    for (mode, name) in MODES {
        for num in [5i128, 15, 25, 35, -5, -15, -25, -35, 1, -1, 29, -29, 0] {
            row(
                "div_round",
                name,
                "",
                "",
                &num.to_string(),
                "10",
                "",
                &div_round(num, 10, mode).to_string(),
            );
        }
        for _ in 0..25 {
            let num = rng.in_range(-1_000_000_000_000, 1_000_000_000_000) as i128;
            let den = {
                let d = rng.in_range(1, 1_000_000);
                if rng.next() % 2 == 0 {
                    d
                } else {
                    -d
                }
            } as i128;
            row(
                "div_round",
                name,
                "",
                "",
                &num.to_string(),
                &den.to_string(),
                "",
                &div_round(num, den, mode).to_string(),
            );
        }
    }

    let corpus = [
        "0",
        "-0",
        "+0",
        "1",
        "-1",
        "+1.5",
        "61278.01",
        "-61278.01",
        "00042.10",
        "0.000000001",
        "-0.000000001",
        "0.123456789",
        "0.1234567891",
        "0.1234567895",
        "9223372036854.775807",
        "-9223372036854.775808",
        "9223372036854.775808",
        "99999999999999999999",
        "",
        ".",
        "-",
        "+",
        "1.",
        ".5",
        "1..2",
        "1.2.3",
        " 1",
        "1 ",
        "1;5",
        "1e5",
        "12345678901234567890123456789012345678901",
    ];
    for s in corpus {
        for (mode, name) in MODES {
            row(
                "parse",
                name,
                "9",
                "",
                s,
                "0",
                "",
                &parse_outcome(Decimal::<9>::from_str_rounded(s, mode)),
            );
        }
        row(
            "parse",
            "HalfEven",
            "9",
            "",
            s,
            "1",
            "",
            &parse_outcome(Decimal::<9>::from_str_exact(s)),
        );
    }

    for m in [
        0i64,
        1,
        -1,
        999_999_999,
        -999_999_999,
        1_234_567_890_123,
        -42,
        i64::MAX,
        i64::MIN,
    ] {
        row(
            "format",
            "HalfEven",
            "9",
            "",
            &m.to_string(),
            "",
            "",
            &Decimal::<9>::from_raw(m).to_string(),
        );
    }

    for _ in 0..40 {
        let a = rng.in_range(-4_000_000_000_000_000_000, 4_000_000_000_000_000_000);
        let b = rng.in_range(-4_000_000_000_000_000_000, 4_000_000_000_000_000_000);
        let (da, db) = (Decimal::<9>::from_raw(a), Decimal::<9>::from_raw(b));
        row(
            "add",
            "HalfEven",
            "9",
            "9",
            &a.to_string(),
            &b.to_string(),
            "",
            &none_or(da.checked_add(db).map(|v| v.raw())),
        );
    }
    row(
        "add",
        "HalfEven",
        "9",
        "9",
        &i64::MAX.to_string(),
        "1",
        "",
        "NONE",
    );
    row(
        "add",
        "HalfEven",
        "9",
        "9",
        &i64::MIN.to_string(),
        "-1",
        "",
        "NONE",
    );

    for (mode, name) in MODES {
        for _ in 0..20 {
            let a = rng.in_range(-3_000_000_000_000, 3_000_000_000_000);
            let b = rng.in_range(-3_000_000_000_000, 3_000_000_000_000);
            let (da, db) = (Decimal::<9>::from_raw(a), Decimal::<9>::from_raw(b));
            row(
                "mul",
                name,
                "9",
                "9",
                &a.to_string(),
                &b.to_string(),
                "",
                &none_or(da.checked_mul_round(db, mode).map(|v| v.raw())),
            );
            let d = if b == 0 { 1 } else { b };
            row(
                "div",
                name,
                "9",
                "9",
                &a.to_string(),
                &d.to_string(),
                "",
                &none_or(
                    da.checked_div_round(Decimal::<9>::from_raw(d), mode)
                        .map(|v| v.raw()),
                ),
            );
        }
        row("div", name, "9", "9", "12345", "0", "", "NONE");

        for m in [
            1_234_567_891i64,
            -1_234_567_891,
            50_000i64,
            -50_000,
            999_949_999,
            -999_950_000,
        ] {
            row(
                "rescale_9_to_4",
                name,
                "9",
                "4",
                &m.to_string(),
                "",
                "",
                &none_or(
                    Decimal::<9>::from_raw(m)
                        .checked_rescale_round::<4>(mode)
                        .map(|v| v.raw()),
                ),
            );
        }
        for m in [123_456i64, -123_456, 1, -1, 9_000_000_000_000_000] {
            row(
                "rescale_4_to_9",
                name,
                "4",
                "9",
                &m.to_string(),
                "",
                "",
                &none_or(
                    Decimal::<4>::from_raw(m)
                        .checked_rescale_round::<9>(mode)
                        .map(|v| v.raw()),
                ),
            );
        }

        for _ in 0..12 {
            let p = rng.in_range(1, 100_000_000_000_000);
            let q = rng.in_range(-10_000_000_000_000, 10_000_000_000_000);
            let (price, qty) = (Price::from_raw(p), Qty::from_raw(q));
            let notional = price.mul_qty_round(qty, mode);
            row(
                "mul_qty",
                name,
                "9",
                "9",
                &p.to_string(),
                &q.to_string(),
                "",
                &notional.raw().to_string(),
            );
            row(
                "div_price",
                name,
                "9",
                "9",
                &notional.raw().to_string(),
                &p.to_string(),
                "",
                &none_or(
                    notional
                        .checked_div_price_round(price, mode)
                        .map(|v| v.raw()),
                ),
            );
            if q != 0 {
                row(
                    "div_qty",
                    name,
                    "9",
                    "9",
                    &notional.raw().to_string(),
                    &q.to_string(),
                    "",
                    &none_or(notional.checked_div_qty_round(qty, mode).map(|v| v.raw())),
                );
            }
        }
        row("div_price", name, "9", "9", "1000000000", "0", "", "NONE");

        for _ in 0..10 {
            let tick = rng.in_range(1, 1_000_000_000);
            let a = rng.in_range(-1_000_000_000_000_000, 1_000_000_000_000_000);
            row(
                "round_tick",
                name,
                "9",
                "9",
                &a.to_string(),
                &tick.to_string(),
                "",
                &none_or(
                    Price::from_raw(a)
                        .round_to_tick_round(Price::from_raw(tick), mode)
                        .map(|v| v.raw()),
                ),
            );
        }
    }
    row("round_tick", "HalfEven", "9", "9", "12345", "0", "", "NONE");

    for _ in 0..15 {
        let tick = rng.in_range(1, 1_000_000);
        let a = rng.in_range(-1_000_000_000, 1_000_000_000) * 7;
        let steps = rng.in_range(-1_000, 1_000);
        let b = a + steps * tick;
        row(
            "ticks_between",
            "HalfEven",
            "9",
            "9",
            &a.to_string(),
            &b.to_string(),
            &tick.to_string(),
            &none_or(Price::from_raw(a).ticks_between(Price::from_raw(b), Price::from_raw(tick))),
        );
        row(
            "ticks_between",
            "HalfEven",
            "9",
            "9",
            &a.to_string(),
            &(b + 1).to_string(),
            &tick.to_string(),
            &none_or(
                Price::from_raw(a).ticks_between(Price::from_raw(b + 1), Price::from_raw(tick)),
            ),
        );
    }

    let big = Notional::from_raw(170_000_000_000_000_000_001i128 * 1_000_000_000);
    row(
        "div_price",
        "HalfEven",
        "9",
        "9",
        &big.raw().to_string(),
        "1000000000",
        "",
        "NONE",
    );

    std::fs::create_dir_all("../tests").expect("create tests dir");
    std::fs::write("../tests/vectors.csv", out).expect("write vectors.csv");
    println!("wrote ../tests/vectors.csv");
}
