use crate::{
    math::{Matrix3, Vec3},
    Float,
};

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

pub struct Triangle {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
}

impl Triangle {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
        Self { p0, p1, p2 }
    }
    // TODO: numerical error with almost parallel?
    /// Check if the triangle intersects a line segment.
    ///
    /// If it does, returns the proportion of the line segment before the triangle.
    pub fn intersect_proportion(&self, p0: Vec3, p1: Vec3) -> Option<Float> {
        let base = self.p2;
        let (t0, t1) = (self.p0 - base, self.p1 - base);
        let normal = t0.cross(t1);
        // Z coordinate is normal component.
        let to_triangle_basis = Matrix3::from_columns(t0, t1, normal).inverse();
        let (l0, l1) = (p0 - base, p1 - base);
        let (l0_prime, l1_prime) = (&to_triangle_basis * l0, &to_triangle_basis * l1);
        let normal_component_prod = l0_prime.z * l1_prime.z;
        // If the line segment is on one side of the triangle
        // (matching sign for normal component)
        if normal_component_prod > 0.0 {
            return None;
        }
        // if the line segment touches the triangle (normal component is 0)
        if l0_prime.z == 0.0 {
            return Some(0.0);
        }
        if l1_prime.z == 0.0 {
            return Some(1.0);
        }
        let difference = l1_prime - l0_prime;
        // We want to know where the line segment intersects the triangle
        // in terms of t0 and t1.
        // In other words, we want to have normal component = 0
        // along the line between l1_prime and l0_prime or equivalently
        // the line t * difference + l0_prime.
        let line_scale = -l0_prime.z / difference.z;
        let t0_component = line_scale * difference.x + l0_prime.x;
        let t1_component = line_scale * difference.y + l0_prime.y;
        let (c0, c1) = (t0_component, t1_component);

        // For the intersection to lie in the triangle, each component must
        // be between 0 and 1, and the sum must be less than 1.
        if (0.0..=1.0).contains(&c0)
            && (0.0..=1.0).contains(&c1)
            && (0.0..=1.0).contains(&(c0 + c1))
        {
            Some(line_scale)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod mesh_test {
    use super::{Triangle, Vec3};

    #[test]
    fn triangle_intersect_tests() {
        let triangle = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        );
        // Touching works.
        assert_eq!(
            triangle.intersect_proportion(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            Some(0.0)
        );
        // Going through works.
        assert_eq!(
            triangle.intersect_proportion(Vec3::new(0.1, 0.1, -1.0), Vec3::new(0.1, 0.1, 1.0)),
            Some(0.5)
        );
        // Slightly tilted.
        assert_eq!(
            triangle.intersect_proportion(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.1, 0.1, 1.0)),
            Some(0.5)
        );

        // Completely tangent
        // not sure correct behavior here
        assert_eq!(
            triangle.intersect_proportion(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            Some(0.0)
        );

        // Going by the side won't intersect.
        assert!(triangle
            .intersect_proportion(Vec3::new(-0.1, -0.1, -1.0), Vec3::new(-0.1, -0.1, 1.0))
            .is_none());
        // Not quite touching.
        assert!(triangle
            .intersect_proportion(Vec3::new(0.1, 0.1, 0.1), Vec3::new(0.1, 0.1, 1.0))
            .is_none());
    }
}
