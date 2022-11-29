use std::{
    f64::consts::PI,
    ops::{Add, Deref, Div, Mul, Sub},
};

use crate::math::Angle;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SkyCoord {
    ra: Angle,
    dec: Angle,
    date: JulianDate,
}
impl SkyCoord {
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

    pub fn ra_dec(&self) -> (Angle, Angle) {
        (self.ra, self.dec)
    }

    pub fn alt_az(&self, ground_pos: GroundCoord) -> (Angle, Angle) {
        let (lat, long) = ground_pos.lat_long();
        //Meeus 13.5 and 13.6, modified so West longitudes are negative and 0 is North
        let gmst = greenwich_mean_sidereal_time(self.date);
        let local_sidereal_time = (gmst + long) % Angle::TAU;

        let h = match local_sidereal_time - self.ra {
            x if x.radians() < 0.0 => x + Angle::TAU,
            x if x.radians() > PI => x - Angle::TAU,
            x => x,
        };

        let az = match Angle::atan2(h.sin(), h.cos() * lat.sin() - self.dec.tan() * lat.cos())
            - Angle::PI
        {
            x if x.radians() < 0.0 => x + Angle::TAU,
            x => x,
        };
        let alt = Angle::asin(lat.sin() * self.dec.sin() + lat.cos() * self.dec.cos() * h.cos());

        (alt, az)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct GroundCoord {
    lat: Angle,
    long: Angle,
}
impl GroundCoord {
    pub fn from_lat_long(lat: Angle, long: Angle) -> Self {
        Self { lat, long }
    }
    pub fn lat_long(&self) -> (Angle, Angle) {
        (self.lat, self.long)
    }
}

/// Stores affine transformation from pixel (x, y) to world (ra, dec)
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WorldTransform {
    affine: glam::DAffine2,
}
impl WorldTransform {
    pub fn from_mat2_translation(mat2: [f64; 4], translation: [f64; 2]) -> Self {
        Self {
            affine: glam::DAffine2::from_mat2_translation(
                glam::DMat2::from_cols_array(&mat2),
                glam::DVec2::from_array(translation),
            ),
        }
    }

    pub fn pixel_to_world(&self, pixel_coord: (f64, f64)) -> SkyCoord {
        let ra_dec = self
            .affine
            .transform_point2(glam::dvec2(pixel_coord.0, pixel_coord.1));
        SkyCoord::from_ra_dec(Angle::from_degrees(ra_dec.x), Angle::from_degrees(ra_dec.y))
    }

    pub fn world_to_pixel(&self, world_coord: SkyCoord) -> (f64, f64) {
        let (ra, dec) = world_coord.ra_dec();
        let xy = self
            .affine
            .inverse()
            .transform_point2(glam::dvec2(ra.degrees(), dec.degrees()));
        (xy.x, xy.y)
    }
}

fn greenwich_mean_sidereal_time(julian_date: JulianDate) -> Angle {
    //The IAU Resolutions on Astronomical Reference Systems, Time Scales, and Earth Rotation Models Explanation and Implementation (George H. Kaplan)
    //https://arxiv.org/pdf/astro-ph/0602086.pdf
    let t = ((julian_date - JulianDate::J2000) / JulianInterval::YEAR).julian();
    let era = earth_rotation_angle(julian_date);

    //EQ 2.12
    let gmst = (86400.0 * era.rotations())
        + (0.014506 + 4612.156534 * t + 1.3915817 * t.powi(2)
            - 0.00000044 * t.powi(3)
            - 0.000029956 * t.powi(4)
            - 0.0000000368 * t.powi(5));

    Angle::from_radians(gmst)
}

fn earth_rotation_angle(julian_date: JulianDate) -> Angle {
    //https://arxiv.org/pdf/astro-ph/0602086.pdf
    let t = julian_date - JulianDate::J2000;
    let era = 0.7790572732640 + 0.00273781191135448 * t.julian() + julian_date.frac().julian();
    Angle::from_rotations(era).normalize()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct JulianDate(f64);
impl JulianDate {
    const J2000: Self = Self(2451545.0);

    fn julian(&self) -> f64 {
        self.0
    }

    fn frac(&self) -> Self {
        Self(self.0 % 1.0)
    }
}
impl Deref for JulianDate {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Sub for JulianDate {
    type Output = JulianInterval;

    fn sub(self, rhs: Self) -> Self::Output {
        JulianInterval(self.0 - rhs.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct JulianInterval(f64);
impl JulianInterval {
    const YEAR: Self = Self(36525.0);

    fn julian(&self) -> f64 {
        self.0
    }
}
impl Add for JulianInterval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        JulianInterval(self.0 + rhs.0)
    }
}
impl Sub for JulianInterval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        JulianInterval(self.0 - rhs.0)
    }
}
impl Mul for JulianInterval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        JulianInterval(self.0 * rhs.0)
    }
}
impl Div for JulianInterval {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        JulianInterval(self.0 / rhs.0)
    }
}
