use std::ops::{Add, Mul};

use glam::{Mat4, Vec3, Vec4};

use crate::renderer::Renderer;

//TODO: add enum that allows giving mesh simple list of points instead of list of triangles

//A mesh of triangles that have other data. The "V" type is the triangle's vertex data
//(probably contains a vec3 position. The "TE" type is the vertex transformer enviorment (probably
//contains projection-camera-model matrix). The "P" type is the pixel coloring data produced by the
//vertex transform that gets interpolated across the triangle. The "CE" type is the pixel colorer enviorment.
//Like the "TE" type, it's just information that's computed once per mesh draw and is then available to the
//vertex transform and pixel coloring functions.

#[derive(Clone)]
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

#[derive(Debug, Clone, Copy)]
pub struct Tri<V>(pub [V; 3]);

impl<V> Tri<V> {
	pub fn new(
		v1 : V,
		v2 : V,
		v3 : V,
	) -> Tri<V> {
		Tri([v1, v2, v3])
	}
}

// Creates an equilateral triangle
// centered on the origin with side
// lengths of 1.
impl Default for Tri<Vec3> {
	fn default() -> Tri<Vec3> {
		Tri::<Vec3>::new(
			Vec3::new(-0.0_f32, 0_f32, 0.433012701892),
			Vec3::new(-0.5_f32, 0_f32, -0.433012701892),
			Vec3::new(0.5_f32, 0_f32, -0.433012701892),
		)
	}
}
