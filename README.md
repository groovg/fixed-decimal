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

```cpp
#include <fixed_decimal.hpp>
using namespace fixed_decimal;

const auto price = Price::from_string("65000.25").value();
const auto qty = Qty::from_string("0.5").value();
const auto notional = price.mul_qty(qty);
assert(notional.to_string() == "32500.125000000");
```

## Layout

| Path    | Language | Status |
|---------|----------|--------|
| [`rust/`](rust/) | Rust (1.81+, `no_std`) | implemented ([crate README](rust/README.md)) |
| [`cpp/`](cpp/)  | C++23, header-only | implemented — same contract |
| [`tests/vectors.csv`](tests/vectors.csv) | shared | cross-language conformance gate |

Both sides are specified by one shared [`CONTRACT.md`](CONTRACT.md) (representation,
scale, rounding, overflow, parse/format, ticks) so a value is interpreted identically on
either side — which is what lets the same raw mantissa cross a shared-memory boundary
between a Rust and a C++ process (see the polyglot stack project) and still mean the same
number.

## Conformance

The contract is enforced, not just documented: `rust/examples/gen_vectors.rs`
deterministically generates ~1300 golden vectors from the Rust implementation — every
rounding mode across `div_round`, parse (including every rejection class), format,
add, mul, div, rescale, `Price×Qty→Notional` and its inverses, and tick arithmetic —
into [`tests/vectors.csv`](tests/vectors.csv). Both test suites replay the same file;
CI fails if either side drifts by a single mantissa unit. The C++ side additionally
pins the unit-safety rules at compile time (`static_assert` over concepts: `Price + Qty`,
`Price * Price`, cross-scale mixes do not compile).

```sh
cd rust && cargo test                         # unit + property + conformance
cmake -S cpp -B cpp/build && cmake --build cpp/build && ctest --test-dir cpp/build
cd rust && cargo run --example gen_vectors    # regenerate the golden file
```

The C++ header requires `__int128` (GCC/Clang). MSVC needs a two-limb shim that is not
written yet — see deferred work.

## Design (short)

`Fixed<SCALE, Unit, Repr>`: an `i64` mantissa (`i128` for `Notional`) with `value =
mantissa · 10^-SCALE`. `SCALE` and `Unit` are compile-time, so scale mismatches and
unit mix-ups (`Price + Qty`, `Price * Price`) are compile errors — in both languages
(Rust type system / C++ requires-clauses). Default rounding is HalfEven via one shared
`div_round` primitive used by arithmetic, parse and ticks alike; overflow is checked
(`Option` / `std::optional`; operators panic in Rust and throw `std::overflow_error`
in C++, never wrap). Full rationale is in `CONTRACT.md`.

## Status / deferred

Both libraries are complete and conformance-gated. The Rust side additionally has
property tests (400k iterations) and a benchmark vs `rust_decimal`; the C++ side pins
unit safety with compile-time asserts. Tracked follow-ups: an MSVC `__int128` shim,
Rust `trybuild` compile-fail tests (the C++ side already has their `static_assert`
equivalent), and a `serde` feature.

## License

MIT — see [LICENSE](LICENSE).
