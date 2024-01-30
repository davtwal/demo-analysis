
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::fmt::{Display, Formatter};

use pyo3::prelude::*;
use bitbuffer::{BitRead, BitWrite};

use tf_demo_parser::demo::vector::{Vector as TFVector, VectorXY as TFVectorXY};

pub const ZERO_EPSILON_F32: f32 = 0.001;

// todo: serialize, deserialize, bitread, bitwrite ?
#[pyclass(get_all, set_all)]
#[derive(BitRead, BitWrite, Debug, Clone, Copy, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl From<Vector> for TFVector {
    fn from(value: Vector) -> Self {
        TFVector {
            x: value.x,
            y: value.y,
            z: value.z
        }
    }
}

impl From<TFVector> for Vector {
    fn from(value: TFVector) -> Self {
        Vector {
            x: value.x,
            y: value.y,
            z: value.z
        }
    }
}

impl From<VectorXY> for Vector {
    fn from(value: VectorXY) -> Self {
        value.xyz()
    }
}

impl From<Vector> for [f32; 3] {
    fn from(vec: Vector) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x < ZERO_EPSILON_F32) 
        && (self.y - other.y < ZERO_EPSILON_F32) 
        && (self.z - other.z < ZERO_EPSILON_F32)
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector{
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}

impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector{x: -self.x, y: -self.y, z: -self.z}
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector{x: self.x*rhs, y: self.y*rhs, z: self.z*rhs}
    }
}

impl Mul<Vector> for f32 {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

impl Sum for Vector {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut ret = Vector::default();
        for i in iter {
            ret = ret + i;
        }
        ret
    }
}



impl Div<f32> for Vector {
    type Output = Vector;
    fn div(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}

use quake_inverse_sqrt::QSqrt;

/// Python methods
#[pymethods]
impl Vector {
    #[new]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector {x, y, z}
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn dist_to(&self, other: &Self) -> f32 {
        ((self.x-other.x)*(self.x-other.x)
        +(self.y-other.y)*(self.y-other.y)
        +(self.z-other.z)*(self.z-other.z)).sqrt()
    }

    /// 3D Cross product. 
    pub fn cross(&self, other: &Self) -> Vector {
        Vector {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    pub fn angle_btwn(&self, other: &Self) -> f32 {
        f32::acos(self.dot(other) / (self.len() * other.len()))
    }

    pub fn abs2(&self) -> f32 {
        self.dot(self)
    }

    pub fn len(&self) -> f32 {
        self.abs2().sqrt()
    }

    pub fn xy(&self) -> VectorXY {
        VectorXY{x: self.x, y: self.y}
    }

    pub fn normalized(&self) -> Self {
        // fisqrt never panics for f32
        let i = QSqrt::fast_inverse_sqrt_unchecked(&self.abs2());
        Vector{
            x: self.x * i,
            y: self.y * i,
            z: self.z * i
        }
    }

    // Python operators
    fn __add__(&self, other: &Self) -> Self {
        *self + *other
    }
    fn __sub__(&self, other: &Self) -> Self {
        *self - *other
    }
    fn __neg__(&self) -> Self {
        -*self
    }
    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    fn __mul__(&self, other: f32) -> Self {
        *self * other
    }
    fn __rmul__(&self, other: f32) -> Self {
        *self * other
    }
    fn __truediv__(&self, other: f32) -> Self {
        *self / other
    }
    fn __rtruediv__(&self, other: f32) -> Self {
        *self / other
    }
}

////////////////
/// VectorXY

// todo: serialize, deserialize, bitread, bitwrite ?
#[pyclass(get_all, set_all)]
#[derive(BitRead, BitWrite, Debug, Clone, Copy, Default)]
pub struct VectorXY {
    pub x: f32,
    pub y: f32
}

impl Display for VectorXY {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "XY({}, {})", self.x, self.y)
    }
}

impl Add for VectorXY {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        VectorXY{
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub for VectorXY {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        VectorXY{
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Neg for VectorXY {
    type Output = Self;
    fn neg(self) -> Self::Output {
        VectorXY{x: -self.x, y: -self.y}
    }
}

impl Mul<f32> for VectorXY {
    type Output = VectorXY;
    fn mul(self, rhs: f32) -> Self::Output {
        VectorXY{x: self.x*rhs, y: self.y*rhs}
    }
}

impl Mul<VectorXY> for f32 {
    type Output = VectorXY;
    fn mul(self, rhs: VectorXY) -> Self::Output {
        rhs * self
    }
}

impl Sum for VectorXY {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut ret = VectorXY::default();
        for i in iter {
            ret = ret + i;
        }
        ret
    }
}

impl Div<f32> for VectorXY {
    type Output = VectorXY;
    fn div(self, rhs: f32) -> Self::Output {
        VectorXY {
            x: self.x / rhs,
            y: self.y / rhs
        }
    }
}

impl From<TFVectorXY> for VectorXY {
    fn from(value: TFVectorXY) -> Self {
        VectorXY {
            x: value.x,
            y: value.y
        }
    }
}

impl From<VectorXY> for TFVectorXY {
    fn from(value: VectorXY) -> Self {
        TFVectorXY {
            x: value.x,
            y: value.y
        }
    }
}

impl From<Vector> for VectorXY {
    fn from(value: Vector) -> Self {
        value.xy()
    }
}

impl From<VectorXY> for [f32; 2] {
    fn from(vec: VectorXY) -> Self {
        [vec.x, vec.y]
    }
}

#[pymethods]
impl VectorXY {
    #[new]
    pub fn new(x: f32, y: f32) -> Self {
        VectorXY {x, y}
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn dist_to(&self, other: &Self) -> f32 {
        ((self.x-other.x)*(self.x-other.x)
        +(self.y-other.y)*(self.y-other.y)).sqrt()
    }

    /// |v|^2
    pub fn abs2(&self) -> f32 {
        self.dot(self)
    }

    /// |v|
    pub fn len(&self) -> f32 {
        self.abs2().sqrt()
    }

    /// {x, y, 0}
    pub fn xyz(&self) -> Vector {
        Vector{x: self.x, y: self.y, z: 0.0}
    }

    /// same direction but length 1
    pub fn normalized(&self) -> Self {
        // fisqrt never panics for f32
        let i = QSqrt::fast_inverse_sqrt_unchecked(&self.abs2());
        VectorXY{
            x: self.x * i,
            y: self.y * i
        }
    }

    // Python operators
    fn __add__(&self, other: &Self) -> Self {
        *self + *other
    }
    fn __sub__(&self, other: &Self) -> Self {
        *self - *other
    }
    fn __neg__(&self) -> Self {
        -*self
    }
    fn __mul__(&self, other: f32) -> Self {
        *self * other
    }
    fn __rmul__(&self, other: f32) -> Self {
        *self * other
    }
    fn __truediv__(&self, other: f32) -> Self {
        *self / other
    }
    fn __rtruediv__(&self, other: f32) -> Self {
        *self / other
    }
}