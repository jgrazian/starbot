//! Solver wrapper for [ASTAP](https://www.hnsky.org/astap.htm), the Astrometric STAcking Program
//!
//! ASTAP's command line interface provides a ready-to-go plate solving implementation and star database.
//! This is a simple wrapper around the `astap_cli` command line program.
//!
//! # Setup
//! There is some setup required to use this wrapper.
//!
//! 1. Download the appropriate `astap_cli` application for your OS from the "Alternative links & development version"
//! section of the ASTAP website.
//! 2. Download a corresponding star database (H18, H17, G17, ..)
//! 3. Place *both* the cli executable and the star database files directly in the appropriate location for your OS
//!
//! | OS        | ASTAP Folder              |
//! |-----------|---------------------------|
//! | Windows   | `C:/Program Files/astap/` |
//! | MacOS     | `/usr/local/opt/astap/`   |
//! | Linux     | `/opt/astap/`             |
//!
//!  

use thiserror::Error;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::common::{AstroCoord, WorldTransform};
use crate::math::Angle;
use crate::solver::common;

use super::common::{PlateSolveResult, PlateSolver};

/// ASTAP solver wrapper
pub struct AstapSolver {
    cli_path: PathBuf,
}

impl AstapSolver {
    /// Create a new instance of the ASTAP solver wrapper
    /// This does not consume any resources but does check
    /// that the cli application and star database files can be found.
    pub fn new() -> Result<Self, AstapInitError> {
        let (base_path, extension) = Self::base_path()?;

        // /base/path/../astap_cli[.ext]
        let mut cli_path: PathBuf = [base_path, Path::new("astap_cli")].iter().collect();
        if let Some(extension) = extension {
            cli_path.set_extension(extension);
        }
        // Check the solver exists
        if !cli_path.try_exists()? {
            return Err(AstapInitError::SolverNotFound);
        }

        if !std::fs::read_dir(base_path)?
            .filter_map(|p| p.ok())
            .any(|p| match p.path().extension() {
                None => false,
                Some(os_str) => {
                    matches!(os_str.to_str(), Some("290") | Some("1476") | Some("001"))
                }
            })
        {
            return Err(AstapInitError::DatabaseNotFound);
        }

        Ok(Self { cli_path })
    }

    fn base_path() -> Result<(&'static Path, Option<&'static str>), AstapInitError> {
        Ok(match std::env::consts::OS {
            "linux" => (Path::new("/opt/astap/"), None),
            "macos" => (Path::new("/usr/local/opt/astap/"), None),
            "windows" => (Path::new("C:/Program Files/astap/"), Some(".exe")),
            os => return Err(AstapInitError::UnsupportedOs(os.to_string())),
        })
    }
}

impl PlateSolver for AstapSolver {
    type E = AstapSolverError;

    fn solve(
        &self,
        img: &std::path::Path,
        opts: &Option<common::PlateSolverOptions>,
    ) -> Result<common::PlateSolveResult, Self::E> {
        let mut cmd = Command::new(&self.cli_path);
        cmd.arg("-f").arg(img).stdout(Stdio::piped());
        if let Some(opts) = opts {
            cmd.arg("-fov").arg(opts.fov_guess.degrees().to_string());
        }
        let mut child = cmd.spawn()?;

        // Monitor process
        std::thread::scope(|s| {
            let num_iter = Arc::new(AtomicU8::new(0));

            let stdout = child.stdout.take().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            let num_iter_2 = num_iter.clone();
            s.spawn(move || {
                for line in stdout_lines {
                    if line.unwrap().starts_with("ASTAP solver version") {
                        num_iter_2.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            loop {
                let timeout = Duration::from_secs_f64(0.1);
                sleep(timeout);
                match child.try_wait()? {
                    Some(_status) => break,
                    None => {
                        if num_iter.load(Ordering::Relaxed) >= 3 {
                            child.kill()?;
                            return Err(AstapSolverError::IterationsExceeded);
                        }
                    }
                }
            }

            Ok(())
        })?;

        // Read wcs
        let wcs_path = img.with_extension("wcs");
        let result = parse_wcs(wcs_path.as_path());

        result
    }
}

fn parse_wcs(wcs_path: &Path) -> Result<PlateSolveResult, AstapSolverError> {
    // Parse resulting .wcs file
    let mut ra: f64 = 0.0;
    let mut dec: f64 = 0.0;
    let mut cd = [0.0; 4];

    let reader = BufReader::new(File::open(wcs_path)?);
    for line in reader.lines() {
        let line = line?;
        let contents = line.get(10..31).unwrap_or("").trim();
        match &line[0..8] {
            "CRVAL1  " => ra = contents.parse::<f64>()?,
            "CRVAL2  " => dec = contents.parse::<f64>()?,
            "CD1_1   " => cd[0] = contents.parse::<f64>()?,
            "CD1_2   " => cd[1] = contents.parse::<f64>()?,
            "CD2_1   " => cd[2] = contents.parse::<f64>()?,
            "CD2_2   " => cd[3] = contents.parse::<f64>()?,
            _ => (),
        }
    }

    Ok(PlateSolveResult {
        coord: AstroCoord::from_ra_dec(Angle::from_degrees(ra), Angle::from_degrees(dec)),
        transform: WorldTransform::new(cd, [ra, dec]),
    })
}

/// Errors from creating the wrapper
/// Likely to occur if you have not followed setup instructions
#[derive(Error, Debug)]
pub enum AstapInitError {
    #[error("Unsupported OS: {0}")]
    UnsupportedOs(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Solver not found")]
    SolverNotFound,
    #[error("Star database not found")]
    DatabaseNotFound,
}

/// Solver runtime errors
#[derive(Error, Debug)]
pub enum AstapSolverError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Solver iterations exceeded. Try increasing guess fov.")]
    IterationsExceeded,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn test_astap_wrapper() -> Result<(), Box<dyn Error>> {
        let solver = match AstapSolver::new() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let result = solver.solve(Path::new("./test/img_01.jpg"), &None)?;
        assert_eq!(
            result,
            PlateSolveResult {
                coord: AstroCoord::from_ra_dec(
                    Angle::from_degrees(234.5683671466),
                    Angle::from_degrees(88.14896797072)
                ),
                transform: WorldTransform::new(
                    [
                        -0.0006800583210471,
                        0.006300323281833,
                        0.006309995699828,
                        0.0005179551839743
                    ],
                    [234.5683671466, 88.14896797072]
                ),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_wcs() {
        let wcs_path = Path::new("./test/img_02.wcs");
        assert_eq!(
            parse_wcs(wcs_path).unwrap(),
            PlateSolveResult {
                coord: AstroCoord::from_ra_dec(
                    Angle::from_degrees(212.500334678),
                    Angle::from_degrees(87.87278365695)
                ),
                transform: WorldTransform::new(
                    [
                        9.944802584645e-6,
                        0.006515892384153,
                        0.006526241730441,
                        -4.221341466003e-5
                    ],
                    [212.500334678, 87.87278365695]
                ),
            }
        );
    }
}
