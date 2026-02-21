// A point in 3D space. X and Y are in
// NDC while Z is for sorting with 0
// being 0 being infinitly far
// away and f32::MAX being ontop of
// everything

use glam::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
	pub points : [Vec3; 3],
}

impl Triangle {
	pub fn new(
		p1 : Vec3,
		p2 : Vec3,
		p3 : Vec3,
	) -> Triangle {
		Triangle {
			points : [p1, p2, p3],
		}
	}
}

// Creates an equilateral triangle
// centered on the origin with side
// lengths of 1
impl Default for Triangle {
	fn default() -> Triangle {
		Triangle::new(
			Vec3::new(0_f32, 0.433012701892, 1.0_f32),
			Vec3::new(-0.5_f32, -0.433012701892, 1.0_f32),
			Vec3::new(0.5_f32, -0.433012701892, 1.0_f32),
		)
	}
}
