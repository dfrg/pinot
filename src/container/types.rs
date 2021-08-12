/*!
Fundamental data types defined by the font specification.
*/

#![allow(dead_code)]

use super::parse::ReadData;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Fixed point value in 2.14 format.
pub type F2dot14 = i16;

/// Signed 16-bit value in font units.
pub type FWord = i16;

/// Unsigned 16-bit value in font units.
pub type UFWord = u16;

/// Glyph identifier.
pub type GlyphId = u16;

/// Fixed point value in 2.14 format.
pub type NormalizedCoord = F2dot14;

pub(crate) mod ops {
    pub fn floor(x: i32) -> i32 {
        x & !63
    }

    pub fn ceil(x: i32) -> i32 {
        floor(x + 63)
    }

    pub fn round(x: i32) -> i32 {
        floor(x + 32)
    }

    #[inline(always)]
    pub fn mul(a: i32, b: i32) -> i32 {
        let ab = a as i64 * b as i64;
        ((ab + 0x8000 - if ab < 0 { 1 } else { 0 }) >> 16) as i32
    }

    pub fn div(mut a: i32, mut b: i32) -> i32 {
        let mut sign = 1;
        if a < 0 {
            a = -a;
            sign = -1;
        }
        if b < 0 {
            b = -b;
            sign = -sign;
        }
        let q = if b == 0 {
            0x7FFFFFFF
        } else {
            ((((a as u64) << 16) + ((b as u64) >> 1)) / (b as u64)) as u32
        };
        if sign < 0 {
            -(q as i32)
        } else {
            q as i32
        }
    }

    pub fn muldiv(mut a: i32, mut b: i32, mut c: i32) -> i32 {
        let mut sign = 1;
        if a < 0 {
            a = -a;
            sign = -1;
        }
        if b < 0 {
            b = -b;
            sign = -sign;
        }
        if c < 0 {
            c = -c;
            sign = -sign;
        }
        let d = if c > 0 {
            ((a as i64) * (b as i64) + ((c as i64) >> 1)) / c as i64
        } else {
            0x7FFFFFFF
        };
        if sign < 0 {
            -(d as i32)
        } else {
            d as i32
        }
    }

    pub fn transform(x: i32, y: i32, xx: i32, xy: i32, yx: i32, yy: i32) -> (i32, i32) {
        let scale = 0x10000;
        (
            muldiv(x, xx, scale) + muldiv(y, xy, scale),
            muldiv(x, yx, scale) + muldiv(y, yy, scale),
        )
    }
}

pub(crate) use ops::*;

/// Fixed point value in 16.16 format.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct Fixed(pub i32);

impl Fixed {
    /// Minimum value.
    pub const MIN: Self = Self(0x80000000u32 as i32);

    /// Maximum value.
    pub const MAX: Self = Self(0x7FFFFFFF);

    /// Smallest representable value.
    pub const EPSILON: Self = Self(1);

    /// The value 0.
    pub const ZERO: Self = Self(0);

    /// The value 1.
    pub const ONE: Self = Self(0x10000);

    /// Creates a 16.16 fixed point value from a 32-bit integer.
    pub const fn from_i32(x: i32) -> Self {
        Self(x << 16)
    }

    /// Creates a 16.16 fixed point value from a 32-bit floating point value.
    pub fn from_f32(x: f32) -> Self {
        Self((x * 65536. + 0.5) as i32)
    }

    /// Creates a 16.16 fixed point value from a 2.14 fixed point value.
    pub fn from_f2dot14(x: F2dot14) -> Self {
        Self(x as i32 * 4)
    }

    /// Returns the nearest integer value.
    pub fn round(self) -> Self {
        Self(((self.0 as u32 + 0x8000) & 0xFFFF0000) as i32)
    }

    /// Returns the absolute value of the number.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Returns the minimum of the two numbers.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Returns the maximum of the two numbers.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Returns the largest integer less than or equal to the number.
    pub fn floor(self) -> Self {
        Self((self.0 as u32 & 0xFFFF0000) as i32)
    }

    /// Returns the fractional part of the number.
    pub fn fract(self) -> Self {
        Self(self.0 - self.floor().0)
    }

    /// Returns the value rounded to the nearest integer.
    pub fn to_i32(self) -> i32 {
        (self.0 + 0x8000) >> 16
    }

    /// Returns the value as a 32-bit floating point number.
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 65536.
    }

    /// Returns the value as a 2.14 fixed point number.
    pub fn to_f2dot14(self) -> F2dot14 {
        ((self.0 + 2) >> 2) as F2dot14
    }
}

impl Add for Fixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self((self.0 as u32).wrapping_add(other.0 as u32) as i32)
    }
}

impl AddAssign for Fixed {
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_add(other.0);
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self((self.0 as u32).wrapping_sub(other.0 as u32) as i32)
    }
}

impl SubAssign for Fixed {
    fn sub_assign(&mut self, other: Self) {
        self.0 = self.0.wrapping_sub(other.0);
    }
}

impl Mul for Fixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        Self(mul(self.0, other.0))
    }
}

impl MulAssign for Fixed {
    fn mul_assign(&mut self, other: Self) {
        self.0 = mul(self.0, other.0);
    }
}

impl Div for Fixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: Self) -> Self {
        Self(div(self.0, other.0))
    }
}

impl DivAssign for Fixed {
    fn div_assign(&mut self, other: Self) {
        self.0 = div(self.0, other.0);
    }
}

impl Div<i32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: i32) -> Self {
        Self(self.0 / other)
    }
}

impl Neg for Fixed {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl ReadData for Fixed {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self(i32::read_data_unchecked(buf, offset))
    }
}
