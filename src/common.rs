use std::{f64::consts::PI, ops::Deref};

use crate::math::Angle;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AstroCoord {
    ra: Angle,
    dec: Angle,
    date: JulianDate,
}
impl AstroCoord {
    pub fn from_ra_dec(ra: Angle, dec: Angle) -> Self {
        Self {
            ra,
            dec,
            date: JulianDate::J2000,
        }
    }

    pub fn from_ra_dec_date(ra: Angle, dec: Angle, date: JulianDate) -> Self {
        Self { ra, dec, date }
    }
}

/// Stores affine transformation from pixel (x, y) to world (ra, dec)
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct WorldTransform {
    scale_rot: [f64; 4],
    translate: [f64; 2],
}
impl WorldTransform {
    pub fn new(scale_rot: [f64; 4], translate: [f64; 2]) -> Self {
        Self {
            scale_rot,
            translate,
        }
    }
    pub fn pixel_to_world(&self, coords: (f64, f64)) -> AstroCoord {
        unimplemented!()
    }
    pub fn world_to_pixel(&self, coords: AstroCoord) -> (f64, f64) {
        unimplemented!()
    }
}

// TODO: Correct this with https://www.astrogreg.com/snippets/nutation2000b.html
fn greenwich_mean_sidereal_time(jd_ut1: JulianDate) -> Angle {
    //The IAU Resolutions on Astronomical Reference Systems, Time Scales, and Earth Rotation Models Explanation and Implementation (George H. Kaplan)
    //https://arxiv.org/pdf/astro-ph/0602086.pdf
    let t = (*jd_ut1 - *JulianDate::J2000) / *JulianDate::YEAR;
    let era = earth_rotation_angle(jd_ut1);

    //EQ 2.12
    let mut gmst = (era.radians()
        + (0.014506
            + 4612.15739966 * t
            + 1.39667721 * t * t
            + -0.00009344 * t * t * t
            + 0.00001882 * t * t * t * t)
            / 60.0
            / 60.0
            * PI
            / 180.0)
        % (PI * 2.0);
    if gmst < 0.0 {
        gmst += PI * 2.0;
    }

    Angle::from_radians(gmst)
}

fn earth_rotation_angle(jd_ut1: JulianDate) -> Angle {
    //Explanatory Supplement eq 6.59
    let t = *jd_ut1 - *JulianDate::J2000;

    let frac = *jd_ut1 % 1.0;

    let mut era = (PI * 2.0 * (0.7790572732640 + 0.00273781191135448 * t + frac)) % (PI * 2.0);
    if era < 0.0 {
        era += PI * 2.0;
    }
    Angle::from_radians(era)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct JulianDate(f64);
impl JulianDate {
    const J2000: Self = Self(2451545.0);
    const YEAR: Self = Self(36525.0);
}
impl Deref for JulianDate {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
