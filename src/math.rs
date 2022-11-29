//! Mathy related things
//!  

use std::ops::{Add, Div, Mul, Rem, Sub};

/// The base angle type used in the crate
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle(f64);
impl Default for Angle {
    fn default() -> Self {
        Self(0.0)
    }
}
impl Angle {
    pub const PI: Self = Self(180.0);
    pub const TAU: Self = Self(360.0);

    pub fn from_degrees(deg: f64) -> Self {
        Self(deg)
    }
    pub fn from_radians(rad: f64) -> Self {
        Self(rad.to_degrees())
    }
    pub fn degrees(&self) -> f64 {
        self.0
    }
    pub fn radians(&self) -> f64 {
        self.0.to_radians()
    }
    pub fn from_rotations(rot: f64) -> Self {
        Self(rot * 360.0)
    }
    pub fn rotations(&self) -> f64 {
        self.0 / 360.0
    }

    /// Normalizes the angle
    ///
    /// Converts to the following range
    /// | Units     | Min Value | Max Value |
    /// |-----------|-----------|-----------|
    /// | Degrees   | 0.0       | 360.0     |
    /// | Radians   | 0.0       | 2PI       |
    /// | Rotations | 0.0       | 1.0       |
    pub fn normalize(&self) -> Self {
        Self(((self.0 % 360.0) + 360.0) % 360.0)
    }

    pub fn sin(&self) -> f64 {
        self.0.to_radians().sin()
    }
    pub fn cos(&self) -> f64 {
        self.0.to_radians().cos()
    }
    pub fn tan(&self) -> f64 {
        self.0.to_radians().tan()
    }
    pub fn sinh(&self) -> f64 {
        self.0.to_radians().sinh()
    }
    pub fn cosh(&self) -> f64 {
        self.0.to_radians().cosh()
    }
    pub fn tanh(&self) -> f64 {
        self.0.to_radians().tanh()
    }

    pub fn asin(rad: f64) -> Self {
        Self(rad.asin().to_degrees())
    }
    pub fn acos(rad: f64) -> Self {
        Self(rad.acos().to_degrees())
    }
    pub fn atan(rad: f64) -> Self {
        Self(rad.atan().to_degrees())
    }
    pub fn atan2(x: f64, y: f64) -> Self {
        Self(x.atan2(y).to_degrees())
    }
}

impl Add for Angle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl Mul for Angle {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl Div for Angle {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl Rem for Angle {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}
