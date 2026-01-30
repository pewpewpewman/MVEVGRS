// A point in 3D space. X and Y are in
// NDC while Z is for sorting with 0
// being 0 being infinitly far
// away and f32::MAX being ontop of
// everything

#[derive(Default, Clone, Debug)]
pub struct Point {
	pub x : f32,
	pub y : f32,
	pub z : f32,
}

impl Point {
	pub fn new(
		x : f32,
		y : f32,
		z : f32,
	) -> Point {
		Point {
			x,
			y,
			z,
		}
	}
}

#[derive(Clone, Debug)]
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

// Type for conveniently reffering to
// device coordinates
pub struct ScreenCoords {
	pub x : u32,
	pub y : u32,
}

impl ScreenCoords {
	pub fn new(
		x : u32,
		y : u32,
	) -> ScreenCoords {
		ScreenCoords {
			x,
			y,
		}
	}
}

pub fn lin_interp(
	a : f32,
	b : f32,
	t : f32,
) -> f32 {
	a + t * (b - a)
}
