// A point in 3D space. X and Y are in
// NDC while Z is for sorting with 0
// being 0 being infinitly far
// away and f32::MAX being ontop of
// everything

use glam::{Mat4, Vec3};
#[derive(Clone, Debug)]
pub struct Mesh {
	//Mesh data that's drawn, TODO: add enum that allows giving mesh simple list of points instead
	//of list of triangles
	pub tris : Vec<Triangle>,
	pub model_mat : Mat4,
}

impl Mesh {
	fn new(
		tris : Vec<Triangle>,
		model_mat : Mat4,
	) -> Mesh {
		Mesh {
			tris,
			model_mat,
		}
	}
}

impl Mesh {
	//A unit cube centered at the origin
	pub fn unit_cube() -> Mesh {
		Mesh::new(
			vec![
				//Front Face
				//Upper Left
				Triangle::new(
					Vec3::new(-0.5, 0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
					Vec3::new(0.5, 0.5, 0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(0.5, -0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
					Vec3::new(0.5, 0.5, 0.5),
				),
				//Back Face
				//Upper Left
				Triangle::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
				),
				//Top Face
				//Upper Left
				Triangle::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(-0.5, 0.5, -0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(-0.5, 0.5, -0.5),
				),
				//Bottom Face
				//Upper Left
				Triangle::new(
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Right Face
				//Upper Left
				Triangle::new(
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
				),
				//Left Face
				//Upper Left
				Triangle::new(
					Vec3::new(-0.5, 0.5, -0.5),
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Lower Right
				Triangle::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
			],
			Mat4::IDENTITY,
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle(pub [Vec3; 3]);

impl Triangle {
	pub fn new(
		p1 : Vec3,
		p2 : Vec3,
		p3 : Vec3,
	) -> Triangle {
		Triangle([p1, p2, p3])
	}
}

impl Triangle {
	//Area formula from https://math.stackexchange.com/questions/128991/how-to-calculate-the-area-of-a-3d-triangle
	pub fn area(self: &Triangle) -> f32 {
		let a : Vec3 = self.0[0];
		let b : Vec3 = self.0[1];
		let c : Vec3 = self.0[2];

		0.5 * (a - b).cross(a - c).length()
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
