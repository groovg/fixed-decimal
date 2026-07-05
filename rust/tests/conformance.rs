use fixed_decimal::{div_round, Decimal, Notional, ParseError, Price, Qty, Round};

fn mode(s: &str) -> Round {
    match s {
        "HalfEven" => Round::HalfEven,
        "HalfUp" => Round::HalfUp,
        "HalfDown" => Round::HalfDown,
        "TowardZero" => Round::TowardZero,
        "AwayFromZero" => Round::AwayFromZero,
        "Floor" => Round::Floor,
        "Ceil" => Round::Ceil,
        other => panic!("unknown mode {other}"),
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

#[test]
fn replays_the_shared_vectors() {
    let csv = std::fs::read_to_string("../tests/vectors.csv").expect("vectors.csv");
    let mut rows = 0usize;
    for line in csv.lines().skip(1) {
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.splitn(8, ',').collect();
        let (op, m, a, b, c, expect) = (f[0], mode(f[1]), f[4], f[5], f[6], f[7]);
        rows += 1;
        let i64a = || a.parse::<i64>().unwrap();
        let i64b = || b.parse::<i64>().unwrap();
        let got = match op {
            "div_round" => div_round(a.parse().unwrap(), b.parse().unwrap(), m).to_string(),
            "parse" => parse_outcome(if b == "1" {
                Decimal::<9>::from_str_exact(a)
            } else {
                Decimal::<9>::from_str_rounded(a, m)
            }),
            "format" => Decimal::<9>::from_raw(i64a()).to_string(),
            "add" => none_or(
                Decimal::<9>::from_raw(i64a())
                    .checked_add(Decimal::<9>::from_raw(i64b()))
                    .map(|v| v.raw()),
            ),
            "mul" => none_or(
                Decimal::<9>::from_raw(i64a())
                    .checked_mul_round(Decimal::<9>::from_raw(i64b()), m)
                    .map(|v| v.raw()),
            ),
            "div" => none_or(
                Decimal::<9>::from_raw(i64a())
                    .checked_div_round(Decimal::<9>::from_raw(i64b()), m)
                    .map(|v| v.raw()),
            ),
            "rescale_9_to_4" => none_or(
                Decimal::<9>::from_raw(i64a())
                    .checked_rescale_round::<4>(m)
                    .map(|v| v.raw()),
            ),
            "rescale_4_to_9" => none_or(
                Decimal::<4>::from_raw(i64a())
                    .checked_rescale_round::<9>(m)
                    .map(|v| v.raw()),
            ),
            "mul_qty" => Price::from_raw(i64a())
                .mul_qty_round(Qty::from_raw(i64b()), m)
                .raw()
                .to_string(),
            "div_price" => none_or(
                Notional::from_raw(a.parse().unwrap())
                    .checked_div_price_round(Price::from_raw(i64b()), m)
                    .map(|v| v.raw()),
            ),
            "div_qty" => none_or(
                Notional::from_raw(a.parse().unwrap())
                    .checked_div_qty_round(Qty::from_raw(i64b()), m)
                    .map(|v| v.raw()),
            ),
            "round_tick" => none_or(
                Price::from_raw(i64a())
                    .round_to_tick_round(Price::from_raw(i64b()), m)
                    .map(|v| v.raw()),
            ),
            "ticks_between" => none_or(
                Price::from_raw(i64a())
                    .ticks_between(Price::from_raw(i64b()), Price::from_raw(c.parse().unwrap())),
            ),
            other => panic!("unknown op {other}"),
        };
        assert_eq!(got, expect, "row {rows}: {line}");
    }
    assert!(rows > 500, "vectors file looks truncated ({rows} rows)");
}
