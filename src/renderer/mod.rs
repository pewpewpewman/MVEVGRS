// Module that handles the main loop of
// drawing and rendering.

mod camera;

use std::ops::{Add, Mul};

use camera::Camera;
use glam::{IVec2, Mat3, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

use crate::mesh::{
	Mesh,
	PixelColorer,
	Triangle,
	VertTransOut,
	VertexTransformer,
};
use crate::pixel::Pixel;

type UpdateFunc<V, TE, P, CE> =
	Box<dyn FnMut(&mut Renderer<V, TE, P, CE>) -> ()>;

//The main renderer. For information on what these type generics do, please refer to
//./src/mesh/mod.rs
pub struct Renderer<V, TE, P, CE> {
	// Main frame buffer that is written to
	pub frame_buffer : Vec<Pixel>,
	// Depth buffer that is used for knowing what tris are visible
	pub depth_buffer : Vec<f32>,
	// Settings for how to draw things
	pub renderer_settings : RendererSettings,
	// Camera that holds the camera and projection matrix
	pub camera : Camera,
	// Triangles to be rastered
	pub meshes : Vec<Mesh<V, TE, P, CE>>,
	// Update function to run before drawing each frame
	update_fn : Option<UpdateFunc<V, TE, P, CE>>,
}

impl<V, TE, P, CE> Renderer<V, TE, P, CE>
where
	V : Clone + Copy,
	TE : Clone,
	P : Clone + Copy + Mul<f32, Output = P> + Add<Output = P>,
	CE : Clone,
{
	pub fn new(
		renderer_settings : RendererSettings,
		meshes : Vec<Mesh<V, TE, P, CE>>,
		update_fn : Option<Box<dyn FnMut(&mut Renderer<V, TE, P, CE>) -> ()>>,
	) -> Renderer<V, TE, P, CE> {
		let pix_area : usize =
			(renderer_settings.width * renderer_settings.height) as usize;

		Renderer {
			frame_buffer : vec![renderer_settings.background_col; pix_area],
			depth_buffer : vec![f32::MAX; pix_area],
			renderer_settings,
			camera : Camera::default(),
			meshes,
			update_fn,
		}
	}

	pub fn width(self: &Renderer<V, TE, P, CE>) -> u32 {
		self.renderer_settings.width
	}

	pub fn height(self: &Renderer<V, TE, P, CE>) -> u32 {
		self.renderer_settings.height
	}

	// Helpful conversion functions between
	// NDC and pixel coordinates and vice
	// versa
	pub fn screen_x_to_ndx(
		self: &Renderer<V, TE, P, CE>,
		x : i32,
	) -> f32 {
		x as f32 / self.width() as f32 * 2_f32 - 1_f32
	}

	pub fn screen_y_to_ndy(
		self: &Renderer<V, TE, P, CE>,
		y : i32,
	) -> f32 {
		(1_f32 - (y as f32 / self.height() as f32)) * 2_f32 - 1_f32
	}

	pub fn screen_coords_to_ndc(
		self: &Renderer<V, TE, P, CE>,
		c : IVec2,
	) -> Vec3 {
		Vec3::new(self.screen_x_to_ndx(c.x), self.screen_y_to_ndy(c.y), 0_f32)
	}

	pub fn ndx_to_screen_x(
		self: &Renderer<V, TE, P, CE>,
		x : f32,
	) -> i32 {
		f32::round(self.width() as f32 * ((1_f32 + x) / 2_f32)) as i32
	}

	pub fn ndy_to_screen_y(
		self: &Renderer<V, TE, P, CE>,
		y : f32,
	) -> i32 {
		f32::round(self.height() as f32 * (1_f32 - ((1_f32 + y) / 2_f32))) as i32
	}

	pub fn ndc_to_screen_c(
		self: &Renderer<V, TE, P, CE>,
		p : Vec2,
	) -> IVec2 {
		IVec2::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	// Draw a single triangle to the
	// frame_buffer
	fn raster_tri(
		self: &mut Renderer<V, TE, P, CE>,
		tri : &Triangle<V>,
		vertex_transformer : VertexTransformer<V, TE, P, CE>,
		transformer_env : &TE,
		pixel_colorer : PixelColorer<V, TE, P, CE>,
		color_env : &CE,
	) -> () {
		//Vertex information from the vertex transformer
		let trans_out : [VertTransOut<P>; 3] =
			tri.0.map(|v : V| -> VertTransOut<P> {
				vertex_transformer(&v, transformer_env, self)
			});

		//Only needed to make further references to
		//vertex coords cleaner
		let tri_points : [Vec4; 3] = trans_out
			.each_ref()
			.map(|vto : &VertTransOut<P>| -> Vec4 { vto.pos });

		//Points on the convex polygon that results from clipping - Praying
		//to the rust compiler that it smartly eliminates heap allocations
		let mut poly_points : Vec<Vec4> = Vec::with_capacity(6);

		//Iterate over every pair of points, then iterate over each
		//coordinate to be clipped, then iterate over each sign of
		//that coordinate, and then clip
		tri_points
			.iter()
			.enumerate()
			.for_each(|(i, v1) : (usize, &Vec4)| -> () {
				if tri_points.iter().enumerate().all(
					|(j, v2) : (usize, &Vec4)| -> bool {
						if i == j {
							return true;
						}

						(0..=1_usize).all(|c : usize| -> bool {
							[-1_f32, 1_f32].into_iter().all(|mut s : f32| -> bool {
								if v1[c] > v1.w && v2[c] < v2.w {
									//Formula from
									//https://www.cs.ucr.edu/~shinar/courses/cs130-winter-2021/content/clipping.pdf
									//It brings me immeasurable pain to use something from UCR -
									//but it is my only hope...
									let alpha : f32 =
										(s * v2.w - v2[c]) / (v1[c] + s * -v1.w + s * v2.w - v2[c]);
									poly_points.push(alpha * v1 + (1_f32 - alpha) * v2);
									return false;
								} else {
									return true;
								}
							})
						})
					},
				) {
					poly_points.push(*v1);
				}
			});

		dbg!(poly_points.len());

		//Perspective divide
		poly_points.iter_mut().for_each(|v : &mut Vec4| -> () {
			*v = Vec4::from((v.xyz() / v.w, v.w));
		});

		/*
		poly_points = vec![
			Vec4::new(0.35, 0.17, 1_f32, 1_f32),
			Vec4::new(0.248, 0.398, 1_f32, 1_f32),
			Vec4::new(-0.472, 0.609, 1_f32, 1_f32),
			Vec4::new(-0.58, -0.126, 1_f32, 1_f32),
			Vec4::new(-0.016, -0.692, 1_f32, 1_f32),
			Vec4::new(0.299, -0.396, 1_f32, 1_f32),
			Vec4::new(-0.411, -0.547, 1_f32, 1_f32),
		];
		*/

		poly_points.sort_by(|a : &Vec4, b : &Vec4| -> std::cmp::Ordering {
			b.y.total_cmp(&a.y)
		});

		poly_points[0..=poly_points.len() - 3]
			.iter()
			.enumerate()
			.for_each(|(i, p) : (usize, &Vec4)| -> () {
				let lef_pnt : Vec2 = poly_points[i + 1..]
					.iter()
					.find(|a : &&Vec4| -> bool { a.x <= p.x })
					.map_or(poly_points[i + 2].xy(), |v : &Vec4| -> Vec2 { v.xy() });

				let rig_pnt : Vec2 = poly_points[i + 1..]
					.iter()
					.find(|a : &&Vec4| -> bool { a.x >= p.x })
					.map_or(poly_points[i + 2].xy(), |v : &Vec4| -> Vec2 { v.xy() });

				let lef_is_mid : bool = lef_pnt.y > rig_pnt.y;
				let y_sorted : [Vec2; 3] = if lef_is_mid {
					[p.xy(), lef_pnt, rig_pnt]
				} else {
					[p.xy(), rig_pnt, lef_pnt]
				};

				let y_sorted_screen : [i32; 3] =
					y_sorted.map(|v : Vec2| -> i32 { self.ndy_to_screen_y(v.y) });

				//Iterate over the triangle in two segments
				(0..=1_usize).for_each(|j : usize| -> () {
					//Iterate the top to the mid point in the first iteration
					//and then from the mid point to the bottom in the second
					(y_sorted_screen[j]..y_sorted_screen[j + 1]).for_each(
						|y_screen : i32| -> () {
							let y_ndc : f32 = self.screen_y_to_ndy(y_screen);
							//t used to lerp between the unbroken edge of the triangle
							let full_t : f32 =
								(y_ndc - y_sorted[0].y) / (y_sorted[2].y - y_sorted[0].y);

							//t used to lerp between the 2 broken edges of the triangle
							let part_t : f32 =
								(y_ndc - y_sorted[j].y) / (y_sorted[j + 1].y - y_sorted[j].y);

							let mut full_x : f32 =
								y_sorted[0].x + (y_sorted[2].x - y_sorted[0].x) * full_t;

							let mut part_x : f32 =
								y_sorted[j].x + (y_sorted[j + 1].x - y_sorted[j].x) * part_t;

							if full_x > part_x {
								std::mem::swap(&mut full_x, &mut part_x)
							}

							let init_x : i32 = self.ndx_to_screen_x(full_x);
							let fina_x : i32 = self.ndx_to_screen_x(part_x);

							(init_x..fina_x).for_each(|x_screen : i32| -> () {
								let fb_idx : usize =
									((y_screen * self.width() as i32) + x_screen) as usize;

								self.frame_buffer[fb_idx] = Vec4::new(full_t, part_t, 0.0, 1.0);
							});
						},
					);
				});

				//MARK VERTICES ON TRIANGLE IN RED
				[p.xy(), lef_pnt, rig_pnt]
					.into_iter()
					.for_each(|v : Vec2| -> () {
						let y : i32 = self.ndy_to_screen_y(v.y);
						let x : i32 = self.ndx_to_screen_x(v.x);
						(-2..=2).for_each(|x_offset : i32| -> () {
							(-2..=2).for_each(|y_offset : i32| -> () {
								let fb_idx : usize = (((y + y_offset) * self.width() as i32)
									+ x + x_offset) as usize;

								self.frame_buffer[fb_idx] = Vec4::new(1.0, 0.0, 0.0, 1.0);
							});
						});
					});
				//END OF RED DEBUG VERTS
			});
	}

	pub fn draw(self: &mut Renderer<V, TE, P, CE>) -> () {
		// Raster all triangles
		self
			.frame_buffer
			.fill(self.renderer_settings.background_col);

		self.depth_buffer.fill(f32::MAX);

		let _proj_cam_mat : Mat4 = self.camera.proj_mat * self.camera.camera_mat;

		self
			.meshes
			.clone()
			.into_iter()
			.for_each(|m : Mesh<V, TE, P, CE>| -> () {
				let trans_env : TE = (m.trans_env_updater)(&m, self);

				let color_env : CE = (m.color_env_updater)(&m, self);

				m.tris.iter().for_each(|t : &Triangle<V>| -> () {
					self.raster_tri(
						&t,
						m.vertex_transformer,
						&trans_env,
						m.pixel_colorer,
						&color_env,
					);
				});
			});
	}

	pub fn frame_step(self: &mut Renderer<V, TE, P, CE>) -> () {
		//Calling a function that acts on its own struct causes
		//some borrow checker problems, let's do some shenanigans
		//to please it.
		let mut temp : Option<UpdateFunc<V, TE, P, CE>> = self.update_fn.take();
		if let Some(f) = &mut temp {
			let f : &mut UpdateFunc<V, TE, P, CE> = f;
			(f)(self);
		}
		self.update_fn = temp;

		self.draw();
	}
}

pub struct RendererSettings {
	// INTERNAL render width and height - may or may not match up with what the target for
	// rendering is
	pub width : u32,
	pub height : u32,
	// Go to value for filling the frame buffer
	pub background_col : Pixel,
	// Triangles are drawn in 2 phases, set this to true if you want the second phase to have
	// inverted colors
	pub show_tri_div : bool,
}

impl Default for RendererSettings {
	fn default() -> RendererSettings {
		RendererSettings {
			width : 320 * 2,
			height : 240 * 2,
			background_col : Pixel::new(0.8, 0.45, 0.3, 0.5),
			show_tri_div : false,
		}
	}
}
