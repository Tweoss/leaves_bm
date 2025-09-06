use std::{
    fmt::Display,
    ops::{Add, Div, Mul},
};

pub type Float = f32;

pub fn lerp(a: Float, b: Float, mix: Float) -> Float {
    b * mix + a * (1.0 - mix)
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Vec3 {
    pub fn wrap(self, (x, y, z): (usize, usize, usize)) -> Self {
        (
            self.x.rem_euclid(x as Float),
            self.y.rem_euclid(y as Float),
            self.z.rem_euclid(z as Float),
        )
            .into()
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Vec3({:+.8}, {:+.8}, {:+.8})",
            self.x, self.y, self.z
        ))
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new(0.0, 0.0, 0.0);
    pub const fn new(x: Float, y: Float, z: Float) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, other: Self) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
impl Mul<Vec3> for Float {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}
impl Div<Float> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Float) -> Self::Output {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl Add<Vec3> for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::iter::Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or(Vec3::ZERO)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Int3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Int3 {
    pub const ZERO: Self = Int3 { x: 0, y: 0, z: 0 };
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn wrap<const X: usize, const Y: usize, const Z: usize>(self) -> Bound3<X, Y, Z> {
        Bound3::new(
            self.x.rem_euclid(X as i32) as usize,
            self.y.rem_euclid(Y as i32) as usize,
            self.z.rem_euclid(Z as i32) as usize,
        )
        .unwrap()
    }
}
impl From<(i32, i32, i32)> for Int3 {
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self { x, y, z }
    }
}
impl Add<Int3> for Int3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl From<Int3> for Vec3 {
    fn from(value: Int3) -> Self {
        Self::new(value.x as Float, value.y as Float, value.z as Float)
    }
}

#[derive(Clone, Copy)]
pub struct Bound3<const X: usize, const Y: usize, const Z: usize> {
    x: usize,
    y: usize,
    z: usize,
}

impl<const X: usize, const Y: usize, const Z: usize> Bound3<X, Y, Z> {
    #[allow(clippy::result_unit_err)]
    pub fn new(x: usize, y: usize, z: usize) -> Result<Self, ()> {
        (x, y, z).try_into()
    }

    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
}

impl<const X: usize, const Y: usize, const Z: usize> TryFrom<(usize, usize, usize)>
    for Bound3<X, Y, Z>
{
    type Error = ();
    fn try_from((x, y, z): (usize, usize, usize)) -> Result<Self, Self::Error> {
        if x >= X || y >= Y || z >= Z {
            return Err(());
        }
        Ok(Self { x, y, z })
    }
}

impl<const X: usize, const Y: usize, const Z: usize> TryFrom<Int3> for Bound3<X, Y, Z> {
    type Error = ();
    fn try_from(Int3 { x, y, z }: Int3) -> Result<Self, Self::Error> {
        if x < 0 || y < 0 || z < 0 {
            return Err(());
        }
        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x >= X || y >= Y || z >= Z {
            return Err(());
        }
        Ok(Self { x, y, z })
    }
}
