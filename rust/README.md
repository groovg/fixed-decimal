# fixed-decimal (Rust)

Exact fixed-point decimal for prices and quantities — `no_std`, 8 bytes, no floating
point in the value path.

```rust
use fixed_decimal::{Price, Qty};

let price: Price = "65000.25".parse().unwrap();
let qty: Qty = "0.5".parse().unwrap();

let notional = price.mul_qty(qty);          // Price * Qty -> Notional
assert_eq!(notional.to_string(), "32500.125000000");

// dimensional safety: this does not compile —
// let bad = price + qty;                   // Price + Qty is a type error
```

`scale` and `unit` live in the type, so `Price`, `Qty` and `Notional` can't be mixed
up, and a `Price + Qty` is a compile error. Values are an `i64` mantissa (`i128` for
`Notional`) at a fixed base-10 scale.

## Highlights

- **Exact:** `"0.1".parse::<Price>()? + "0.2".parse()? == "0.3".parse()?`.
- **HalfEven (banker's) rounding** by default on every value-losing op; six other
  modes available. `+`/`-` never round.
- **Checked overflow** everywhere (`checked_*` return `Option`; operators panic — never
  silent wrap). Multiplication that can round is a named method (`mul`, `mul_qty`), not
  an operator, so the rounding is always explicit.
- **`no_std`**, `const`-friendly, 8 bytes for `Price`/`Qty`.
- **Compile-fail doctests** pin the unit algebra: `Price + Qty`, mixed scales, and
  cross-unit assignment are type errors, and stay that way.
- **`serde` feature** (off by default): serializes as the exact decimal string
  (`"1.250000000"`), deserializes with `FromStr` semantics; allocation-free, `no_std`.
  `to_f64_lossy` is the one explicitly lossy escape hatch.

## Benchmarks

AMD Ryzen 9 9950X3D, rustc 1.95 (LTO). Fixed-scale multiply means both sides rescale
the product back to 9 decimal places (`rust_decimal`'s bare `*` keeps a growing scale,
so the fair comparison rounds it with `round_dp(9)`).

| op (scale 9)            | fixed-decimal | rust_decimal | f64 |
|-------------------------|--------------:|-------------:|----:|
| multiply (rescaled)     | **4.0 ns**    | 9.8 ns       | 0.36 ns (inexact) |
| parse from string       | 11.8 ns       | **9.6 ns**   | — |

The fixed scale wins on multiply (one divide by a constant vs a general rescale), at
half the footprint (8 vs 16 bytes). `rust_decimal`'s hand-tuned parser is a bit faster;
`f64` is fastest but cannot represent `0.1` — the reason this type exists.

## Test & benchmark

```sh
cargo test
cargo bench --bench decimal
```

See the repository [CONTRACT.md](../CONTRACT.md) for the exact semantics (shared with the
planned C++ twin) and [README](../README.md) for the project overview.
