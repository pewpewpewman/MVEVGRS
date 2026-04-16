use std::ops::{Add, Mul};

use glam::{Mat4, Vec3, Vec4};

use crate::pixel::Pixel;
use crate::renderer::Renderer;

//TODO: add enum that allows giving mesh simple list of points instead of list of triangles

//A mesh of triangles that have other data. The "V" type is the triangle's vertex data
//(probably contains a vec3 position. The "TE" type is the vertex transformer enviorment (probably
//contains projection-camera-model matrix). The "P" type is the pixel coloring data produced by the
//vertex transform that gets interpolated across the triangle. The "CE" type is the pixel colorer enviorment.
//Like the "TE" type, it's just information that's computed once per mesh draw and is then available to the
//vertex transform and pixel coloring functions.

#[derive(Clone)]
pub struct Mesh<V, TE, P, CE> {
	//Mesh data that's drawn,
	pub tris : Vec<Triangle<V>>,
	pub vertex_transformer : VertexTransformer<V, TE, P, CE>,
	pub trans_env_updater : VertexEnvUpdater<V, TE, P, CE>,
	pub pixel_colorer : PixelColorer<V, TE, P, CE>,
	pub color_env_updater : ColorEnvUpdater<V, TE, P, CE>,
	pub model_mat : Mat4,
}

impl<V, TE, P, CE> Mesh<V, TE, P, CE> {
	pub fn new(
		tris : Vec<Triangle<V>>,
		vertex_transformer : VertexTransformer<V, TE, P, CE>,
		pixel_colorer : PixelColorer<V, TE, P, CE>,
		trans_env_updater : VertexEnvUpdater<V, TE, P, CE>,
		color_env_updater : ColorEnvUpdater<V, TE, P, CE>,
		model_mat : Mat4,
	) -> Mesh<V, TE, P, CE> {
		Mesh {
			tris,
			vertex_transformer,
			trans_env_updater,
			pixel_colorer,
			color_env_updater,
			model_mat,
		}
	}
}

impl<V, TE, P, CE> Mesh<V, TE, P, CE> {
	//A unit cube centered at the origin
	pub fn unit_cube() -> Mesh<BasicV, BasicTE, BasicP, BasicCE> {
		Mesh::new(
			vec![
				//Front Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, -0.5),
						color : Vec3::new(1.0, 1.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.5, -0.5),
						color : Vec3::new(0.0, 1.0, 0.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, -0.5, -0.5),
						color : Vec3::new(1.0, 0.0, 0.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, -0.5),
						color : Vec3::new(1.0, 1.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
				//Back Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, -0.5, 0.5),
						color : Vec3::new(0.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.5, 0.5),
						color : Vec3::new(0.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, -0.5, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, 0.5),
						color : Vec3::new(0.0, 0.0, 1.0),
					},
				),
				//Top Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, 0.5, 0.5),
						color : Vec3::new(0.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.5, -0.5),
						color : Vec3::new(0.0, 1.0, 0.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, -0.5),
						color : Vec3::new(1.0, 1.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.5, -0.5),
						color : Vec3::new(0.0, 1.0, 0.0),
					},
				),
				//Bottom Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, -0.5, 0.5),
						color : Vec3::new(0.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, -0.5, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, -0.5, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, -0.5, -0.5),
						color : Vec3::new(1.0, 0.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
				//Right Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, 0.5, -0.5),
						color : Vec3::new(1.0, 1.0, 0.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, -0.5, -0.5),
						color : Vec3::new(1.0, 0.0, 0.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, 0.5, 0.5),
						color : Vec3::new(1.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, -0.5, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, -0.5, -0.5),
						color : Vec3::new(1.0, 0.0, 0.0),
					},
				),
				//Left Face
				//Upper Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, 0.5, 0.5),
						color : Vec3::new(0.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.5, -0.5),
						color : Vec3::new(0.0, 1.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
				//Lower Right
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, 0.5, 0.5),
						color : Vec3::new(0.0, 1.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, 0.5),
						color : Vec3::new(0.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, -0.5, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
			],
			basic_vertex_transformer,
			basic_pixel_colorer,
			basic_trans_env_updater,
			basic_color_env_updater,
			Mat4::IDENTITY,
		)
	}

	pub fn unit_plane() -> Mesh<BasicV, BasicTE, BasicP, BasicCE> {
		Mesh::new(
			vec![
				//Back Left
				Triangle::new(
					BasicV {
						position : Vec3::new(-0.5, 0.0, 0.5),
						color : Vec3::new(0.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.0, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.0, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
				//Front Right
				Triangle::new(
					BasicV {
						position : Vec3::new(0.5, 0.0, 0.5),
						color : Vec3::new(1.0, 0.0, 1.0),
					},
					BasicV {
						position : Vec3::new(0.5, 0.0, -0.5),
						color : Vec3::new(1.0, 0.0, 0.0),
					},
					BasicV {
						position : Vec3::new(-0.5, 0.0, -0.5),
						color : Vec3::new(0.0, 0.0, 0.0),
					},
				),
			],
			basic_vertex_transformer,
			basic_pixel_colorer,
			basic_trans_env_updater,
			basic_color_env_updater,
			Mat4::IDENTITY,
		)
	}
}

//The generic "V" here is the vertex data, in the simplest case,
//this will just be a position Vec3
#[derive(Debug, Clone, Copy)]
pub struct Triangle<V>(pub [V; 3]);

impl<V> Triangle<V> {
	pub fn new(
		v1 : V,
		v2 : V,
		v3 : V,
	) -> Triangle<V> {
		Triangle([v1, v2, v3])
	}
}

// Creates an equilateral triangle
// centered on the origin with side
// lengths of 1. Vertices are red blue and green
impl Default for Triangle<BasicV> {
	fn default() -> Triangle<BasicV> {
		Triangle::<BasicV>::new(
			BasicV {
				position : Vec3::new(0_f32, 0.433012701892, 0_f32),
				color : Vec3::new(1.0, 0.0, 0.0),
			},
			BasicV {
				position : Vec3::new(-0.5_f32, -0.433012701892, 0_f32),
				color : Vec3::new(0.0, 1.0, 0.0),
			},
			BasicV {
				position : Vec3::new(0.5_f32, -0.433012701892, 0_f32),
				color : Vec3::new(0.0, 0.0, 1.0),
			},
		)
	}
}

//Vertex Transform and Pixel Colorer Function Types
pub type VertexTransformer<V, TE, P, CE> =
	fn(&V, &TE, &Renderer<V, TE, P, CE>) -> VertTransOut<P>;

pub type PixelColorer<V, TE, P, CE> =
	fn(&P, &CE, &Renderer<V, TE, P, CE>) -> Pixel;

//Vertex Transer and Pixel Colorer enviorment construction types
pub type VertexEnvUpdater<V, TE, P, CE> =
	fn(&Mesh<V, TE, P, CE>, &Renderer<V, TE, P, CE>) -> TE;

pub type ColorEnvUpdater<V, TE, P, CE> =
	fn(&Mesh<V, TE, P, CE>, &Renderer<V, TE, P, CE>) -> CE;

//The output of the vertex transformer. The generic
//type "P" is interpolated and passed to the pixel
//coloring function
#[derive(Debug)]
pub struct VertTransOut<P> {
	//This position field is the vertex's position in normalized device coordinates.
	//PLEASE NOTE!!! THIS VALUE SHOULD **NOT** BE DIVIDED BY W AFTER BEING MULTIPLIED
	//BY THE PROJECTION MATRIX. GLAM HAS A Mat4::project_point3 function. DO **NOT**
	//USE IT, IT DIVIDES XYZ BY W!!
	pub pos : Vec4,
	//Data to be interpolated and passed to the coloring function
	pub colorer_in : P,
}

//A basic basic of the types put into Mesh's generics
#[derive(Clone, Copy, Debug)]
pub struct BasicV {
	pub position : Vec3,
	pub color : Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct BasicTE {
	pub p_mat : Mat4,
	pub cm_mat : Mat4,
}

//TODO: Look into derive ops crate
#[derive(Clone, Copy, Debug)]
pub struct BasicP {
	pub color : Vec3,
}

impl Mul<f32> for BasicP {
	type Output = BasicP;

	fn mul(
		self: BasicP,
		rhs : f32,
	) -> Self::Output {
		BasicP {
			color : self.color * rhs,
		}
	}
}

impl Add for BasicP {
	type Output = BasicP;

	fn add(
		self: BasicP,
		rhs : BasicP,
	) -> Self::Output {
		BasicP {
			color : self.color + rhs.color,
		}
	}
}

#[derive(Clone, Copy)]
pub struct BasicCE {}

pub fn basic_vertex_transformer(
	vert_data : &BasicV,
	vert_env : &BasicTE,
	_rend : &Renderer<BasicV, BasicTE, BasicP, BasicCE>,
) -> VertTransOut<BasicP> {
	let mut pos : Vec4 =
		vert_env.p_mat * vert_env.cm_mat * Vec4::from((vert_data.position, 1_f32));
	//pos.z = pos.z.max(0.01_f32);

	VertTransOut {
		pos,
		colorer_in : BasicP {
			color : vert_data.color,
		},
	}
}

pub fn basic_pixel_colorer(
	color_data : &BasicP,
	_color_env : &BasicCE,
	_rend : &Renderer<BasicV, BasicTE, BasicP, BasicCE>,
) -> Pixel {
	Vec4::from((color_data.color, 1.0))
}

pub fn basic_trans_env_updater(
	m : &Mesh<BasicV, BasicTE, BasicP, BasicCE>,
	r : &Renderer<BasicV, BasicTE, BasicP, BasicCE>,
) -> BasicTE {
	BasicTE {
		//Kinda sucks that proj * camera has to be computed once per mesh now :/
		p_mat : r.camera.proj_mat,
		cm_mat : r.camera.camera_mat * m.model_mat,
	}
}

pub fn basic_color_env_updater(
	_m : &Mesh<BasicV, BasicTE, BasicP, BasicCE>,
	_r : &Renderer<BasicV, BasicTE, BasicP, BasicCE>,
) -> BasicCE {
	BasicCE {}
}
