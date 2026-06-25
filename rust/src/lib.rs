#![cfg_attr(not(test), no_std)]

use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Round {
    HalfEven,
    HalfUp,
    HalfDown,
    TowardZero,
    AwayFromZero,
    Floor,
    Ceil,
}

pub fn div_round(num: i128, den: i128, mode: Round) -> i128 {
    debug_assert!(den != 0, "div_round: division by zero");
    let sign = num.signum() * den.signum();
    let n = num.unsigned_abs();
    let d = den.unsigned_abs();
    let q = n / d;
    let r = n % d;
    let half = d - r; // 2r vs d  <=>  r vs (d - r)
    let away = match mode {
        Round::HalfEven => r > half || (r == half && q & 1 == 1),
        Round::HalfUp => r >= half,
        Round::HalfDown => r > half,
        Round::TowardZero => false,
        Round::AwayFromZero => r > 0,
        Round::Floor => sign < 0 && r > 0,
        Round::Ceil => sign > 0 && r > 0,
    };
    let magnitude = q + u128::from(away);
    sign * magnitude as i128
}

const fn make_pow10() -> [i128; 39] {
    let mut table = [1i128; 39];
    let mut i = 1;
    while i < 39 {
        table[i] = table[i - 1] * 10;
        i += 1;
    }
    table
}

const POW10: [i128; 39] = make_pow10();

pub trait Mantissa: Copy + Eq + Ord + core::hash::Hash + Default {
    const MAX_SCALE: u32;
    const ZERO: Self;
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    fn to_i128(self) -> i128;
    fn from_i128(value: i128) -> Option<Self>;
    fn saturating_add(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
}

impl Mantissa for i64 {
    const MAX_SCALE: u32 = 18;
    const ZERO: Self = 0;
    const MIN_VALUE: Self = i64::MIN;
    const MAX_VALUE: Self = i64::MAX;

    #[inline]
    fn to_i128(self) -> i128 {
        self as i128
    }

    #[inline]
    fn from_i128(value: i128) -> Option<Self> {
        if value >= i64::MIN as i128 && value <= i64::MAX as i128 {
            Some(value as i64)
        } else {
            None
        }
    }

    #[inline]
    fn saturating_add(self, rhs: Self) -> Self {
        i64::saturating_add(self, rhs)
    }
    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        i64::saturating_sub(self, rhs)
    }
    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        i64::wrapping_add(self, rhs)
    }
    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        i64::wrapping_sub(self, rhs)
    }
}

impl Mantissa for i128 {
    const MAX_SCALE: u32 = 38;
    const ZERO: Self = 0;
    const MIN_VALUE: Self = i128::MIN;
    const MAX_VALUE: Self = i128::MAX;

    #[inline]
    fn to_i128(self) -> i128 {
        self
    }

    #[inline]
    fn from_i128(value: i128) -> Option<Self> {
        Some(value)
    }

    #[inline]
    fn saturating_add(self, rhs: Self) -> Self {
        i128::saturating_add(self, rhs)
    }
    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        i128::saturating_sub(self, rhs)
    }
    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        i128::wrapping_add(self, rhs)
    }
    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        i128::wrapping_sub(self, rhs)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct PriceTag;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct QtyTag;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct NotionalTag;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct PlainTag;

#[repr(transparent)]
pub struct Fixed<const SCALE: u32, Unit, Repr: Mantissa = i64> {
    mantissa: Repr,
    _unit: PhantomData<Unit>,
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Clone for Fixed<SCALE, Unit, Repr> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<const SCALE: u32, Unit, Repr: Mantissa> Copy for Fixed<SCALE, Unit, Repr> {}

impl<const SCALE: u32, Unit, Repr: Mantissa> PartialEq for Fixed<SCALE, Unit, Repr> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.mantissa == other.mantissa
    }
}
impl<const SCALE: u32, Unit, Repr: Mantissa> Eq for Fixed<SCALE, Unit, Repr> {}

impl<const SCALE: u32, Unit, Repr: Mantissa> PartialOrd for Fixed<SCALE, Unit, Repr> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<const SCALE: u32, Unit, Repr: Mantissa> Ord for Fixed<SCALE, Unit, Repr> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.mantissa.cmp(&other.mantissa)
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> core::hash::Hash for Fixed<SCALE, Unit, Repr> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.mantissa.hash(state);
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Default for Fixed<SCALE, Unit, Repr> {
    fn default() -> Self {
        Self::from_raw(Repr::ZERO)
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> core::fmt::Debug for Fixed<SCALE, Unit, Repr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Fixed<{SCALE}>({})", self.mantissa.to_i128())
    }
}

pub type Price = Fixed<9, PriceTag, i64>;
pub type Qty = Fixed<9, QtyTag, i64>;
pub type Notional = Fixed<9, NotionalTag, i128>;
pub type Decimal<const SCALE: u32> = Fixed<SCALE, PlainTag, i64>;
pub type Price4 = Fixed<4, PriceTag, i64>;
pub type Qty4 = Fixed<4, QtyTag, i64>;

impl<const SCALE: u32, Unit, Repr: Mantissa> Fixed<SCALE, Unit, Repr> {
    const GUARD: () = assert!(
        SCALE <= Repr::MAX_SCALE,
        "SCALE too large for the backing integer"
    );

    pub const ZERO: Self = Self::from_raw(Repr::ZERO);
    pub const MIN: Self = Self::from_raw(Repr::MIN_VALUE);
    pub const MAX: Self = Self::from_raw(Repr::MAX_VALUE);

    #[inline]
    pub const fn from_raw(mantissa: Repr) -> Self {
        let () = Self::GUARD;
        Self {
            mantissa,
            _unit: PhantomData,
        }
    }

    #[inline]
    pub const fn raw(self) -> Repr {
        self.mantissa
    }

    #[inline]
    pub fn scale_factor() -> i128 {
        POW10[SCALE as usize]
    }

    pub fn one() -> Self {
        Self::from_int(1)
    }

    pub fn checked_from_int(value: i64) -> Option<Self> {
        let mantissa = (value as i128).checked_mul(POW10[SCALE as usize])?;
        Repr::from_i128(mantissa).map(Self::from_raw)
    }

    pub fn from_int(value: i64) -> Self {
        Self::checked_from_int(value).expect("fixed-decimal: from_int overflow")
    }

    pub fn try_from_parts(whole: i64, frac: i64) -> Option<Self> {
        let factor = POW10[SCALE as usize];
        if frac < 0 || (frac as i128) >= factor {
            return None;
        }
        let scaled = (whole as i128).checked_mul(factor)?;
        let mantissa = if whole < 0 {
            scaled.checked_sub(frac as i128)?
        } else {
            scaled.checked_add(frac as i128)?
        };
        Repr::from_i128(mantissa).map(Self::from_raw)
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.mantissa == Repr::ZERO
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.mantissa
            .to_i128()
            .checked_add(rhs.mantissa.to_i128())
            .and_then(Repr::from_i128)
            .map(Self::from_raw)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.mantissa
            .to_i128()
            .checked_sub(rhs.mantissa.to_i128())
            .and_then(Repr::from_i128)
            .map(Self::from_raw)
    }

    pub fn checked_neg(self) -> Option<Self> {
        self.mantissa
            .to_i128()
            .checked_neg()
            .and_then(Repr::from_i128)
            .map(Self::from_raw)
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self::from_raw(self.mantissa.saturating_add(rhs.mantissa))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self::from_raw(self.mantissa.saturating_sub(rhs.mantissa))
    }

    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self::from_raw(self.mantissa.wrapping_add(rhs.mantissa))
    }

    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self::from_raw(self.mantissa.wrapping_sub(rhs.mantissa))
    }

    pub fn checked_mul_int(self, n: i64) -> Option<Self> {
        self.mantissa
            .to_i128()
            .checked_mul(n as i128)
            .and_then(Repr::from_i128)
            .map(Self::from_raw)
    }

    pub fn checked_div_int_round(self, n: i64, mode: Round) -> Option<Self> {
        if n == 0 {
            return None;
        }
        Repr::from_i128(div_round(self.mantissa.to_i128(), n as i128, mode)).map(Self::from_raw)
    }

    pub fn checked_div_int(self, n: i64) -> Option<Self> {
        self.checked_div_int_round(n, Round::HalfEven)
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Add for Fixed<SCALE, Unit, Repr> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("fixed-decimal: add overflow")
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Sub for Fixed<SCALE, Unit, Repr> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).expect("fixed-decimal: sub overflow")
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Neg for Fixed<SCALE, Unit, Repr> {
    type Output = Self;
    fn neg(self) -> Self {
        self.checked_neg().expect("fixed-decimal: neg overflow")
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> AddAssign for Fixed<SCALE, Unit, Repr> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> SubAssign for Fixed<SCALE, Unit, Repr> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Mul<i64> for Fixed<SCALE, Unit, Repr> {
    type Output = Self;
    fn mul(self, n: i64) -> Self {
        self.checked_mul_int(n)
            .expect("fixed-decimal: scalar mul overflow")
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Div<i64> for Fixed<SCALE, Unit, Repr> {
    type Output = Self;
    fn div(self, n: i64) -> Self {
        self.checked_div_int(n)
            .expect("fixed-decimal: scalar div by zero or overflow")
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> Fixed<SCALE, Unit, Repr> {
    pub fn checked_rescale_round<const TO: u32>(
        self,
        mode: Round,
    ) -> Option<Fixed<TO, Unit, Repr>> {
        let m = self.mantissa.to_i128();
        let rescaled = if TO >= SCALE {
            m.checked_mul(POW10[(TO - SCALE) as usize])?
        } else {
            div_round(m, POW10[(SCALE - TO) as usize], mode)
        };
        Repr::from_i128(rescaled).map(Fixed::<TO, Unit, Repr>::from_raw)
    }

    pub fn checked_rescale<const TO: u32>(self) -> Option<Fixed<TO, Unit, Repr>> {
        self.checked_rescale_round::<TO>(Round::HalfEven)
    }
}

impl<const SCALE: u32> Fixed<SCALE, PlainTag, i64> {
    pub fn checked_mul_round(self, rhs: Self, mode: Round) -> Option<Self> {
        let product = self.mantissa as i128 * rhs.mantissa as i128;
        let m = div_round(product, POW10[SCALE as usize], mode);
        <i64 as Mantissa>::from_i128(m).map(Self::from_raw)
    }

    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        self.checked_mul_round(rhs, Round::HalfEven)
    }

    // No `Mul`/`Div` operators: a value-losing product must name its rounding mode.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).expect("fixed-decimal: mul overflow")
    }

    pub fn checked_div_round(self, rhs: Self, mode: Round) -> Option<Self> {
        if rhs.mantissa == 0 {
            return None;
        }
        let num = (self.mantissa as i128).checked_mul(POW10[SCALE as usize])?;
        let m = div_round(num, rhs.mantissa as i128, mode);
        <i64 as Mantissa>::from_i128(m).map(Self::from_raw)
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        self.checked_div_round(rhs, Round::HalfEven)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(self, rhs: Self) -> Self {
        self.checked_div(rhs)
            .expect("fixed-decimal: div overflow or div by zero")
    }
}

fn price_times_qty(price_m: i64, qty_m: i64, mode: Round) -> Notional {
    // scale 9 * scale 9 = scale 18; rescale once to scale 9. The i64*i64 product
    // fits i128 and notional/1e9 stays well inside i128, so this never overflows.
    let product = price_m as i128 * qty_m as i128;
    Notional::from_raw(div_round(product, POW10[9], mode))
}

impl Price {
    pub fn mul_qty_round(self, qty: Qty, mode: Round) -> Notional {
        price_times_qty(self.mantissa, qty.mantissa, mode)
    }

    pub fn mul_qty(self, qty: Qty) -> Notional {
        self.mul_qty_round(qty, Round::HalfEven)
    }
}

impl Qty {
    pub fn mul_price_round(self, price: Price, mode: Round) -> Notional {
        price_times_qty(price.mantissa, self.mantissa, mode)
    }

    pub fn mul_price(self, price: Price) -> Notional {
        self.mul_price_round(price, Round::HalfEven)
    }
}

impl Notional {
    pub fn checked_div_price_round(self, price: Price, mode: Round) -> Option<Qty> {
        if price.mantissa == 0 {
            return None;
        }
        let num = self.mantissa.checked_mul(POW10[9])?;
        let m = div_round(num, price.mantissa as i128, mode);
        <i64 as Mantissa>::from_i128(m).map(Qty::from_raw)
    }

    pub fn checked_div_price(self, price: Price) -> Option<Qty> {
        self.checked_div_price_round(price, Round::HalfEven)
    }

    pub fn checked_div_qty_round(self, qty: Qty, mode: Round) -> Option<Price> {
        if qty.mantissa == 0 {
            return None;
        }
        let num = self.mantissa.checked_mul(POW10[9])?;
        let m = div_round(num, qty.mantissa as i128, mode);
        <i64 as Mantissa>::from_i128(m).map(Price::from_raw)
    }

    pub fn checked_div_qty(self, qty: Qty) -> Option<Price> {
        self.checked_div_qty_round(qty, Round::HalfEven)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseError {
    Empty,
    InvalidChar,
    Overflow,
    TooManyDigits,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            ParseError::Empty => "empty input",
            ParseError::InvalidChar => "invalid character",
            ParseError::Overflow => "value out of range",
            ParseError::TooManyDigits => "more fraction digits than the scale allows",
        })
    }
}

impl core::error::Error for ParseError {}

impl<const SCALE: u32, Unit, Repr: Mantissa> Fixed<SCALE, Unit, Repr> {
    // Inherent twin of FromStr so callers can parse without importing the trait.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        Self::parse(s, Round::HalfEven, false)
    }

    pub fn from_str_rounded(s: &str, mode: Round) -> Result<Self, ParseError> {
        Self::parse(s, mode, false)
    }

    pub fn from_str_exact(s: &str) -> Result<Self, ParseError> {
        Self::parse(s, Round::HalfEven, true)
    }

    fn parse(s: &str, mode: Round, exact: bool) -> Result<Self, ParseError> {
        let b = s.as_bytes();
        if b.is_empty() {
            return Err(ParseError::Empty);
        }
        let mut i = 0;
        let neg = match b[0] {
            b'+' => {
                i = 1;
                false
            }
            b'-' => {
                i = 1;
                true
            }
            _ => false,
        };

        let mut digits: i128 = 0;
        let mut int_digits = 0usize;
        while i < b.len() && b[i].is_ascii_digit() {
            digits = digits
                .checked_mul(10)
                .and_then(|d| d.checked_add((b[i] - b'0') as i128))
                .ok_or(ParseError::Overflow)?;
            int_digits += 1;
            i += 1;
        }

        let mut frac_digits = 0usize;
        if i < b.len() && b[i] == b'.' {
            i += 1;
            let frac_start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                digits = digits
                    .checked_mul(10)
                    .and_then(|d| d.checked_add((b[i] - b'0') as i128))
                    .ok_or(ParseError::Overflow)?;
                frac_digits += 1;
                i += 1;
            }
            if i == frac_start {
                return Err(ParseError::InvalidChar);
            }
        }

        if i != b.len() || int_digits == 0 {
            return Err(ParseError::InvalidChar);
        }

        let signed = if neg { -digits } else { digits };
        let target = SCALE as usize;
        let mantissa = if frac_digits <= target {
            signed
                .checked_mul(POW10[target - frac_digits])
                .ok_or(ParseError::Overflow)?
        } else {
            let divisor = POW10[frac_digits - target];
            if exact && digits % divisor != 0 {
                return Err(ParseError::TooManyDigits);
            }
            div_round(signed, divisor, mode)
        };

        Repr::from_i128(mantissa)
            .map(Self::from_raw)
            .ok_or(ParseError::Overflow)
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> core::str::FromStr for Fixed<SCALE, Unit, Repr> {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        Self::parse(s, Round::HalfEven, false)
    }
}

impl<const SCALE: u32, Unit, Repr: Mantissa> core::fmt::Display for Fixed<SCALE, Unit, Repr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let m = self.mantissa.to_i128();
        let magnitude = m.unsigned_abs();
        let factor = POW10[SCALE as usize] as u128;
        if m < 0 {
            f.write_str("-")?;
        }
        write!(f, "{}", magnitude / factor)?;
        if SCALE > 0 {
            f.write_str(".")?;
            let mut buf = [b'0'; 39];
            let mut frac = magnitude % factor;
            let mut idx = SCALE as usize;
            while idx > 0 {
                idx -= 1;
                buf[idx] = b'0' + (frac % 10) as u8;
                frac /= 10;
            }
            f.write_str(core::str::from_utf8(&buf[..SCALE as usize]).unwrap())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_is_eight_bytes() {
        assert_eq!(core::mem::size_of::<Price>(), 8);
        assert_eq!(core::mem::size_of::<Qty>(), 8);
        assert_eq!(core::mem::size_of::<Notional>(), 16);
    }

    #[test]
    fn construction_and_raw() {
        let p = Price::from_int(7);
        assert_eq!(p.raw(), 7_000_000_000);
        assert_eq!(Price::ZERO.raw(), 0);
        assert!(Price::ZERO.is_zero());
        assert_eq!(Price::one().raw(), 1_000_000_000);
    }

    #[test]
    fn from_parts() {
        assert_eq!(
            Price::try_from_parts(1, 250_000_000).unwrap().raw(),
            1_250_000_000
        );
        assert_eq!(
            Price::try_from_parts(-1, 250_000_000).unwrap().raw(),
            -1_250_000_000
        );
        assert!(Price::try_from_parts(0, 1_000_000_000).is_none());
        assert!(Price::try_from_parts(0, -1).is_none());
    }

    #[test]
    fn ordering_by_value() {
        assert!(Price::from_int(2) > Price::from_int(1));
        assert_eq!(Price::from_int(3), Price::try_from_parts(3, 0).unwrap());
    }

    #[test]
    fn from_int_overflow_is_checked() {
        assert!(Price::checked_from_int(i64::MAX).is_none());
        assert!(Notional::checked_from_int(1_000_000_000).is_some());
    }

    #[test]
    fn half_even_tie_table() {
        // x.5 -> div_round(2x+1, 2). Anchors from the contract.
        let cases = [
            (1, 0),   // 0.5 -> 0
            (3, 2),   // 1.5 -> 2
            (5, 2),   // 2.5 -> 2
            (7, 4),   // 3.5 -> 4
            (-1, 0),  // -0.5 -> 0
            (-3, -2), // -1.5 -> -2
            (-5, -2), // -2.5 -> -2
            (-7, -4), // -3.5 -> -4
        ];
        for (num, expected) in cases {
            assert_eq!(
                div_round(num, 2, Round::HalfEven),
                expected,
                "HalfEven {num}/2"
            );
        }
    }

    #[test]
    fn rounding_modes_on_ties() {
        assert_eq!(div_round(5, 2, Round::HalfUp), 3); // 2.5 -> 3
        assert_eq!(div_round(-5, 2, Round::HalfUp), -3); // -2.5 -> -3 (away)
        assert_eq!(div_round(5, 2, Round::HalfDown), 2);
        assert_eq!(div_round(5, 2, Round::TowardZero), 2);
        assert_eq!(div_round(-5, 2, Round::TowardZero), -2);
        assert_eq!(div_round(1, 2, Round::AwayFromZero), 1);
        assert_eq!(div_round(-1, 2, Round::AwayFromZero), -1);
        assert_eq!(div_round(-1, 2, Round::Floor), -1); // -0.5 -> -1
        assert_eq!(div_round(1, 2, Round::Floor), 0);
        assert_eq!(div_round(1, 2, Round::Ceil), 1);
        assert_eq!(div_round(-1, 2, Round::Ceil), 0);
    }

    #[test]
    fn add_sub_neg_exact() {
        let a = Price::try_from_parts(1, 250_000_000).unwrap();
        let b = Price::try_from_parts(0, 750_000_000).unwrap();
        assert_eq!((a + b).raw(), 2_000_000_000);
        assert_eq!((a - b).raw(), 500_000_000);
        assert_eq!((-a).raw(), -1_250_000_000);
        let mut c = a;
        c += b;
        assert_eq!(c.raw(), 2_000_000_000);
    }

    #[test]
    fn scalar_mul_div() {
        let p = Price::try_from_parts(1, 250_000_000).unwrap();
        assert_eq!((p * 4).raw(), 5_000_000_000);
        assert_eq!((p / 2).raw(), 625_000_000);
        // scalar division rounds half-even
        assert_eq!((Price::from_raw(5) / 2).raw(), 2);
        assert!(p.checked_div_int(0).is_none());
    }

    #[test]
    fn saturating_and_wrapping() {
        assert_eq!(Price::MAX.saturating_add(Price::from_int(1)), Price::MAX);
        assert_eq!(Price::MIN.saturating_sub(Price::from_int(1)), Price::MIN);
        assert!(Price::MAX.checked_add(Price::from_int(1)).is_none());
    }

    #[test]
    fn decimal_mul_div() {
        let a = Decimal::<2>::try_from_parts(1, 50).unwrap(); // 1.50
        let b = Decimal::<2>::try_from_parts(2, 0).unwrap(); // 2.00
        assert_eq!(a.mul(b).raw(), 300); // 3.00

        // 0.1 * 0.2 == 0.02 exactly at scale 9
        let p = Decimal::<9>::try_from_parts(0, 100_000_000).unwrap();
        let q = Decimal::<9>::try_from_parts(0, 200_000_000).unwrap();
        assert_eq!(p.mul(q).raw(), 20_000_000);

        // 0.5 * 0.5 = 0.25 -> 0.2 at scale 1 (HalfEven on the 2.5 tenths)
        let h = Decimal::<1>::try_from_parts(0, 5).unwrap();
        assert_eq!(h.mul(h).raw(), 2);

        let six = Decimal::<2>::from_int(6);
        let four = Decimal::<2>::from_int(4);
        assert_eq!(six.div(four).raw(), 150); // 1.50
        assert!(six.checked_div(Decimal::<2>::ZERO).is_none());
    }

    #[test]
    fn cross_unit_algebra() {
        let price = Price::try_from_parts(2, 500_000_000).unwrap(); // 2.5
        let qty = Qty::from_int(3); // 3.0
        let notional = price.mul_qty(qty);
        assert_eq!(notional.raw(), 7_500_000_000); // 7.5
        assert_eq!(qty.mul_price(price), notional);

        assert_eq!(notional.checked_div_price(price).unwrap(), qty);
        assert_eq!(notional.checked_div_qty(qty).unwrap(), price);
        assert!(notional.checked_div_price(Price::ZERO).is_none());
    }

    #[test]
    fn rescale_changes_scale() {
        let p = Price::try_from_parts(1, 250_000_000).unwrap(); // 1.25 @ scale 9
        let p2: Fixed<2, PriceTag, i64> = p.checked_rescale::<2>().unwrap();
        assert_eq!(p2.raw(), 125);
        let back: Price = p2.checked_rescale::<9>().unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn parse_accepts_and_normalizes() {
        assert_eq!(Price::from_str("0.1").unwrap().raw(), 100_000_000);
        assert_eq!(Price::from_str("0.2").unwrap().raw(), 200_000_000);
        // 0.1 + 0.2 == 0.3 exactly
        let sum = Price::from_str("0.1").unwrap() + Price::from_str("0.2").unwrap();
        assert_eq!(sum, Price::from_str("0.3").unwrap());

        assert_eq!(Price::from_str("007.50").unwrap().raw(), 7_500_000_000);
        assert_eq!(Price::from_str("1.2500").unwrap().raw(), 1_250_000_000);
        assert_eq!(Price::from_str("+5").unwrap().raw(), 5_000_000_000);
        assert_eq!(Price::from_str("-3.25").unwrap().raw(), -3_250_000_000);
        for z in ["0", "-0", "+0", "-0.0", "0.000"] {
            assert_eq!(Price::from_str(z).unwrap().raw(), 0, "{z}");
        }
    }

    #[test]
    fn parse_rounds_excess_digits_half_even() {
        let two = Decimal::<2>::from_str("1.005").unwrap();
        assert_eq!(two.raw(), 100); // 1.005 -> 1.00
        let two_b = Decimal::<2>::from_str("1.015").unwrap();
        assert_eq!(two_b.raw(), 102); // 1.015 -> 1.02
                                      // exact mode rejects a dropped non-zero digit
        assert_eq!(
            Decimal::<2>::from_str_exact("1.005"),
            Err(ParseError::TooManyDigits)
        );
        assert_eq!(Decimal::<2>::from_str_exact("1.230").unwrap().raw(), 123);
    }

    #[test]
    fn parse_rejects_bad_input() {
        for bad in [
            "", ".", "+", "-", "1.", ".5", " 1.5", "1.5 ", "1,000", "1e9", "1.2.3", "1.2x",
        ] {
            assert!(Price::from_str(bad).is_err(), "should reject {bad:?}");
        }
        assert_eq!(Price::from_str(""), Err(ParseError::Empty));
        assert_eq!(Price::from_str("99999999999.0"), Err(ParseError::Overflow));
    }

    #[test]
    fn format_is_fixed_width_and_round_trips() {
        assert_eq!(
            Price::try_from_parts(1, 250_000_000).unwrap().to_string(),
            "1.250000000"
        );
        assert_eq!(Price::ZERO.to_string(), "0.000000000");
        assert_eq!(
            Price::try_from_parts(-1, 250_000_000).unwrap().to_string(),
            "-1.250000000"
        );
        // never negative zero
        assert!(!Price::ZERO.to_string().starts_with('-'));

        for raw in [
            0i64,
            1,
            -1,
            5,
            -5,
            1_250_000_000,
            -9_223_372_036,
            i64::MAX,
            i64::MIN + 1,
        ] {
            let x = Price::from_raw(raw);
            assert_eq!(
                Price::from_str(&x.to_string()).unwrap(),
                x,
                "round-trip {raw}"
            );
        }
    }

    #[test]
    fn mul_matches_i128_oracle() {
        let xs = [1i64, 5, 123, -7, 1_000_000_000, -999_999_999, 250_000_000];
        for &a in &xs {
            for &b in &xs {
                let da = Decimal::<9>::from_raw(a);
                let db = Decimal::<9>::from_raw(b);
                let got = da.mul(db).raw() as i128;
                let oracle = div_round(a as i128 * b as i128, POW10[9], Round::HalfEven);
                assert_eq!(got, oracle, "{a} * {b}");
            }
        }
    }
}
