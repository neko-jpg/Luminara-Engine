//! Scalar trait for generic math.

use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

pub trait Scalar:
    Copy + Clone + PartialEq + PartialOrd +
    Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> + Neg<Output=Self> +
    AddAssign + SubAssign + MulAssign + DivAssign +
    Sized
{
    fn zero() -> Self;
    fn one() -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
}

impl Scalar for f32 {
    #[inline(always)] fn zero() -> Self { 0.0 }
    #[inline(always)] fn one() -> Self { 1.0 }
    #[inline(always)] fn sqrt(self) -> Self { self.sqrt() }
    #[inline(always)] fn sin(self) -> Self { self.sin() }
    #[inline(always)] fn cos(self) -> Self { self.cos() }
    #[inline(always)] fn tan(self) -> Self { self.tan() }
    #[inline(always)] fn asin(self) -> Self { self.asin() }
    #[inline(always)] fn acos(self) -> Self { self.acos() }
    #[inline(always)] fn atan(self) -> Self { self.atan() }
    #[inline(always)] fn atan2(self, other: Self) -> Self { self.atan2(other) }
    #[inline(always)] fn abs(self) -> Self { self.abs() }
    #[inline(always)] fn signum(self) -> Self { self.signum() }
}

impl Scalar for f64 {
    #[inline(always)] fn zero() -> Self { 0.0 }
    #[inline(always)] fn one() -> Self { 1.0 }
    #[inline(always)] fn sqrt(self) -> Self { self.sqrt() }
    #[inline(always)] fn sin(self) -> Self { self.sin() }
    #[inline(always)] fn cos(self) -> Self { self.cos() }
    #[inline(always)] fn tan(self) -> Self { self.tan() }
    #[inline(always)] fn asin(self) -> Self { self.asin() }
    #[inline(always)] fn acos(self) -> Self { self.acos() }
    #[inline(always)] fn atan(self) -> Self { self.atan() }
    #[inline(always)] fn atan2(self, other: Self) -> Self { self.atan2(other) }
    #[inline(always)] fn abs(self) -> Self { self.abs() }
    #[inline(always)] fn signum(self) -> Self { self.signum() }
}
