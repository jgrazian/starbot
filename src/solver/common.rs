//! Primitives and traits for plate solving
//!
//! Provides the [PlateSolver] trait as well as the [PlateSolveResult] type
//! to help standardize plate solver implementations.
//!
//! Most types are simple newtype wrappers around [crate::math::Degree].
//!

use std::{error::Error, path::Path};

use crate::{
    common::{SkyCoord, WorldTransform},
    math::Angle,
};

/// Plate Solver Trait
pub trait PlateSolver {
    type E: Error;

    fn solve(
        &self,
        img: &Path,
        opts: &Option<PlateSolverOptions>,
    ) -> Result<PlateSolveResult, Self::E>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// Results of plate solving an image
pub struct PlateSolveResult {
    pub coord: SkyCoord,
    pub transform: WorldTransform,
}

/// Common options to give to plate solvers
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct PlateSolverOptions {
    /// Guess at FoV to help seed plate solving.
    /// If this value is too inacurate plate solving may fail.
    pub fov_guess: Angle,
}
