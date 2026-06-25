# fixed-decimal

[![CI](https://github.com/groovg/fixed-decimal/actions/workflows/ci.yml/badge.svg)](https://github.com/groovg/fixed-decimal/actions/workflows/ci.yml)

Exact fixed-point decimal arithmetic for prices and quantities — no floating point
anywhere in the value path.

`0.1 + 0.2 != 0.3` in binary floating point, and using `double` for money is an instant
red flag in any trading system. This is an integer-backed decimal with the scale and
unit fixed in the type: exact `+ - * /`, explicit rounding, exact parse/format
round-trips, and `Price`/`Qty`/`Notional` that can't be mixed up by accident.

```rust
use fixed_decimal::{Price, Qty};

let price: Price = "65000.25".parse().unwrap();
let qty: Qty = "0.5".parse().unwrap();
let notional = price.mul_qty(qty);     // Price * Qty -> Notional, one exact rescale
assert_eq!(notional.to_string(), "32500.125000000");
```

## Layout

| Path    | Language | Status |
|---------|----------|--------|
| [`rust/`](rust/) | Rust | implemented ([crate README](rust/README.md)) |
| `cpp/`  | C++20 | planned — same contract |

Both sides are specified by one shared [`CONTRACT.md`](CONTRACT.md) (representation,
scale, rounding, overflow, parse/format, ticks) so a value is interpreted identically on
either side — which is what lets the same raw mantissa cross a shared-memory boundary
between a Rust and a C++ process (see the polyglot stack project) and still mean the same
number.

## Design (short)

`Fixed<SCALE, Unit, Repr>`: an `i64` mantissa (`i128` for `Notional`) with `value =
mantissa · 10^-SCALE`. `SCALE` and `Unit` are compile-time, so scale mismatches and
unit mix-ups (`Price + Qty`, `Price * Price`) are compile errors. Default rounding is
HalfEven via one shared `div_round` primitive used by arithmetic, parse and ticks alike;
overflow is checked (operators panic, never wrap). Full rationale is in `CONTRACT.md`.

## Status / deferred

Rust library is complete (core type, 128-bit mul/div, cross-unit algebra, float-free
parse/format, tick arithmetic, property tests, benchmark). Tracked follow-ups: the C++
twin, an MSVC `__int128` shim, a shared `vectors.csv` cross-language conformance gate,
`trybuild` compile-fail tests, and a `serde` feature.

## License

MIT — see [LICENSE](LICENSE).
