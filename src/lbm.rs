use crate::math::{lerp, Bound3, Float, Int3, Vec3};

pub struct Lattice<const X: usize, const Y: usize, const Z: usize> {
    pub q0: Box<PacketDistribution<X, Y, Z>>,
    pub q1: Box<[PacketDistribution<X, Y, Z>; 6]>,
    pub q2: Box<[PacketDistribution<X, Y, Z>; 12]>,
}

impl<const X: usize, const Y: usize, const Z: usize> Default for Lattice<X, Y, Z> {
    fn default() -> Self {
        Self {
            q0: Box::new(PacketDistribution {
                values: [[[1.0; Z]; Y]; X],
            }),
            q1: Default::default(),
            q2: Default::default(),
        }
    }
}

impl<const X: usize, const Y: usize, const Z: usize> Lattice<X, Y, Z> {
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut PacketDistribution<X, Y, Z>, Int3, Float)> {
        std::iter::once(self.q0.as_mut())
            .chain(self.q1.iter_mut())
            .chain(self.q2.iter_mut())
            .enumerate()
            .map(|(i, dist)| (dist, Self::direction(i), Self::weights(i)))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PacketDistribution<X, Y, Z>, Int3, Float)> {
        std::iter::once(self.q0.as_ref())
            .chain(self.q1.iter())
            .chain(self.q2.iter())
            .enumerate()
            .map(|(i, dist)| (dist, Self::direction(i), Self::weights(i)))
    }

    fn direction(i: usize) -> Int3 {
        const fn sign(v: i32) -> i32 {
            match v % 2 == 0 {
                true => 1,
                false => -1,
            }
        }
        let directions: [(i32, i32, i32); 19] = std::array::from_fn(|i| {
            let i = i as i32;
            match i {
                0 => (0, 0, 0),
                1..7 => {
                    let a = sign(i);
                    match i {
                        1..3 => (a, 0, 0),
                        3..5 => (0, a, 0),
                        5..7 => (0, 0, a),
                        _ => unreachable!(),
                    }
                }
                7..19 => {
                    let a = sign(i);
                    let b = sign(i / 2);
                    match i {
                        7..11 => (a, b, 0),
                        11..15 => (a, 0, b),
                        15..19 => (0, a, b),
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        });
        directions[i].into()
    }

    // https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods#Mathematical_equations_for_simulations
    fn weights(i: usize) -> Float {
        1.0 / (match i {
            0 => 3.0,
            1..7 => 18.0,
            7..19 => 36.0,
            _ => unreachable!(),
        })
    }
}

/// The packet distributions at each point in the lattice in a specific direction.
pub struct PacketDistribution<const X: usize, const Y: usize, const Z: usize> {
    values: [[[Float; Z]; Y]; X],
}

impl<const X: usize, const Y: usize, const Z: usize> Default for PacketDistribution<X, Y, Z> {
    fn default() -> Self {
        Self {
            values: [[[0.0; Z]; Y]; X],
        }
    }
}

impl<const X: usize, const Y: usize, const Z: usize> PacketDistribution<X, Y, Z> {
    pub fn get(&self, bounds: Bound3<X, Y, Z>) -> &Float {
        &self.values[bounds.x()][bounds.y()][bounds.z()]
    }
    pub fn get_mut(&mut self, bounds: Bound3<X, Y, Z>) -> &mut Float {
        &mut self.values[bounds.x()][bounds.y()][bounds.z()]
    }
}

pub struct Field<const X: usize, const Y: usize, const Z: usize, T> {
    values: [[[T; Z]; Y]; X],
}

impl<const X: usize, const Y: usize, const Z: usize, T: Default + Clone + Copy> Default
    for Field<X, Y, Z, T>
{
    fn default() -> Self {
        Self {
            values: [[[T::default(); Z]; Y]; X],
        }
    }
}

impl<const X: usize, const Y: usize, const Z: usize, T: Clone + Copy> Field<X, Y, Z, T> {
    pub fn new_from(v: T) -> Self {
        Self {
            values: [[[v; Z]; Y]; X],
        }
    }

    pub fn get(&self, bounds: Bound3<X, Y, Z>) -> &T {
        &self.values[bounds.x()][bounds.y()][bounds.z()]
    }
    pub fn get_mut(&mut self, bounds: Bound3<X, Y, Z>) -> &mut T {
        &mut self.values[bounds.x()][bounds.y()][bounds.z()]
    }
}

#[derive(Clone, Copy)]
pub struct Constants {
    pub time_relaxation_constant: Float,
    pub speed_of_sound: Float,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            time_relaxation_constant: 0.5,
            speed_of_sound: 1.0 / Float::sqrt(3.0),
        }
    }
}

pub struct Simulation<const X: usize, const Y: usize, const Z: usize> {
    pub distributions: Lattice<X, Y, Z>,
    velocity: Box<Field<X, Y, Z, Vec3>>,
    pub density: Box<Field<X, Y, Z, Float>>,
    pub constants: Constants,
}

impl<const X: usize, const Y: usize, const Z: usize> Simulation<X, Y, Z> {
    pub fn new(constants: Constants) -> Self {
        Self {
            distributions: Lattice::default(),
            velocity: Box::new(Field::default()),
            density: Box::new(Field::new_from(1.0)),
            constants,
        }
    }

    // https://en.wikipedia.org/wiki/Lattice_Boltzmann_methods#Example_implementation
    // but in 3D
    pub fn step(&mut self) {
        let collided_packets = self.collide();
        self.stream(&collided_packets);
        self.calc_conditions();
    }

    fn collide(&self) -> Box<Lattice<X, Y, Z>> {
        let mut new_packets = Box::new(Lattice::default());
        for ((distribution, direction, weight), (new_dist, _, _)) in
            self.distributions.iter().zip(new_packets.iter_mut())
        {
            for x in 0..X {
                for y in 0..Y {
                    for z in 0..Z {
                        let loc = (x, y, z).try_into().unwrap();
                        let flow_velocity = self.velocity.get(loc);
                        let direction_magnitude = flow_velocity.dot(direction.into());
                        let dm = direction_magnitude;
                        let c = self.constants.speed_of_sound;
                        let c2 = c * c;
                        // Taylor expansion of equilibrium term in this direction.
                        let equilibrium = weight
                            * self.density.get(loc)
                            * (1.0 + dm / c2 + dm * dm / (2.0 * c2 * c2)
                                - flow_velocity.dot(*flow_velocity) / (2.0 * c2));

                        // Wikipedia uses
                        // lerp(current, equilibrium, (TRC-1)/TRC)
                        // where TRC=time_relaxation_constant
                        *new_dist.get_mut(loc) = lerp(
                            *distribution.get(loc),
                            equilibrium,
                            self.constants.time_relaxation_constant,
                        );
                        // df.field[y][x][v]=Velocity * ((TimeRelaxationConstant-1)/TimeRelaxationConstant)+(equilibrium)/TimeRelaxationConstant
                    }
                }
            }
        }
        new_packets
    }

    fn stream(&mut self, collided_packets: &Lattice<X, Y, Z>) {
        for ((new_dist, direction, _), (target, _, _)) in
            collided_packets.iter().zip(self.distributions.iter_mut())
        {
            let bounds = (X as i32, Y as i32, Z as i32);
            for x in 0..bounds.0 {
                for y in 0..bounds.1 {
                    for z in 0..(bounds.2) {
                        let loc = Int3::new(x, y, z);
                        let new_loc = ((loc + direction).wrap(bounds)).try_into().unwrap();
                        *target.get_mut(new_loc) = *new_dist.get(loc.try_into().unwrap());
                    }
                }
            }
        }
    }

    pub fn calc_conditions(&mut self) {
        // let mut total_mass = 0.0;
        let mut total_momentum = Vec3::default();
        for x in 0..X {
            for y in 0..Y {
                for z in 0..Z {
                    let loc = (x, y, z).try_into().unwrap();
                    let (packet_sum, direction_sum) = self
                        .distributions
                        .iter()
                        .map(|(dist, dir, _)| {
                            let packet = dist.get(loc);
                            (*packet, *packet * Into::<Vec3>::into(dir))
                        })
                        .reduce(|acc, e| (acc.0 + e.0, acc.1 + e.1))
                        .unwrap();
                    let velocity = direction_sum / packet_sum;
                    // total_mass += packet_sum;
                    total_momentum = total_momentum + direction_sum;
                    *self.density.get_mut(loc) = packet_sum;
                    *self.velocity.get_mut(loc) = velocity;
                }
            }
        }
        // println!(
        //     "momentum = {total_momentum}, {}",
        //     total_momentum.dot(total_momentum).sqrt()
        // );
        // dbg!(total_momentum);
        // dbg!(total_mass, total_velocity);
    }
}
