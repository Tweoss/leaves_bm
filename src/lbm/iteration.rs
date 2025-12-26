use crate::{
    lbm::{Lattice, PacketDistribution},
    math::Int3,
    Float,
};

enum LatticeIndex {
    Q0,
    Q1(usize),
    Q2(usize),
}

impl LatticeIndex {
    // ~100% speedup over not inlined.
    #[inline(always)]
    const fn direction(&self) -> Int3 {
        const fn sign(v: usize) -> i32 {
            match v.is_multiple_of(2) {
                true => 1,
                false => -1,
            }
        }
        let (x, y, z) = match self {
            LatticeIndex::Q0 => (0, 0, 0),
            LatticeIndex::Q1(i) => {
                let i = *i;
                let a = sign(i);
                match i {
                    0..2 => (a, 0, 0),
                    2..4 => (0, a, 0),
                    4..6 => (0, 0, a),
                    _ => unreachable!(),
                }
            }
            LatticeIndex::Q2(i) => {
                let i = *i;
                let a = sign(i);
                let b = sign(i / 2);
                match i {
                    0..4 => (a, b, 0),
                    4..8 => (a, 0, b),
                    8..12 => (0, a, b),
                    _ => unreachable!(),
                }
            }
        };
        Int3::new(x, y, z)
    }
    // https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods#Mathematical_equations_for_simulations
    const fn weight(&self) -> Float {
        match self {
            LatticeIndex::Q0 => 1.0 / 3.0,
            LatticeIndex::Q1(_) => 1.0 / 18.0,
            LatticeIndex::Q2(_) => 1.0 / 36.0,
        }
    }
}

impl<const X: usize, const Y: usize, const Z: usize> Lattice<X, Y, Z> {
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut PacketDistribution<X, Y, Z>, Int3, Float)> {
        std::iter::once((
            self.q0.as_mut(),
            LatticeIndex::Q0.direction(),
            LatticeIndex::Q0.weight(),
        ))
        .chain(self.q1.iter_mut().enumerate().map(|(i, d)| {
            (
                d,
                LatticeIndex::Q1(i).direction(),
                LatticeIndex::Q1(i).weight(),
            )
        }))
        .chain(self.q2.iter_mut().enumerate().map(|(i, d)| {
            (
                d,
                LatticeIndex::Q2(i).direction(),
                LatticeIndex::Q2(i).weight(),
            )
        }))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PacketDistribution<X, Y, Z>, Int3, Float)> {
        std::iter::once((
            self.q0.as_ref(),
            LatticeIndex::Q0.direction(),
            LatticeIndex::Q0.weight(),
        ))
        .chain(self.q1.iter().enumerate().map(|(i, d)| {
            (
                d,
                LatticeIndex::Q1(i).direction(),
                LatticeIndex::Q1(i).weight(),
            )
        }))
        .chain(self.q2.iter().enumerate().map(|(i, d)| {
            (
                d,
                LatticeIndex::Q2(i).direction(),
                LatticeIndex::Q2(i).weight(),
            )
        }))
    }

    // TODO: this could be way cleaner ... probably
    pub fn iter_pairs(&mut self) -> [[(&mut PacketDistribution<X, Y, Z>, Int3, Float); 2]; 9] {
        let [q1_0, q1_1, q1_2, q1_3, q1_4, q1_5] = self.q1.each_mut();
        let [q2_0, q2_1, q2_2, q2_3, q2_4, q2_5, q2_6, q2_7, q2_8, q2_9, q2_10, q2_11] =
            self.q2.each_mut();
        [
            [
                (
                    q1_0,
                    LatticeIndex::Q1(0).direction(),
                    LatticeIndex::Q1(0).weight(),
                ),
                (
                    q1_1,
                    LatticeIndex::Q1(1).direction(),
                    LatticeIndex::Q1(1).weight(),
                ),
            ],
            [
                (
                    q1_2,
                    LatticeIndex::Q1(2).direction(),
                    LatticeIndex::Q1(2).weight(),
                ),
                (
                    q1_3,
                    LatticeIndex::Q1(3).direction(),
                    LatticeIndex::Q1(3).weight(),
                ),
            ],
            [
                (
                    q1_4,
                    LatticeIndex::Q1(4).direction(),
                    LatticeIndex::Q1(4).weight(),
                ),
                (
                    q1_5,
                    LatticeIndex::Q1(5).direction(),
                    LatticeIndex::Q1(5).weight(),
                ),
            ],
            [
                (
                    q2_0,
                    LatticeIndex::Q2(0).direction(),
                    LatticeIndex::Q2(0).weight(),
                ),
                (
                    q2_3,
                    LatticeIndex::Q2(3).direction(),
                    LatticeIndex::Q2(3).weight(),
                ),
            ],
            [
                (
                    q2_1,
                    LatticeIndex::Q2(1).direction(),
                    LatticeIndex::Q2(1).weight(),
                ),
                (
                    q2_2,
                    LatticeIndex::Q2(2).direction(),
                    LatticeIndex::Q2(2).weight(),
                ),
            ],
            [
                (
                    q2_4,
                    LatticeIndex::Q2(4).direction(),
                    LatticeIndex::Q2(4).weight(),
                ),
                (
                    q2_7,
                    LatticeIndex::Q2(7).direction(),
                    LatticeIndex::Q2(7).weight(),
                ),
            ],
            [
                (
                    q2_5,
                    LatticeIndex::Q2(5).direction(),
                    LatticeIndex::Q2(5).weight(),
                ),
                (
                    q2_6,
                    LatticeIndex::Q2(6).direction(),
                    LatticeIndex::Q2(6).weight(),
                ),
            ],
            [
                (
                    q2_8,
                    LatticeIndex::Q2(8).direction(),
                    LatticeIndex::Q2(8).weight(),
                ),
                (
                    q2_11,
                    LatticeIndex::Q2(11).direction(),
                    LatticeIndex::Q2(11).weight(),
                ),
            ],
            [
                (
                    q2_9,
                    LatticeIndex::Q2(9).direction(),
                    LatticeIndex::Q2(9).weight(),
                ),
                (
                    q2_10,
                    LatticeIndex::Q2(10).direction(),
                    LatticeIndex::Q2(10).weight(),
                ),
            ],
        ]
    }
}
