//! Mathy related things
//!  

/// The base angle type used in the crate
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle(f64);
impl Default for Angle {
    fn default() -> Self {
        Self(0.0)
    }
}
impl Angle {
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
}
