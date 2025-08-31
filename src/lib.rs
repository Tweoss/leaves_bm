#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod lbm;
pub mod math;
pub use lbm::{Constants, Lattice, PacketDistribution, Simulation};
pub use math::{Bound3, Float};
