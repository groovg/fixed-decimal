#pragma once

#include <compare>
#include <concepts>
#include <cstdint>
#include <expected>
#include <functional>
#include <optional>
#include <ostream>
#include <stdexcept>
#include <string>
#include <string_view>

#if !defined(__SIZEOF_INT128__)
#error "fixed-decimal requires __int128 (GCC/Clang); an MSVC two-limb shim is not provided yet"
#endif

namespace fixed_decimal {

using int128 = __int128;
using uint128 = unsigned __int128;

enum class Round {
    HalfEven,
    HalfUp,
    HalfDown,
    TowardZero,
    AwayFromZero,
    Floor,
    Ceil,
};

constexpr int128 int128_max = static_cast<int128>((static_cast<uint128>(1) << 127) - 1);
constexpr int128 int128_min = -int128_max - 1;

namespace detail {

constexpr uint128 unsigned_abs(int128 v) {
    return v < 0 ? uint128{0} - static_cast<uint128>(v) : static_cast<uint128>(v);
}

constexpr int128 pow10_entry(unsigned n) {
    int128 r = 1;
    for (unsigned i = 0; i < n; ++i) {
        r *= 10;
    }
    return r;
}

template <typename T>
constexpr std::optional<T> checked_add(T a, T b) {
    T out;
    if (__builtin_add_overflow(a, b, &out)) {
        return std::nullopt;
    }
    return out;
}

template <typename T>
constexpr std::optional<T> checked_sub(T a, T b) {
    T out;
    if (__builtin_sub_overflow(a, b, &out)) {
        return std::nullopt;
    }
    return out;
}

template <typename T>
constexpr std::optional<T> checked_mul(T a, T b) {
    T out;
    if (__builtin_mul_overflow(a, b, &out)) {
        return std::nullopt;
    }
    return out;
}

}  // namespace detail

constexpr int128 pow10(unsigned n) {
    return detail::pow10_entry(n);
}

constexpr int128 div_round(int128 num, int128 den, Round mode) {
    const int sign = (num > 0 ? 1 : num < 0 ? -1 : 0) * (den > 0 ? 1 : -1);
    const uint128 n = detail::unsigned_abs(num);
    const uint128 d = detail::unsigned_abs(den);
    const uint128 q = n / d;
    const uint128 r = n % d;
    const uint128 half = d - r;
    bool away = false;
    switch (mode) {
        case Round::HalfEven: away = r > half || (r == half && (q & 1) == 1); break;
        case Round::HalfUp: away = r >= half; break;
        case Round::HalfDown: away = r > half; break;
        case Round::TowardZero: away = false; break;
        case Round::AwayFromZero: away = r > 0; break;
        case Round::Floor: away = sign < 0 && r > 0; break;
        case Round::Ceil: away = sign > 0 && r > 0; break;
    }
    const uint128 magnitude = q + (away ? 1 : 0);
    return sign < 0 ? -static_cast<int128>(magnitude) : static_cast<int128>(magnitude);
}

enum class ParseError {
    Empty,
    InvalidChar,
    Overflow,
    TooManyDigits,
};

template <typename Rep>
struct mantissa_traits;

template <>
struct mantissa_traits<std::int64_t> {
    static constexpr unsigned max_scale = 18;
    static constexpr std::int64_t min_value = INT64_MIN;
    static constexpr std::int64_t max_value = INT64_MAX;

    static constexpr int128 to_i128(std::int64_t v) { return v; }

    static constexpr std::optional<std::int64_t> from_i128(int128 v) {
        if (v < static_cast<int128>(INT64_MIN) || v > static_cast<int128>(INT64_MAX)) {
            return std::nullopt;
        }
        return static_cast<std::int64_t>(v);
    }
};

template <>
struct mantissa_traits<int128> {
    static constexpr unsigned max_scale = 38;
    static constexpr int128 min_value = int128_min;
    static constexpr int128 max_value = int128_max;

    static constexpr int128 to_i128(int128 v) { return v; }
    static constexpr std::optional<int128> from_i128(int128 v) { return v; }
};

struct PriceTag {};
struct QtyTag {};
struct NotionalTag {};
struct PlainTag {};

template <unsigned Scale, typename Unit, typename Rep = std::int64_t>
class Fixed {
    static_assert(Scale <= mantissa_traits<Rep>::max_scale,
                  "SCALE too large for the backing integer");

public:
    using rep = Rep;
    static constexpr unsigned scale = Scale;

    constexpr Fixed() = default;

    static constexpr Fixed from_raw(Rep mantissa) { return Fixed(mantissa); }
    constexpr Rep raw() const { return mantissa_; }

    static constexpr Fixed zero() { return from_raw(0); }
    static constexpr Fixed min() { return from_raw(mantissa_traits<Rep>::min_value); }
    static constexpr Fixed max() { return from_raw(mantissa_traits<Rep>::max_value); }
    static constexpr int128 scale_factor() { return pow10(Scale); }

    static constexpr std::optional<Fixed> checked_from_int(std::int64_t value) {
        const auto mantissa = detail::checked_mul<int128>(value, pow10(Scale));
        if (!mantissa) {
            return std::nullopt;
        }
        return narrow(*mantissa);
    }

    static constexpr Fixed from_int(std::int64_t value) {
        const auto v = checked_from_int(value);
        if (!v) {
            throw std::overflow_error("fixed-decimal: from_int overflow");
        }
        return *v;
    }

    static constexpr Fixed one() { return from_int(1); }

    static constexpr std::optional<Fixed> try_from_parts(std::int64_t whole, std::int64_t frac) {
        const int128 factor = pow10(Scale);
        if (frac < 0 || static_cast<int128>(frac) >= factor) {
            return std::nullopt;
        }
        const auto scaled = detail::checked_mul<int128>(whole, factor);
        if (!scaled) {
            return std::nullopt;
        }
        const auto mantissa = whole < 0 ? detail::checked_sub<int128>(*scaled, frac)
                                        : detail::checked_add<int128>(*scaled, frac);
        if (!mantissa) {
            return std::nullopt;
        }
        return narrow(*mantissa);
    }

    constexpr bool is_zero() const { return mantissa_ == 0; }

    constexpr std::optional<Fixed> checked_add(Fixed rhs) const {
        const auto m = detail::checked_add(mantissa_, rhs.mantissa_);
        if (!m) {
            return std::nullopt;
        }
        return from_raw(*m);
    }

    constexpr std::optional<Fixed> checked_sub(Fixed rhs) const {
        const auto m = detail::checked_sub(mantissa_, rhs.mantissa_);
        if (!m) {
            return std::nullopt;
        }
        return from_raw(*m);
    }

    constexpr std::optional<Fixed> checked_neg() const {
        const auto m = detail::checked_sub<Rep>(0, mantissa_);
        if (!m) {
            return std::nullopt;
        }
        return from_raw(*m);
    }

    constexpr Fixed saturating_add(Fixed rhs) const {
        const auto m = detail::checked_add(mantissa_, rhs.mantissa_);
        if (m) {
            return from_raw(*m);
        }
        return rhs.mantissa_ > 0 ? max() : min();
    }

    constexpr Fixed saturating_sub(Fixed rhs) const {
        const auto m = detail::checked_sub(mantissa_, rhs.mantissa_);
        if (m) {
            return from_raw(*m);
        }
        return rhs.mantissa_ > 0 ? min() : max();
    }

    constexpr Fixed wrapping_add(Fixed rhs) const {
        using U = std::make_unsigned_t<Rep>;
        return from_raw(static_cast<Rep>(static_cast<U>(mantissa_) + static_cast<U>(rhs.mantissa_)));
    }

    constexpr Fixed wrapping_sub(Fixed rhs) const {
        using U = std::make_unsigned_t<Rep>;
        return from_raw(static_cast<Rep>(static_cast<U>(mantissa_) - static_cast<U>(rhs.mantissa_)));
    }

    constexpr std::optional<Fixed> checked_mul_int(std::int64_t n) const {
        const auto m = detail::checked_mul<int128>(mantissa_traits<Rep>::to_i128(mantissa_),
                                                   static_cast<int128>(n));
        if (!m) {
            return std::nullopt;
        }
        return narrow(*m);
    }

    constexpr std::optional<Fixed> checked_div_int_round(std::int64_t n, Round mode) const {
        if (n == 0) {
            return std::nullopt;
        }
        return narrow(div_round(mantissa_traits<Rep>::to_i128(mantissa_), n, mode));
    }

    constexpr std::optional<Fixed> checked_div_int(std::int64_t n) const {
        return checked_div_int_round(n, Round::HalfEven);
    }

    template <unsigned To>
    constexpr std::optional<Fixed<To, Unit, Rep>> checked_rescale_round(Round mode) const {
        const int128 m = mantissa_traits<Rep>::to_i128(mantissa_);
        int128 rescaled = 0;
        if constexpr (To >= Scale) {
            const auto up = detail::checked_mul<int128>(m, pow10(To - Scale));
            if (!up) {
                return std::nullopt;
            }
            rescaled = *up;
        } else {
            rescaled = div_round(m, pow10(Scale - To), mode);
        }
        const auto narrowed = mantissa_traits<Rep>::from_i128(rescaled);
        if (!narrowed) {
            return std::nullopt;
        }
        return Fixed<To, Unit, Rep>::from_raw(*narrowed);
    }

    template <unsigned To>
    constexpr std::optional<Fixed<To, Unit, Rep>> checked_rescale() const {
        return checked_rescale_round<To>(Round::HalfEven);
    }

    constexpr std::optional<Fixed> checked_mul_round(Fixed rhs, Round mode) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        const int128 product = static_cast<int128>(mantissa_) * rhs.mantissa_;
        return narrow(div_round(product, pow10(Scale), mode));
    }

    constexpr std::optional<Fixed> checked_mul(Fixed rhs) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        return checked_mul_round(rhs, Round::HalfEven);
    }

    constexpr Fixed mul(Fixed rhs) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        const auto v = checked_mul(rhs);
        if (!v) {
            throw std::overflow_error("fixed-decimal: mul overflow");
        }
        return *v;
    }

    constexpr std::optional<Fixed> checked_div_round(Fixed rhs, Round mode) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        if (rhs.mantissa_ == 0) {
            return std::nullopt;
        }
        const auto num = detail::checked_mul<int128>(mantissa_, pow10(Scale));
        if (!num) {
            return std::nullopt;
        }
        return narrow(div_round(*num, rhs.mantissa_, mode));
    }

    constexpr std::optional<Fixed> checked_div(Fixed rhs) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        return checked_div_round(rhs, Round::HalfEven);
    }

    constexpr Fixed div(Fixed rhs) const
        requires std::same_as<Unit, PlainTag> && std::same_as<Rep, std::int64_t>
    {
        const auto v = checked_div(rhs);
        if (!v) {
            throw std::overflow_error("fixed-decimal: div overflow or div by zero");
        }
        return *v;
    }

    static constexpr std::expected<Fixed, ParseError> from_string(std::string_view s) {
        return parse(s, Round::HalfEven, false);
    }

    static constexpr std::expected<Fixed, ParseError> from_string_rounded(std::string_view s,
                                                                          Round mode) {
        return parse(s, mode, false);
    }

    static constexpr std::expected<Fixed, ParseError> from_string_exact(std::string_view s) {
        return parse(s, Round::HalfEven, true);
    }

    std::string to_string() const {
        const int128 m = mantissa_traits<Rep>::to_i128(mantissa_);
        uint128 magnitude = detail::unsigned_abs(m);
        const uint128 factor = static_cast<uint128>(pow10(Scale));
        std::string out;
        if (m < 0) {
            out.push_back('-');
        }
        out += format_u128(magnitude / factor);
        if constexpr (Scale > 0) {
            out.push_back('.');
            char frac[Scale];
            uint128 rem = magnitude % factor;
            for (unsigned i = Scale; i > 0; --i) {
                frac[i - 1] = static_cast<char>('0' + static_cast<unsigned>(rem % 10));
                rem /= 10;
            }
            out.append(frac, Scale);
        }
        return out;
    }

    constexpr std::optional<Fixed> round_to_tick_round(Fixed tick, Round mode) const {
        const int128 t = mantissa_traits<Rep>::to_i128(tick.mantissa_);
        if (t <= 0) {
            return std::nullopt;
        }
        const int128 q = div_round(mantissa_traits<Rep>::to_i128(mantissa_), t, mode);
        const auto scaled = detail::checked_mul<int128>(q, t);
        if (!scaled) {
            return std::nullopt;
        }
        return narrow(*scaled);
    }

    constexpr std::optional<Fixed> round_to_tick(Fixed tick) const {
        return round_to_tick_round(tick, Round::HalfEven);
    }

    constexpr std::optional<Fixed> floor_to_tick(Fixed tick) const {
        return round_to_tick_round(tick, Round::Floor);
    }

    constexpr std::optional<Fixed> ceil_to_tick(Fixed tick) const {
        return round_to_tick_round(tick, Round::Ceil);
    }

    constexpr bool is_on_tick(Fixed tick) const {
        const int128 t = mantissa_traits<Rep>::to_i128(tick.mantissa_);
        return t > 0 && mantissa_traits<Rep>::to_i128(mantissa_) % t == 0;
    }

    constexpr std::optional<Fixed> next_tick(Fixed tick) const {
        return step_tick(tick, Round::Floor, 1);
    }

    constexpr std::optional<Fixed> prev_tick(Fixed tick) const {
        return step_tick(tick, Round::Ceil, -1);
    }

    constexpr std::optional<std::int64_t> ticks_between(Fixed other, Fixed tick) const {
        const int128 t = mantissa_traits<Rep>::to_i128(tick.mantissa_);
        if (t <= 0) {
            return std::nullopt;
        }
        const int128 delta =
            mantissa_traits<Rep>::to_i128(other.mantissa_) - mantissa_traits<Rep>::to_i128(mantissa_);
        if (delta % t != 0) {
            return std::nullopt;
        }
        return mantissa_traits<std::int64_t>::from_i128(delta / t);
    }

    constexpr std::optional<std::int64_t> ticks_between_trunc(Fixed other, Fixed tick) const {
        const int128 t = mantissa_traits<Rep>::to_i128(tick.mantissa_);
        if (t <= 0) {
            return std::nullopt;
        }
        const int128 delta =
            mantissa_traits<Rep>::to_i128(other.mantissa_) - mantissa_traits<Rep>::to_i128(mantissa_);
        return mantissa_traits<std::int64_t>::from_i128(delta / t);
    }

    Fixed<9, NotionalTag, int128> mul_qty_round(Fixed<9, QtyTag, std::int64_t> qty,
                                                Round mode) const
        requires std::same_as<Unit, PriceTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>;

    Fixed<9, NotionalTag, int128> mul_qty(Fixed<9, QtyTag, std::int64_t> qty) const
        requires std::same_as<Unit, PriceTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>;

    Fixed<9, NotionalTag, int128> mul_price_round(Fixed<9, PriceTag, std::int64_t> price,
                                                  Round mode) const
        requires std::same_as<Unit, QtyTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>;

    Fixed<9, NotionalTag, int128> mul_price(Fixed<9, PriceTag, std::int64_t> price) const
        requires std::same_as<Unit, QtyTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>;

    std::optional<Fixed<9, QtyTag, std::int64_t>> checked_div_price_round(
        Fixed<9, PriceTag, std::int64_t> price, Round mode) const
        requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>;

    std::optional<Fixed<9, QtyTag, std::int64_t>> checked_div_price(
        Fixed<9, PriceTag, std::int64_t> price) const
        requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>;

    std::optional<Fixed<9, PriceTag, std::int64_t>> checked_div_qty_round(
        Fixed<9, QtyTag, std::int64_t> qty, Round mode) const
        requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>;

    std::optional<Fixed<9, PriceTag, std::int64_t>> checked_div_qty(
        Fixed<9, QtyTag, std::int64_t> qty) const
        requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>;

    friend constexpr bool operator==(Fixed a, Fixed b) { return a.mantissa_ == b.mantissa_; }
    friend constexpr auto operator<=>(Fixed a, Fixed b) { return a.mantissa_ <=> b.mantissa_; }

    friend constexpr Fixed operator+(Fixed a, Fixed b) {
        const auto v = a.checked_add(b);
        if (!v) {
            throw std::overflow_error("fixed-decimal: add overflow");
        }
        return *v;
    }

    friend constexpr Fixed operator-(Fixed a, Fixed b) {
        const auto v = a.checked_sub(b);
        if (!v) {
            throw std::overflow_error("fixed-decimal: sub overflow");
        }
        return *v;
    }

    friend constexpr Fixed operator-(Fixed a) {
        const auto v = a.checked_neg();
        if (!v) {
            throw std::overflow_error("fixed-decimal: neg overflow");
        }
        return *v;
    }

    constexpr Fixed& operator+=(Fixed rhs) { return *this = *this + rhs; }
    constexpr Fixed& operator-=(Fixed rhs) { return *this = *this - rhs; }

    friend constexpr Fixed operator*(Fixed a, std::int64_t n) {
        const auto v = a.checked_mul_int(n);
        if (!v) {
            throw std::overflow_error("fixed-decimal: scalar mul overflow");
        }
        return *v;
    }

    friend constexpr Fixed operator/(Fixed a, std::int64_t n) {
        const auto v = a.checked_div_int(n);
        if (!v) {
            throw std::overflow_error("fixed-decimal: scalar div by zero or overflow");
        }
        return *v;
    }

    friend std::ostream& operator<<(std::ostream& os, Fixed v) { return os << v.to_string(); }

private:
    explicit constexpr Fixed(Rep mantissa) : mantissa_(mantissa) {}

    static constexpr std::optional<Fixed> narrow(int128 mantissa) {
        const auto narrowed = mantissa_traits<Rep>::from_i128(mantissa);
        if (!narrowed) {
            return std::nullopt;
        }
        return from_raw(*narrowed);
    }

    constexpr std::optional<Fixed> step_tick(Fixed tick, Round mode, int direction) const {
        const int128 t = mantissa_traits<Rep>::to_i128(tick.mantissa_);
        if (t <= 0) {
            return std::nullopt;
        }
        const int128 q = div_round(mantissa_traits<Rep>::to_i128(mantissa_), t, mode);
        const auto stepped = detail::checked_add<int128>(q, direction);
        if (!stepped) {
            return std::nullopt;
        }
        const auto scaled = detail::checked_mul<int128>(*stepped, t);
        if (!scaled) {
            return std::nullopt;
        }
        return narrow(*scaled);
    }

    static constexpr std::expected<Fixed, ParseError> parse(std::string_view s, Round mode,
                                                            bool exact) {
        if (s.empty()) {
            return std::unexpected(ParseError::Empty);
        }
        std::size_t i = 0;
        bool neg = false;
        if (s[0] == '+') {
            i = 1;
        } else if (s[0] == '-') {
            i = 1;
            neg = true;
        }

        int128 digits = 0;
        std::size_t int_digits = 0;
        while (i < s.size() && s[i] >= '0' && s[i] <= '9') {
            const auto shifted = detail::checked_mul<int128>(digits, 10);
            if (!shifted) {
                return std::unexpected(ParseError::Overflow);
            }
            const auto next = detail::checked_add<int128>(*shifted, s[i] - '0');
            if (!next) {
                return std::unexpected(ParseError::Overflow);
            }
            digits = *next;
            ++int_digits;
            ++i;
        }

        std::size_t frac_digits = 0;
        if (i < s.size() && s[i] == '.') {
            ++i;
            const std::size_t frac_start = i;
            while (i < s.size() && s[i] >= '0' && s[i] <= '9') {
                const auto shifted = detail::checked_mul<int128>(digits, 10);
                if (!shifted) {
                    return std::unexpected(ParseError::Overflow);
                }
                const auto next = detail::checked_add<int128>(*shifted, s[i] - '0');
                if (!next) {
                    return std::unexpected(ParseError::Overflow);
                }
                digits = *next;
                ++frac_digits;
                ++i;
            }
            if (i == frac_start) {
                return std::unexpected(ParseError::InvalidChar);
            }
        }

        if (i != s.size() || int_digits == 0) {
            return std::unexpected(ParseError::InvalidChar);
        }

        const int128 signed_digits = neg ? -digits : digits;
        int128 mantissa = 0;
        if (frac_digits <= Scale) {
            const auto scaled =
                detail::checked_mul<int128>(signed_digits, pow10(Scale - static_cast<unsigned>(frac_digits)));
            if (!scaled) {
                return std::unexpected(ParseError::Overflow);
            }
            mantissa = *scaled;
        } else {
            const int128 divisor = pow10(static_cast<unsigned>(frac_digits) - Scale);
            if (exact && digits % divisor != 0) {
                return std::unexpected(ParseError::TooManyDigits);
            }
            mantissa = div_round(signed_digits, divisor, mode);
        }

        const auto narrowed = narrow(mantissa);
        if (!narrowed) {
            return std::unexpected(ParseError::Overflow);
        }
        return *narrowed;
    }

    static std::string format_u128(uint128 v) {
        if (v == 0) {
            return "0";
        }
        char buf[40];
        std::size_t idx = sizeof(buf);
        while (v > 0) {
            buf[--idx] = static_cast<char>('0' + static_cast<unsigned>(v % 10));
            v /= 10;
        }
        return std::string(buf + idx, sizeof(buf) - idx);
    }

    Rep mantissa_ = 0;
};

using Price = Fixed<9, PriceTag, std::int64_t>;
using Qty = Fixed<9, QtyTag, std::int64_t>;
using Notional = Fixed<9, NotionalTag, int128>;
template <unsigned Scale>
using Decimal = Fixed<Scale, PlainTag, std::int64_t>;
using Price4 = Fixed<4, PriceTag, std::int64_t>;
using Qty4 = Fixed<4, QtyTag, std::int64_t>;

namespace detail {

inline Notional price_times_qty(std::int64_t price_m, std::int64_t qty_m, Round mode) {
    const int128 product = static_cast<int128>(price_m) * qty_m;
    return Notional::from_raw(div_round(product, pow10(9), mode));
}

}  // namespace detail

template <unsigned Scale, typename Unit, typename Rep>
Fixed<9, NotionalTag, int128> Fixed<Scale, Unit, Rep>::mul_qty_round(
    Fixed<9, QtyTag, std::int64_t> qty, Round mode) const
    requires std::same_as<Unit, PriceTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>
{
    return detail::price_times_qty(mantissa_, qty.raw(), mode);
}

template <unsigned Scale, typename Unit, typename Rep>
Fixed<9, NotionalTag, int128> Fixed<Scale, Unit, Rep>::mul_qty(
    Fixed<9, QtyTag, std::int64_t> qty) const
    requires std::same_as<Unit, PriceTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>
{
    return mul_qty_round(qty, Round::HalfEven);
}

template <unsigned Scale, typename Unit, typename Rep>
Fixed<9, NotionalTag, int128> Fixed<Scale, Unit, Rep>::mul_price_round(
    Fixed<9, PriceTag, std::int64_t> price, Round mode) const
    requires std::same_as<Unit, QtyTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>
{
    return detail::price_times_qty(price.raw(), mantissa_, mode);
}

template <unsigned Scale, typename Unit, typename Rep>
Fixed<9, NotionalTag, int128> Fixed<Scale, Unit, Rep>::mul_price(
    Fixed<9, PriceTag, std::int64_t> price) const
    requires std::same_as<Unit, QtyTag> && (Scale == 9) && std::same_as<Rep, std::int64_t>
{
    return mul_price_round(price, Round::HalfEven);
}

template <unsigned Scale, typename Unit, typename Rep>
std::optional<Fixed<9, QtyTag, std::int64_t>> Fixed<Scale, Unit, Rep>::checked_div_price_round(
    Fixed<9, PriceTag, std::int64_t> price, Round mode) const
    requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>
{
    if (price.raw() == 0) {
        return std::nullopt;
    }
    const auto num = detail::checked_mul<int128>(mantissa_, pow10(9));
    if (!num) {
        return std::nullopt;
    }
    const auto m = mantissa_traits<std::int64_t>::from_i128(div_round(*num, price.raw(), mode));
    if (!m) {
        return std::nullopt;
    }
    return Fixed<9, QtyTag, std::int64_t>::from_raw(*m);
}

template <unsigned Scale, typename Unit, typename Rep>
std::optional<Fixed<9, QtyTag, std::int64_t>> Fixed<Scale, Unit, Rep>::checked_div_price(
    Fixed<9, PriceTag, std::int64_t> price) const
    requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>
{
    return checked_div_price_round(price, Round::HalfEven);
}

template <unsigned Scale, typename Unit, typename Rep>
std::optional<Fixed<9, PriceTag, std::int64_t>> Fixed<Scale, Unit, Rep>::checked_div_qty_round(
    Fixed<9, QtyTag, std::int64_t> qty, Round mode) const
    requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>
{
    if (qty.raw() == 0) {
        return std::nullopt;
    }
    const auto num = detail::checked_mul<int128>(mantissa_, pow10(9));
    if (!num) {
        return std::nullopt;
    }
    const auto m = mantissa_traits<std::int64_t>::from_i128(div_round(*num, qty.raw(), mode));
    if (!m) {
        return std::nullopt;
    }
    return Fixed<9, PriceTag, std::int64_t>::from_raw(*m);
}

template <unsigned Scale, typename Unit, typename Rep>
std::optional<Fixed<9, PriceTag, std::int64_t>> Fixed<Scale, Unit, Rep>::checked_div_qty(
    Fixed<9, QtyTag, std::int64_t> qty) const
    requires std::same_as<Unit, NotionalTag> && (Scale == 9) && std::same_as<Rep, int128>
{
    return checked_div_qty_round(qty, Round::HalfEven);
}

}  // namespace fixed_decimal

template <unsigned Scale, typename Unit, typename Rep>
struct std::hash<fixed_decimal::Fixed<Scale, Unit, Rep>> {
    std::size_t operator()(fixed_decimal::Fixed<Scale, Unit, Rep> v) const noexcept {
        const auto m = fixed_decimal::mantissa_traits<Rep>::to_i128(v.raw());
        const auto lo = static_cast<std::uint64_t>(static_cast<fixed_decimal::uint128>(m));
        const auto hi = static_cast<std::uint64_t>(static_cast<fixed_decimal::uint128>(m) >> 64);
        return std::hash<std::uint64_t>{}(lo ^ (hi * 0x9e3779b97f4a7c15ULL));
    }
};
