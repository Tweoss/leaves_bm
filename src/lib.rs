#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod lbm;
mod math;
pub use lbm::{Constants, Simulation};
pub use math::{Bound3, Float};
