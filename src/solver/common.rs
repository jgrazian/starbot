//! Primitives and traits for plate solving
//!
//! Provides the [PlateSolver] trait as well as the [PlateSolveResult] type
//! to help standardize plate solver implementations.
//!
//! Most types are simple newtype wrappers around [crate::math::Degree].
//!

use shrinkwraprs::Shrinkwrap;

use std::{error::Error, path::Path};

use crate::math::Degree;

/// Plate Solver Trait
pub trait PlateSolver {
    type E: Error;

    fn solve(
        &self,
        img: &Path,
        opts: &Option<PlateSolverOptions>,
    ) -> Result<PlateSolveResult, Self::E>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
/// Results of plate solving an image
pub struct PlateSolveResult {
    pub ra: RightAscention,
    pub dec: Declination,
    pub fov: FieldOfView,
    pub pixel_scale: PixelScale,
    pub orientation: Orientation,
}

/// Common options to give to plate solvers
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct PlateSolverOptions {
    /// Guess at FoV to help seed plate solving.
    /// If this value is too inacurate plate solving may fail.
    pub fov_guess: FieldOfView,
}

/// Right ascention
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Shrinkwrap)]
pub struct RightAscention(Degree);
impl RightAscention {
    pub fn new(deg: f64) -> Self {
        Self(Degree::new(deg))
    }
}

/// Declination
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Shrinkwrap)]
pub struct Declination(Degree);
impl Declination {
    pub fn new(deg: f64) -> Self {
        Self(Degree::new(deg))
    }
}

/// Field of view
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Shrinkwrap)]
pub struct FieldOfView(Degree);
impl FieldOfView {
    pub fn new(deg: f64) -> Self {
        Self(Degree::new(deg))
    }
}

/// Image frame rotation
///
/// units: angle east of north
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Shrinkwrap)]
pub struct Orientation(Degree);
impl Orientation {
    pub fn new(deg: f64) -> Self {
        Self(Degree::new(deg))
    }
}

/// Pixel Scale
///
/// units: angle per pixel
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Shrinkwrap)]
pub struct PixelScale(Degree);
impl PixelScale {
    pub fn new(deg: f64) -> Self {
        Self(Degree::new(deg))
    }
}
