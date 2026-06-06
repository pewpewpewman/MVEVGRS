pub mod obj_file;


use glam::{Mat4, Vec3};


//TODO: add enum to support indexed rendering

//For information on the V type generic, please
//refer to ./src/renderer/user_stage/mod.rs

#[derive(Clone, Debug)]
pub struct Mesh<V> {
	//Mesh data that's drawn,
	pub tris : Vec<Tri<V>>,
	pub model_mat : Mat4,
}

impl<V> Mesh<V> {
	pub fn new(
		tris : Vec<Tri<V>>,
		model_mat : Mat4,
	) -> Mesh<V> {
		Mesh {
			tris,
			model_mat,
		}
	}
}

impl Mesh<Vec3> {
	//A unit cube centered at the origin
	pub fn unit_cube() -> Mesh<Vec3> {
		Mesh::new(
			vec![
				//Front Face
				//Upper Left
				Tri::new(
					Vec3::new(-0.5, -0.5, -0.5),
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(-0.5, 0.5, -0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(0.5, -0.5, -0.5),
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Back Face
				//Upper Left
				Tri::new(
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(-0.5, -0.5, 0.5),
				),
				//Top Face
				//Upper Left
				Tri::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(-0.5, 0.5, -0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(-0.5, 0.5, -0.5),
				),
				//Bottom Face
				//Upper Left
				Tri::new(
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Right Face
				//Upper Left
				Tri::new(
					Vec3::new(0.5, 0.5, -0.5),
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(0.5, 0.5, 0.5),
					Vec3::new(0.5, -0.5, 0.5),
					Vec3::new(0.5, -0.5, -0.5),
				),
				//Left Face
				//Upper Left
				Tri::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(-0.5, 0.5, -0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
				//Lower Right
				Tri::new(
					Vec3::new(-0.5, 0.5, 0.5),
					Vec3::new(-0.5, -0.5, 0.5),
					Vec3::new(-0.5, -0.5, -0.5),
				),
			],
			Mat4::IDENTITY,
		)
	}

	pub fn unit_plane() -> Mesh<Vec3> {
		Mesh::new(
			vec![
				//Back Left
				Tri::new(
					Vec3::new(-0.5, 0.0, 0.5),
					Vec3::new(0.5, 0.0, 0.5),
					Vec3::new(-0.5, 0.0, -0.5),
				),
				//Front Right
				Tri::new(
					Vec3::new(0.5, 0.0, 0.5),
					Vec3::new(0.5, 0.0, -0.5),
					Vec3::new(-0.5, 0.0, -0.5),
				),
			],
			Mat4::IDENTITY,
		)
	}
}

impl Default for Mesh<Vec3> {
	fn default() -> Mesh<Vec3> { Mesh::new(vec![Tri::default()], Mat4::IDENTITY) }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tri<V>(pub [V; 3]);

impl<V> Tri<V> {
	pub fn new(
		v1 : V,
		v2 : V,
		v3 : V,
	) -> Tri<V> {
		Tri([v1, v2, v3])
	}

	// Creates an equilateral triangle
	// centered on the origin with side
	// lengths of 1.
	pub fn equalat_tri() -> Tri<Vec3> {
		Tri::<Vec3>::new(
			Vec3::new(-0.0_f32, 0_f32, 0.433012701892),
			Vec3::new(-0.5_f32, 0_f32, -0.433012701892),
			Vec3::new(0.5_f32, 0_f32, -0.433012701892),
		)
	}
}
