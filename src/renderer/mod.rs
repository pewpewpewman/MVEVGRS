// Module that handles the main loop of
// drawing and rendering.

mod camera;

use std::ops::{Add, Mul, Not};

use camera::Camera;
use glam::{IVec2, Mat3, Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

use crate::mesh::{
	Mesh,
	PixelColorer,
	Triangle,
	VertTransOut,
	VertexTransformer,
};
use crate::pixel::Pixel;

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

	pub fn ndc_to_screen_coords(
		self: &Renderer<V, TE, P, CE>,
		p : &Vec3,
	) -> IVec2 {
		IVec2::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	//A triangle is on screen if any of its points are in NDC range and
	fn tri_visible(
		self: &Renderer<V, TE, P, CE>,
		transformed_verts : &[VertTransOut<P>; 3],
	) -> bool {
		transformed_verts
			.into_iter()
			.any(|v : &VertTransOut<P>| -> bool { v.pos.x > -1_f32 })
			&& transformed_verts
				.into_iter()
				.any(|v : &VertTransOut<P>| -> bool { v.pos.x < 1_f32 })
			&& transformed_verts
				.into_iter()
				.any(|v : &VertTransOut<P>| -> bool { v.pos.y > -1_f32 })
			&& transformed_verts
				.into_iter()
				.any(|v : &VertTransOut<P>| -> bool { v.pos.y < 1_f32 })
			&& transformed_verts
				.into_iter()
				.any(|v : &VertTransOut<P>| -> bool {
					v.pos.z > self.camera.near_plane
				})
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
		let mut trans_out : [VertTransOut<P>; 3] =
			tri.0.map(|v : V| -> VertTransOut<P> {
				vertex_transformer(&v, transformer_env, self)
			});

		//TODO: make this not clip triangles that are in view but have all 3 points out of NDC
		if !self.tri_visible(&trans_out) {
			//println!("TRI CULLED!!!");
			return;
		}

		let mut x_sorted : [&VertTransOut<P>; 3] = trans_out.each_ref();

		x_sorted.sort_by(
			|a : &&VertTransOut<P>, b : &&VertTransOut<P>| -> std::cmp::Ordering {
				a.pos.x.total_cmp(&b.pos.x)
			},
		);

		let mut y_sorted : [&VertTransOut<P>; 3] = trans_out.each_ref();

		y_sorted.sort_by(
			|a : &&VertTransOut<P>, b : &&VertTransOut<P>| -> std::cmp::Ordering {
				b.pos.y.total_cmp(&a.pos.y)
			},
		);

		// Perspective space NDC coordinates of scanline ndc y bounds
		// We are applying perspective correction with the w divide
		let ndc_top_y : f32 = y_sorted[0].pos.y / y_sorted[0].pos.w;
		let ndc_mid_y : f32 = y_sorted[1].pos.y / y_sorted[1].pos.w;
		let ndc_bot_y : f32 = y_sorted[2].pos.y / y_sorted[2].pos.w;

		// Camera space Z coordinates
		let cam_top_z : f32 = y_sorted[0].pos.z;
		let cam_mid_z : f32 = y_sorted[1].pos.z;
		let cam_bot_z : f32 = y_sorted[2].pos.z;

		// Screen coordinates of scanline screen y bounds
		let screen_top_y : i32 = self.ndy_to_screen_y(ndc_top_y);
		let screen_mid_y : i32 = self.ndy_to_screen_y(ndc_mid_y);
		let screen_bot_y : i32 = self.ndy_to_screen_y(ndc_bot_y);

		// slices of bounds so the two iterations can be under one loop
		let ndc_y_bounds : [f32; 3] = [ndc_top_y, ndc_mid_y, ndc_bot_y];
		let cam_z_bounds : [f32; 3] = [cam_top_z, cam_mid_z, cam_bot_z];
		let screen_y_bounds : [i32; 3] = [screen_top_y, screen_mid_y, screen_bot_y];

		//The matrix that converts a point in projected space to a vector of the world space
		//barycentric coords. The convention we will use is y_sorted[0] is "a", y_sorted[1] is "b" and y_sorted[2] is
		//"c". Formula is from https://andrewkchan.dev/posts/perspective-interpolation.html
		let bary_mat : Mat3 = Mat3::from_diagonal(Vec3::new(
			1_f32 / y_sorted[0].pos.z,
			1_f32 / y_sorted[1].pos.z,
			1_f32 / y_sorted[2].pos.z,
		))
		.mul(
			Mat3::from_cols(
				y_sorted[0].pos.xyz() / y_sorted[0].pos.w,
				y_sorted[1].pos.xyz() / y_sorted[1].pos.w,
				y_sorted[2].pos.xyz() / y_sorted[2].pos.w,
			)
			.inverse(),
		);

		for i in 0..=1 {
			let ndc_initial_y : f32 = ndc_y_bounds[i];
			let ndc_final_y : f32 = ndc_y_bounds[i + 1];

			let cam_initial_z : f32 = cam_z_bounds[i];
			let cam_final_z : f32 = cam_z_bounds[i + 1];

			let screen_initial_y : i32 = screen_y_bounds[i];
			let screen_final_y : i32 = screen_y_bounds[i + 1];

			if screen_initial_y == screen_final_y {
				continue;
			}

			let top_edge : i32 = screen_initial_y.clamp(0, self.height() as i32 - 1);
			let bot_edge : i32 = screen_final_y.clamp(0, self.height() as i32 - 1);

			// Iterate over lines of triangle - clamped to height for the **PERF**
			for y in top_edge..=bot_edge {
				let ndc_y = self.screen_y_to_ndy(y);

				let t : f32 = (y - screen_initial_y) as f32
					/ (screen_final_y - screen_initial_y) as f32;

				// We can easily find the y coordinate
				// from the side formed by 2 lines
				let mut ndc_lef_x : f32 = <f32 as glam::FloatExt>::lerp(
					y_sorted[i].pos.x / y_sorted[i].pos.w,
					y_sorted[i + 1].pos.x / y_sorted[i + 1].pos.w,
					t,
				);

				let t : f32 =
					(y - screen_top_y) as f32 / (screen_bot_y - screen_top_y) as f32;

				let mut ndc_rig_x : f32 = <f32 as glam::FloatExt>::lerp(
					y_sorted[0].pos.x / y_sorted[0].pos.w,
					y_sorted[2].pos.x / y_sorted[2].pos.w,
					t,
				);

				if ndc_lef_x > ndc_rig_x {
					std::mem::swap(&mut ndc_lef_x, &mut ndc_rig_x);
				}

				//Put bounds into screen pixel coords
				let screen_lef_x : i32 = self.ndx_to_screen_x(ndc_lef_x);
				let screen_rig_x : i32 = self.ndx_to_screen_x(ndc_rig_x);

				//Iterate over each horizontal pixel - also clamped for perf and to prevent drawing
				//in the next scan line
				let lef_edge : i32 = screen_lef_x.clamp(0, self.width() as i32 - 1);
				let rig_edge : i32 = screen_rig_x.clamp(0, self.width() as i32 - 1);

				for x in lef_edge..=rig_edge {
					//PER PIXEL OPERATIONS HERE! :D
					let ndc_x : f32 = self.screen_x_to_ndx(x);

					let [a, b, c] : [f32; 3] = Vec3::normalize(
						bary_mat * Vec3::new(ndc_x, ndc_y, self.camera.near_plane),
					)
					.to_array();

					let z : f32 = y_sorted[0].pos.z * a
						+ y_sorted[1].pos.z * b
						+ y_sorted[2].pos.z * c;

					if z < self.camera.near_plane {
						continue;
					}

					let pixel_fb_idx : usize = usize::min(
						(y * self.width() as i32 + x) as usize,
						self.frame_buffer.len() - 1,
					);

					let p : P = y_sorted[0].colorer_in * a
						+ y_sorted[1].colorer_in * b
						+ y_sorted[2].colorer_in * c;

					if z < self.depth_buffer[pixel_fb_idx] {
						let fill : Pixel = pixel_colorer(&p, &color_env, self);

						self.frame_buffer[pixel_fb_idx] =
							if !self.renderer_settings.show_tri_div {
								fill
							} else {
								if i == 0 {
									fill
								} else {
									Pixel::ONE - fill
								}
							};

						self.depth_buffer[pixel_fb_idx] = z;
					}
				}
			}
		}
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
		//Calling a function that acts on its own struct causes some borrow checker problems, let's
		//do some shenanigans to please it
		let mut temp : Option<UpdateFunc<V, TE, P, CE>> = self.update_fn.take();

		if let Some(f) = &mut temp {
			let f : &mut UpdateFunc<V, TE, P, CE> = f;
			(f)(self);
		}

		self.update_fn = temp;

		self.draw();
	}
}

type UpdateFunc<V, TE, P, CE> =
	Box<dyn FnMut(&mut Renderer<V, TE, P, CE>) -> ()>;

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
			background_col : Pixel::new(0.5, 0.75, 0.9, 0.5),
			show_tri_div : false,
		}
	}
}
