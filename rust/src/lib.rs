#![cfg_attr(not(test), no_std)]

use core::marker::PhantomData;

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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Fixed<const SCALE: u32, Unit, Repr: Mantissa = i64> {
    mantissa: Repr,
    _unit: PhantomData<Unit>,
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
}
