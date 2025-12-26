#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod lbm;
pub mod math;
pub mod mesh;
pub use math::{Bound3, Float};

pub(crate) fn approx_eq(v1: Float, v2: Float) -> bool {
    const EPSILON: Float = 0.0001;
    (v1 - v2).abs() < EPSILON
}
