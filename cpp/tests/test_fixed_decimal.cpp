#include <fixed_decimal.hpp>

#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

using namespace fixed_decimal;

static int failures = 0;

#define CHECK(cond)                                                          \
    do {                                                                     \
        if (!(cond)) {                                                       \
            ++failures;                                                      \
            std::fprintf(stderr, "FAIL %s:%d  %s\n", __FILE__, __LINE__, #cond); \
        }                                                                    \
    } while (0)

template <typename A, typename B>
concept addable = requires(A a, B b) { a + b; };
template <typename A, typename B>
concept ordered_with = requires(A a, B b) { a < b; };
template <typename T>
concept self_multipliable = requires(T a) { a.checked_mul(a); };
template <typename P, typename Q>
concept has_mul_qty = requires(P p, Q q) { p.mul_qty(q); };

static_assert(!addable<Price, Qty>);
static_assert(!addable<Price, Notional>);
static_assert(!addable<Price4, Price>);
static_assert(addable<Price, Price>);
static_assert(!ordered_with<Price, Qty>);
static_assert(!self_multipliable<Price>);
static_assert(self_multipliable<Decimal<9>>);
static_assert(has_mul_qty<Price, Qty>);
static_assert(!has_mul_qty<Qty, Qty>);
static_assert(sizeof(Price) == 8 && sizeof(Qty) == 8 && sizeof(Notional) == 16);

static void div_round_known_answers() {
    CHECK(div_round(5, 10, Round::HalfEven) == 0);
    CHECK(div_round(15, 10, Round::HalfEven) == 2);
    CHECK(div_round(25, 10, Round::HalfEven) == 2);
    CHECK(div_round(35, 10, Round::HalfEven) == 4);
    CHECK(div_round(-5, 10, Round::HalfEven) == 0);
    CHECK(div_round(-15, 10, Round::HalfEven) == -2);
    CHECK(div_round(-25, 10, Round::HalfEven) == -2);
    CHECK(div_round(-35, 10, Round::HalfEven) == -4);
    CHECK(div_round(25, 10, Round::HalfUp) == 3);
    CHECK(div_round(-25, 10, Round::HalfUp) == -3);
    CHECK(div_round(25, 10, Round::HalfDown) == 2);
    CHECK(div_round(29, 10, Round::TowardZero) == 2);
    CHECK(div_round(-29, 10, Round::TowardZero) == -2);
    CHECK(div_round(21, 10, Round::AwayFromZero) == 3);
    CHECK(div_round(-21, 10, Round::AwayFromZero) == -3);
    CHECK(div_round(-1, 10, Round::Floor) == -1);
    CHECK(div_round(1, 10, Round::Floor) == 0);
    CHECK(div_round(1, 10, Round::Ceil) == 1);
    CHECK(div_round(-1, 10, Round::Ceil) == 0);
}

static void construction_and_accessors() {
    const auto p = Price::from_string("61278.01").value();
    CHECK(p.raw() == 61278010000000LL);
    CHECK(Price::from_int(2).raw() == 2000000000LL);
    CHECK(Price::one().raw() == 1000000000LL);
    CHECK(Price::zero().is_zero());
    CHECK(Price::try_from_parts(2, 500000000).value().raw() == 2500000000LL);
    CHECK(Price::try_from_parts(-2, 500000000).value().raw() == -2500000000LL);
    CHECK(!Price::try_from_parts(0, 1000000000).has_value());
    CHECK(!Price::try_from_parts(0, -1).has_value());
    CHECK(Decimal<0>::from_int(7).raw() == 7);
}

static void arithmetic_and_overflow() {
    const auto a = Qty::from_string("1.5").value();
    const auto b = Qty::from_string("2.25").value();
    CHECK((a + b).to_string() == "3.750000000");
    CHECK((b - a).to_string() == "0.750000000");
    CHECK((-a).raw() == -1500000000LL);
    CHECK((a * 3).to_string() == "4.500000000");
    CHECK((a / 2).to_string() == "0.750000000");
    CHECK(!Qty::max().checked_add(Qty::one()).has_value());
    CHECK(!Qty::min().checked_neg().has_value());
    CHECK(Qty::max().saturating_add(Qty::one()) == Qty::max());
    CHECK(Qty::min().saturating_sub(Qty::one()) == Qty::min());
    bool threw = false;
    try {
        (void)(Qty::max() + Qty::one());
    } catch (const std::overflow_error&) {
        threw = true;
    }
    CHECK(threw);
    CHECK(!a.checked_div_int(0).has_value());
}

static void decimal_mul_div() {
    const auto x = Decimal<9>::from_string("1.5").value();
    const auto y = Decimal<9>::from_string("2.5").value();
    CHECK(x.checked_mul(y).value().to_string() == "3.750000000");
    CHECK(y.checked_div(x).value().to_string() == "1.666666667");
    CHECK(x.checked_div_round(y, Round::Floor).value().to_string() == "0.600000000");
    CHECK(!x.checked_div(Decimal<9>::zero()).has_value());
}

static void cross_unit_ops() {
    const auto price = Price::from_string("61278.01").value();
    const auto qty = Qty::from_string("0.5").value();
    const auto notional = price.mul_qty(qty);
    CHECK(notional.to_string() == "30639.005000000");
    CHECK(qty.mul_price(price).raw() == notional.raw());
    CHECK(notional.checked_div_price(price).value() == qty);
    CHECK(notional.checked_div_qty(qty).value() == price);
    CHECK(!notional.checked_div_price(Price::zero()).has_value());
}

static void rescale_round_trip() {
    const auto p9 = Price::from_string("123.456789").value();
    const auto p4 = p9.checked_rescale<4>().value();
    CHECK(p4.to_string() == "123.4568");
    const auto back = p4.checked_rescale<9>().value();
    CHECK(back.to_string() == "123.456800000");
    CHECK(!Price::max().checked_rescale<18>().has_value());
}

static void parse_and_format() {
    CHECK(Price::from_string("0").value().to_string() == "0.000000000");
    CHECK(Price::from_string("+1.5").value().raw() == 1500000000LL);
    CHECK(Price::from_string("-0").value().raw() == 0);
    CHECK(Price::from_string("00042.10").value().to_string() == "42.100000000");
    CHECK(Price::from_string("0.1234567891").value().raw() == 123456789LL);
    CHECK(Price::from_string_rounded("0.1234567895", Round::TowardZero).value().raw() ==
          123456789LL);
    CHECK(Price::from_string_exact("0.123456789").has_value());
    CHECK(Price::from_string_exact("0.1234567891").error() == ParseError::TooManyDigits);
    CHECK(Price::from_string_exact("0.1234567890").has_value());
    CHECK(Price::from_string("").error() == ParseError::Empty);
    for (const char* bad : {".", "-", "+", "1.", ".5", "1..2", "1.2.3", " 1", "1 ", "1,5", "1e5"}) {
        CHECK(!Price::from_string(bad).has_value());
    }
    CHECK(Price::from_string("99999999999999999999").error() == ParseError::Overflow);
    const auto v = Decimal<2>::from_string("7.005").value();
    CHECK(v.raw() == 700);
    CHECK(Notional::from_string("170000000000.000000001").value().raw() ==
          static_cast<int128>(170000000000LL) * 1000000000 + 1);
}

static void tick_arithmetic() {
    const auto tick = Price::from_string("0.05").value();
    const auto p = Price::from_string("10.12").value();
    CHECK(p.round_to_tick(tick).value().to_string() == "10.100000000");
    CHECK(p.floor_to_tick(tick).value().to_string() == "10.100000000");
    CHECK(p.ceil_to_tick(tick).value().to_string() == "10.150000000");
    CHECK(!p.is_on_tick(tick));
    const auto on = Price::from_string("10.10").value();
    CHECK(on.is_on_tick(tick));
    CHECK(on.next_tick(tick).value().to_string() == "10.150000000");
    CHECK(on.prev_tick(tick).value().to_string() == "10.050000000");
    CHECK(on.ticks_between(Price::from_string("10.30").value(), tick).value() == 4);
    CHECK(!on.ticks_between(p, tick).has_value());
    CHECK(on.ticks_between_trunc(p, tick).value() == 0);
    CHECK(!p.round_to_tick(Price::zero()).has_value());
}

struct Row {
    std::string op, mode, sa, sb, a, b, c, expect;
};

static Round parse_mode(const std::string& s) {
    if (s == "HalfEven") return Round::HalfEven;
    if (s == "HalfUp") return Round::HalfUp;
    if (s == "HalfDown") return Round::HalfDown;
    if (s == "TowardZero") return Round::TowardZero;
    if (s == "AwayFromZero") return Round::AwayFromZero;
    if (s == "Floor") return Round::Floor;
    return Round::Ceil;
}

static int128 parse_i128(const std::string& s) {
    int128 v = 0;
    bool neg = false;
    std::size_t i = 0;
    if (s[0] == '-') {
        neg = true;
        i = 1;
    }
    for (; i < s.size(); ++i) {
        v = v * 10 + (s[i] - '0');
    }
    return neg ? -v : v;
}

static std::string format_i128(int128 v) {
    return Fixed<0, PlainTag, int128>::from_raw(v).to_string();
}

template <typename F>
static std::string opt_mantissa(std::optional<F> v) {
    if (!v) {
        return "NONE";
    }
    return format_i128(mantissa_traits<typename F::rep>::to_i128(v->raw()));
}

static std::string parse_outcome(const std::expected<Decimal<9>, ParseError>& r) {
    if (r.has_value()) {
        return format_i128(r->raw());
    }
    switch (r.error()) {
        case ParseError::Empty: return "E:Empty";
        case ParseError::InvalidChar: return "E:InvalidChar";
        case ParseError::Overflow: return "E:Overflow";
        case ParseError::TooManyDigits: return "E:TooManyDigits";
    }
    return "E:?";
}

static void conformance(const char* path) {
    std::ifstream file(path);
    if (!file) {
        ++failures;
        std::fprintf(stderr, "FAIL cannot open vectors file %s\n", path);
        return;
    }
    std::string line;
    std::getline(file, line);
    std::size_t rows = 0;
    while (std::getline(file, line)) {
        if (line.empty()) {
            continue;
        }
        std::stringstream ss(line);
        Row r;
        std::getline(ss, r.op, ',');
        std::getline(ss, r.mode, ',');
        std::getline(ss, r.sa, ',');
        std::getline(ss, r.sb, ',');
        std::getline(ss, r.a, ',');
        std::getline(ss, r.b, ',');
        std::getline(ss, r.c, ',');
        std::getline(ss, r.expect, ',');
        ++rows;

        const Round mode = parse_mode(r.mode);
        std::string got;
        if (r.op == "div_round") {
            got = format_i128(div_round(parse_i128(r.a), parse_i128(r.b), mode));
        } else if (r.op == "parse") {
            got = parse_outcome(r.b == "1" ? Decimal<9>::from_string_exact(r.a)
                                           : Decimal<9>::from_string_rounded(r.a, mode));
        } else if (r.op == "format") {
            got = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.a))).to_string();
        } else if (r.op == "add") {
            const auto a = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto b = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(a.checked_add(b));
        } else if (r.op == "mul") {
            const auto a = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto b = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(a.checked_mul_round(b, mode));
        } else if (r.op == "div") {
            const auto a = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto b = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(a.checked_div_round(b, mode));
        } else if (r.op == "rescale_9_to_4") {
            const auto a = Decimal<9>::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            got = opt_mantissa(a.checked_rescale_round<4>(mode));
        } else if (r.op == "rescale_4_to_9") {
            const auto a = Decimal<4>::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            got = opt_mantissa(a.checked_rescale_round<9>(mode));
        } else if (r.op == "mul_qty") {
            const auto p = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto q = Qty::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = format_i128(p.mul_qty_round(q, mode).raw());
        } else if (r.op == "div_price") {
            const auto n = Notional::from_raw(parse_i128(r.a));
            const auto p = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(n.checked_div_price_round(p, mode));
        } else if (r.op == "div_qty") {
            const auto n = Notional::from_raw(parse_i128(r.a));
            const auto q = Qty::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(n.checked_div_qty_round(q, mode));
        } else if (r.op == "round_tick") {
            const auto a = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto t = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            got = opt_mantissa(a.round_to_tick_round(t, mode));
        } else if (r.op == "ticks_between") {
            const auto a = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.a)));
            const auto b = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.b)));
            const auto t = Price::from_raw(static_cast<std::int64_t>(parse_i128(r.c)));
            const auto d = a.ticks_between(b, t);
            got = d ? format_i128(*d) : "NONE";
        } else {
            ++failures;
            std::fprintf(stderr, "FAIL unknown op %s\n", r.op.c_str());
            continue;
        }
        if (got != r.expect) {
            ++failures;
            std::fprintf(stderr, "FAIL vectors row %zu (%s): got %s want %s\n", rows,
                         r.op.c_str(), got.c_str(), r.expect.c_str());
        }
    }
    std::printf("conformance: %zu vectors\n", rows);
}

int main(int argc, char** argv) {
    div_round_known_answers();
    construction_and_accessors();
    arithmetic_and_overflow();
    decimal_mul_div();
    cross_unit_ops();
    rescale_round_trip();
    parse_and_format();
    tick_arithmetic();
    if (argc > 1) {
        conformance(argv[1]);
    }
    if (failures == 0) {
        std::printf("all tests passed\n");
        return 0;
    }
    std::fprintf(stderr, "%d failure(s)\n", failures);
    return 1;
}
