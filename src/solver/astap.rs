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

use crate::solver::common;

use super::common::{
    Declination, FieldOfView, Orientation, PixelScale, PlateSolveResult, PlateSolver,
    RightAscention,
};

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

/// Get value out of string of the format
/// CDELT1  = -6.526249307470E-003 / X pixel size (deg)
fn extract_value(s: &str) -> &str {
    let mut flag = false;
    let mut start_idx = None;
    let mut end_idx = None;

    for (i, c) in s.char_indices() {
        match (c, flag) {
            ('=', _) => flag = true,
            (' ', true) => match start_idx {
                None => continue,
                Some(_) => break,
            },
            (_, true) => match (start_idx, end_idx) {
                (None, _) => start_idx = Some(i),
                (Some(_), _) => end_idx = Some(i),
            },
            (_, false) => continue,
        }
    }
    if let Some(end_idx) = end_idx {
        s.get(start_idx.unwrap()..end_idx + 1).unwrap()
    } else {
        s.get(start_idx.unwrap()..).unwrap()
    }
}

fn parse_wcs(wcs_path: &Path) -> Result<PlateSolveResult, AstapSolverError> {
    // Parse resulting .wcs file
    let mut result = PlateSolveResult::default();

    let reader = BufReader::new(File::open(wcs_path)?);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("CRVAL1") {
            // RA
            let value_string = extract_value(&line);
            let value = value_string.parse::<f64>()?;
            result = PlateSolveResult {
                ra: RightAscention::new(value),
                ..result
            }
        } else if line.starts_with("CRVAL2") {
            // DEC
            let value_string = extract_value(&line);
            let value = value_string.parse::<f64>()?;
            result = PlateSolveResult {
                dec: Declination::new(value),
                ..result
            }
        } else if line.starts_with("CDELT1") {
            // Pixel Size
            let value_string = extract_value(&line);
            let value = value_string.parse::<f64>()?.abs();
            result = PlateSolveResult {
                pixel_scale: PixelScale::new(value),
                ..result
            }
        } else if line.starts_with("CROTA1") {
            // Pixel Size
            let value_string = extract_value(&line);
            let value = value_string.parse::<f64>()?;
            result = PlateSolveResult {
                orientation: Orientation::new(value),
                ..result
            }
        } else if line.starts_with("WARNING") {
            let fov_idx = line.find("FOV=").unwrap();
            let value = line[fov_idx + 4..fov_idx + 9].parse::<f64>()?;
            result = PlateSolveResult {
                fov: FieldOfView::new(value),
                ..result
            }
        }
    }

    Ok(result)
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
                ra: RightAscention::new(234.5683671466),
                dec: Declination::new(88.14896797072),
                fov: FieldOfView::new(25.44),
                pixel_scale: PixelScale::new(0.006346536461084),
                orientation: Orientation::new(-83.84870393426)
            }
        );
        Ok(())
    }

    #[test]
    fn test_extract_value() {
        let str = "CDELT1  = -6.526249307470E-003 / X pixel size (deg)";
        assert_eq!(extract_value(str), "-6.526249307470E-003");
    }

    #[test]
    fn test_parse_wcs() {
        let wcs_path = Path::new("./test/img_02.wcs");
        assert_eq!(
            parse_wcs(wcs_path).unwrap(),
            PlateSolveResult {
                ra: RightAscention::new(212.500334678),
                dec: Declination::new(87.87278365695),
                fov: FieldOfView::new(26.22),
                pixel_scale: PixelScale::new(0.00652624930747),
                orientation: Orientation::new(-90.08730825469)
            }
        );
    }
}
