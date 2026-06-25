# fixed-decimal

Exact fixed-point decimal arithmetic for prices and quantities, implemented in
**Rust** and **C++20** to a single shared semantic contract — no floating point
anywhere in the value path.

`0.1 + 0.2 != 0.3` in binary floating point, and using `double` for money is an
instant red flag in any trading system. This is an integer-backed decimal with a
fixed scale: exact `+ - * /`, explicit rounding, and exact parse/format round-trips.

## Layout

| Path    | Language | Build |
|---------|----------|-------|
| `rust/` | Rust     | `cargo` |
| `cpp/`  | C++20    | CMake + Ninja |

Both sides share identical semantics (scale, rounding, overflow policy, parse and
format rules) so values computed on either side agree bit-for-bit.

## Status

Work in progress.

## License

MIT — see [LICENSE](LICENSE).
