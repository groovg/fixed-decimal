# Decimal Contract

The Rust and C++ implementations follow this contract exactly so that a value
computed on either side is bit-identical. Both consume the same `tests/vectors.csv`
golden file as the cross-language gate.

## Representation

- Signed integer mantissa `M`; value = `M * 10^(-SCALE)`. Base-10 only.
- `SCALE` (number of fractional digits) lives in the **type**, not the object.
- Backing integer (`Repr`) is `i64` except `Notional`, which is `i128`
  (a `price * qty` product overflows `i64` at realistic sizes).
- Zero is the single mantissa `0`; negative zero is never constructed or printed.

## Locked types

Unit identity is a phantom tag, not the scale (Price and Notional are both scale 9
yet must never mix).

| Alias | SCALE | Repr |
|-------|-------|------|
| `Price` | 9 | i64 |
| `Qty` | 9 | i64 |
| `Notional` | 9 | i128 |
| `Decimal<S>` (unitless) | S | i64 |
| `Price4`, `Qty4` (ITCH) | 4 | i64 |

`SCALE` is range-checked at compile time: `0..=18` for i64, `0..=38` for i128.

**Allowed:** same-unit `+ -`, unary `-`, integer scalar `* /` (with rounding),
`Price.mul_qty(Qty) -> Notional`, `Notional / Price -> Qty`, `Notional / Qty -> Price`.
**Compile errors (proven by negative tests):** `Price + Qty`, `Price * Price`,
`Price + Notional`, comparing `Price` to `Qty`, mixing two scales without an
explicit convert.

## Rounding

Default **HalfEven** (banker's) for every value-losing op. `+` and `-` never round.

`Round { HalfEven, HalfUp, HalfDown, TowardZero, AwayFromZero, Floor, Ceil }`.
Ties are sign-symmetric (decide on the magnitude, then reapply the sign):

- `HalfUp` = half away from zero (`2.5 -> 3`, `-2.5 -> -3`).
- `HalfDown` = half toward zero. `TowardZero` = truncate. `AwayFromZero` = always away.
- `Floor` = toward -inf, `Ceil` = toward +inf.

HalfEven known-answer table (both languages must match):
`0.5->0, 1.5->2, 2.5->2, 3.5->4, -0.5->0, -1.5->-2, -2.5->-2, -3.5->-4`.

### `div_round` (the single shared primitive)

```
div_round(num: i128, den: i128 /* != 0 */, mode) -> i128:
  s = sign(num) * sign(den);  n = |num|;  d = |den|
  q = n / d;  r = n % d;  r2 = 2*r
  E (extra increment, on the magnitude):
    HalfEven:     r2 > d, or (r2 == d and q is odd)
    HalfUp:       r2 >= d
    AwayFromZero: r > 0
    HalfDown:     r2 > d
    TowardZero:   never
    Floor:        s < 0 and r > 0
    Ceil:         s > 0 and r > 0
  return s * (q + E)
```

Port this verbatim — never re-derive with native `round()`, never touch float.

## Arithmetic

All `*` / `/` go through a 128-bit intermediate, round once, then narrow with a
checked range test.

- **mul** `a*b` at result scale R: `p = (i128)a.M * b.M` (scale = a_scale+b_scale);
  `m = div_round(p, 10^(a_scale+b_scale-R), mode)`; narrow to Repr (checked).
  `Price.mul_qty(Qty)`: single `price*qty` then one rescale to scale 9.
- **div** `a/b` at result scale R: `num = (i128)a.M * 10^(R + b_scale - a_scale)`;
  `m = div_round(num, b.M, mode)`; narrow (checked). `b.M == 0` -> None / panic.
- Multiplication-with-rounding is **not associative**: notional is always one
  `price*qty` with a single rescale, never a chained product.

## Overflow

Checked by default. `checked_add/sub/mul/div/rescale/convert` return `Option` /
`std::optional` and never wrap. Operators (`+ - ` and scalar `* /`) call the checked
op and panic (Rust) / throw `std::overflow_error` (C++) on overflow or divide-by-zero.
Named `saturating_*` and `wrapping_*` exist but are never defaults and never operators.

## Parse

Hand-rolled, float-free, identical grammar:
`[+|-] digits [ . digits ]`.

- `from_str` / `from_string` — rounds excess fraction digits (HalfEven).
- `from_str_exact` — rejects if a dropped fraction digit is non-zero.
- `from_str_rounded(s, mode)` — explicit mode.

Accept: leading zeros, trailing zeros, leading `+`, `-0`/`+0` -> 0, fewer fraction
digits (right-padded). Reject (error, never panic/wrap): empty, lone `.`, lone sign,
`1.`, `.5`, any whitespace, thousands separators, exponent, multiple dots, non-digit,
mantissa overflow. Errors: `Empty, InvalidChar, Overflow, TooManyDigits` (exact mode).

## Format

Float-free. Emits `-` only when `M < 0`; integer part with no leading zeros (single
`0` for `|value| < 1`); then, if `SCALE > 0`, `.` and exactly `SCALE` zero-padded
digits (trailing zeros preserved). `from_str(to_string(x)) == x` for all representable
`x`. `to_string_trim` strips trailing zeros for display; a no-alloc `write(buf)`
variant is provided for the hot path.

## Tick arithmetic

`tick` must be the same unit+scale and `tick.M > 0`. All integer math via `div_round`.

- `round_to_tick(tick, mode = HalfEven)`, `floor_to_tick`, `ceil_to_tick`
  (directional placement: bids floor, asks ceil).
- `is_on_tick(tick)`, `next_tick`, `prev_tick`.
- `ticks_between(a, b, tick)` -> exact-or-`None` (off-grid is a bug);
  `ticks_between_trunc` -> truncated distance.
