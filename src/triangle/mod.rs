// A point in 3D space. X and Y are in
// NDC while Z is for sorting with 0
// being 0 being infinitly far
// away and f32::MAX being ontop of
// everything

pub type Point = glam::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
	pub points : [Point; 3],
}

impl Triangle {
	pub fn new(
		p1 : Point,
		p2 : Point,
		p3 : Point,
	) -> Triangle {
		Triangle {
			points : [p1, p2, p3],
		}
	}
}

pub type ScreenCoord = glam::UVec2;

// Creates an equilateral triangle
// centered on the origin with side
// lengths of 1
impl Default for Triangle {
	fn default() -> Triangle {
		Triangle::new(
			Point::new(0_f32, 0.433012701892, 1.0_f32),
			Point::new(-0.5_f32, -0.433012701892, 1.0_f32),
			Point::new(0.5_f32, -0.433012701892, 1.0_f32),
		)
	}
}
