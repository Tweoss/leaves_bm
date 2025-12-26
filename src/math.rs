use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::approx_eq;

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
    pub fn normalized(self) -> Self {
        self / self.dot(self).sqrt()
    }
    pub fn project_onto(self, other: Self) -> Self {
        let other = other.normalized();
        self.dot(other) * other
    }
    pub fn orthonormal(self, other: Self) -> (Self, Self) {
        let a = self.normalized();
        let projected = a.dot(other) * a;
        (a, (other - projected).normalized())
    }
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
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
    pub const fn from_slice([x, y, z]: [Float; 3]) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, other: Self) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn approx_eq(self, other: Self) -> bool {
        (approx_eq(self.x, other.x)) && (approx_eq(self.y, other.y)) && (approx_eq(self.z, other.z))
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
impl Sub<Vec3> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}
impl std::iter::Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or(Vec3::ZERO)
    }
}

#[cfg(test)]
mod vec_test {
    use crate::approx_eq;

    use super::Vec3;
    #[test]
    fn add_sub_is_noop() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(1.75, 1.5, 1.25);
        assert!((a - v + v).approx_eq(a));
    }
    #[test]
    fn normalize_gives_unit() {
        let v = Vec3::new(1.75, 1.5, 1.25).normalized();
        assert!(approx_eq(v.dot(v), 1.0));
    }
    #[test]
    fn orthonormal_gives_normal() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(-3.0, -2.0, -1.0);
        let (a, b) = a.orthonormal(b);
        assert!(approx_eq(a.dot(a), 1.0));
        assert!(approx_eq(b.dot(b), 1.0));
        assert!(approx_eq(a.dot(b), 0.0));
    }
    #[test]
    fn cross_gives_normal() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(-3.0, -2.0, -1.0);
        let c = a.cross(b);
        assert!(approx_eq(a.dot(c), 0.0));
        assert!(approx_eq(b.dot(c), 0.0));
    }
    #[test]
    fn cross_ij_gives_k() {
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        let k = i.cross(j);
        assert!(k.approx_eq(Vec3::new(0.0, 0.0, 1.0)));
        // Reversing the order of a cross product negates the result.
        assert!(j.cross(i).approx_eq(-k));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug, Clone)]
pub struct Matrix3 {
    rows: [[Float; 3]; 3],
}
impl Matrix3 {
    pub const IDENTITY: Matrix3 = Matrix3 {
        rows: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    pub fn from_columns(col0: Vec3, col1: Vec3, col2: Vec3) -> Self {
        Self {
            rows: [
                [col0.x, col1.x, col2.x],
                [col0.y, col1.y, col2.y],
                [col0.z, col1.z, col2.z],
            ],
        }
    }
    fn det2x2(a: Float, b: Float, c: Float, d: Float) -> Float {
        a * d - b * c
    }
    pub fn det(&self) -> Float {
        self.rows[0][0]
            * Self::det2x2(
                self.rows[1][1],
                self.rows[1][2],
                self.rows[2][1],
                self.rows[2][2],
            )
            - self.rows[1][0]
                * Self::det2x2(
                    self.rows[0][1],
                    self.rows[0][2],
                    self.rows[2][1],
                    self.rows[2][2],
                )
            + self.rows[2][0]
                * Self::det2x2(
                    self.rows[0][1],
                    self.rows[0][2],
                    self.rows[1][1],
                    self.rows[1][2],
                )
    }
    pub fn inverse(&self) -> Self {
        let det = self.det();
        let m = self.rows;
        (1.0 / det)
            * Self {
                rows: [
                    [
                        Self::det2x2(m[1][1], m[1][2], m[2][1], m[2][2]),
                        Self::det2x2(m[0][2], m[0][1], m[2][2], m[2][1]),
                        Self::det2x2(m[0][1], m[0][2], m[1][1], m[1][2]),
                    ],
                    [
                        Self::det2x2(m[1][2], m[1][0], m[2][2], m[2][0]),
                        Self::det2x2(m[0][0], m[0][2], m[2][0], m[2][2]),
                        Self::det2x2(m[0][2], m[0][0], m[1][2], m[1][0]),
                    ],
                    [
                        Self::det2x2(m[1][0], m[1][1], m[2][0], m[2][1]),
                        Self::det2x2(m[0][1], m[0][0], m[2][1], m[2][0]),
                        Self::det2x2(m[0][0], m[0][1], m[1][0], m[1][1]),
                    ],
                ],
            }
    }
    pub fn approx_eq(self, other: Self) -> bool {
        self.rows
            .iter()
            .zip(other.rows.iter())
            .all(|(r1, r2)| r1.iter().zip(r2.iter()).all(|(v1, v2)| approx_eq(*v1, *v2)))
    }
}
impl Mul<Matrix3> for Matrix3 {
    type Output = Matrix3;

    fn mul(self, m: Matrix3) -> Self::Output {
        let columns = [
            Vec3::new(m.rows[0][0], m.rows[1][0], m.rows[2][0]),
            Vec3::new(m.rows[0][1], m.rows[1][1], m.rows[2][1]),
            Vec3::new(m.rows[0][2], m.rows[1][2], m.rows[2][2]),
        ];
        Matrix3 {
            rows: self.rows.map(|r| {
                let r = Vec3::from_slice(r);
                [r.dot(columns[0]), r.dot(columns[1]), r.dot(columns[2])]
            }),
        }
    }
}
impl Mul<Vec3> for &Matrix3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let rs = self.rows;
        Vec3::new(
            rs[0][0] * rhs.x + rs[0][1] * rhs.y + rs[0][2] * rhs.z,
            rs[1][0] * rhs.x + rs[1][1] * rhs.y + rs[1][2] * rhs.z,
            rs[2][0] * rhs.x + rs[2][1] * rhs.y + rs[2][2] * rhs.z,
        )
    }
}
impl Mul<Matrix3> for f32 {
    type Output = Matrix3;

    fn mul(self, m: Matrix3) -> Self::Output {
        Matrix3 {
            rows: m.rows.map(|r| r.map(|el| self * el)),
        }
    }
}
#[cfg(test)]
mod mat_test {
    use crate::math::{Matrix3, Vec3};

    #[test]
    fn inverse_of_identity() {
        assert!(
            Matrix3::IDENTITY.inverse().approx_eq(Matrix3::IDENTITY),
            "got {:?} and expected {:?}",
            Matrix3::IDENTITY.inverse(),
            Matrix3::IDENTITY
        );
    }
    #[test]
    fn invert_of_scaling_matrix() {
        let scaling = Matrix3::from_columns(
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(0.0, 0.0, 0.2),
        );
        let inverse = scaling.inverse();
        let result = scaling.clone() * inverse.clone();

        assert!(
            result.clone().approx_eq(Matrix3::IDENTITY),
            "got {:?} and expected {:?}",
            result,
            Matrix3::IDENTITY
        );
    }
    #[test]
    fn invert_of_matrix() {
        let scaling = Matrix3::from_columns(
            Vec3::new(2.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let inverse = scaling.inverse();
        let result = scaling.clone() * inverse.clone();

        assert!(
            result.clone().approx_eq(Matrix3::IDENTITY),
            "got {:?} and expected {:?}",
            result,
            Matrix3::IDENTITY
        );
    }
}
