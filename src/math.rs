//! Mathy related things
//!  

/// The base angle type used in the crate
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Degree(f64);
impl Default for Degree {
    fn default() -> Self {
        Self(0.0)
    }
}
impl Degree {
    pub fn new(deg: f64) -> Self {
        Self(deg)
    }
    pub fn degrees(&self) -> f64 {
        self.0
    }
    pub fn radians(&self) -> f64 {
        self.0.to_radians()
    }
}
