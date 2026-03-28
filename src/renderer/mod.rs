// Module that handles the main loop of
// drawing and rendering.

mod camera;

use std::ops::Not;

use camera::Camera;
use glam::{IVec2, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

use crate::mesh::{Mesh, Triangle};
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

impl<V : Clone, TE : Clone, P : Clone, CE : Clone> Renderer<V, TE, P, CE> {
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

	//Test to see if a given point is on screen
	fn point_in_ndc(p : &Vec3) -> bool {
		p.xy()
			.to_array()
			.iter()
			.any(|f : &f32| -> bool { *f < -1_f32 || *f > 1_f32 })
			.not()
	}

	//A triangle is on screen if any of its points are in NDC range and
	fn is_tri_visible(t : &Triangle<V>) -> bool { !false }

	// Draw a single triangle to the
	// frame_buffer
	fn raster_tri(
		self: &mut Renderer<V, TE, P, CE>,
		tri : &Triangle<V>,
	) -> () {
		//TODO: make this not clip triangles that are in view but have all 3 points out of NDC
		/*
		if !Renderer::<V, TE, P, CE>::is_tri_visible(tri) {
			println!("TRI CULLED!");
			return;
		}

		//let transformed_verts : [VertexTransformerOut<P>; 3] = [

		let mut x_sorted : [&V; 3] = tri.0.each_ref();

		x_sorted.sort_by(|a : &&Vec4, b : &&Vec4| -> std::cmp::Ordering {
			a.x.total_cmp(&b.x)
		});

		let mut y_sorted : [&Vec4; 3] = tri.0.each_ref();

		y_sorted.sort_by(|a : &&Vec4, b : &&Vec4| -> std::cmp::Ordering {
			b.y.total_cmp(&a.y)
		});

		// Screen coordinate of scanline bounds
		let top_y : i32 = self.ndy_to_screen_y(y_sorted[0].y);

		let mid_y : i32 = self.ndy_to_screen_y(y_sorted[1].y);

		let bot_y : i32 = self.ndy_to_screen_y(y_sorted[2].y);

		// slice of bounds so the two iterations
		// can be under one loop
		let bounds : [i32; 3] = [top_y, mid_y, bot_y];

		for i in 0..=1_usize {
			let initial_y : i32 = bounds[i];

			let final_y : i32 = if top_y != mid_y {
				bounds[i + 1]
			} else {
				bot_y
			};

			//Prevents near invisible triangles that are drawn as long lines accross the entire
			//screen
			if initial_y == final_y {
				break;
			}

			// Iterate over lines of triangle - clamped to height for the **PERF**
			for y in initial_y.clamp(0, self.height() as i32 - 1)
				..=final_y.clamp(0, self.height() as i32 - 1)
			{
				let t : f32 = (y - initial_y) as f32 / (final_y - initial_y) as f32;

				// We can easily find the y coordinate
				// from the side formed by 2 lines
				let mut lef_x : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[i].x, y_sorted[i + 1].x, t);

				let mut lef_z : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[i].z, y_sorted[i + 1].z, t);

				let t : f32 = (y - top_y) as f32 / (bot_y - top_y) as f32;

				let mut rig_x : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[0].x, y_sorted[2].x, t);

				let mut rig_z : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[0].z, y_sorted[2].z, t);

				if lef_x > rig_x {
					std::mem::swap(&mut lef_x, &mut rig_x);
					std::mem::swap(&mut lef_z, &mut rig_z);
				}

				let lef_x : i32 = self.ndx_to_screen_x(lef_x);

				let rig_x : i32 = self.ndx_to_screen_x(rig_x);

				//Iterate over each horizontal pixel - also clamped for perf and to prevent drawing
				//in the next scan line
				for x in lef_x.clamp(0, self.width() as i32 - 1)
					..=rig_x.clamp(0, self.width() as i32 - 1)
				{
					//PER PIXEL OPERATIONS HERE! :D

					//Horizontal interp
					let t : f32 =
						((x as i32) - (lef_x as i32)) as f32 / (rig_x - lef_x) as f32;

					let z : f32 = 1.0 / ((1.0 / lef_z) * (1.0 - t) + (1.0 / rig_z) * t);

					let pixel_fb_idx : usize = usize::min(
						(y * self.width() as i32 + x) as usize,
						self.frame_buffer.len() - 1,
					);

					let fill : Pixel = Pixel::new(0.25, 0.0, 0.75, 1.0);

					if z < self.depth_buffer[pixel_fb_idx] && z > self.camera.near_plane {
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
		*/
	}

	pub fn draw(self: &mut Renderer<V, TE, P, CE>) -> () {
		// Raster all triangles
		self
			.frame_buffer
			.fill(self.renderer_settings.background_col);

		self.depth_buffer.fill(f32::MAX);

		let proj_cam_mat : Mat4 = self.camera.proj_mat * self.camera.camera_mat;

		self
			.meshes
			.clone()
			.into_iter()
			.for_each(|m : Mesh<V, TE, P, CE>| -> () {
				let proj_cam_model_mat : Mat4 = proj_cam_mat * m.model_mat;

				m.tris.iter().for_each(|t : &Triangle<V>| -> () {
					self.raster_tri(&t);
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
			width : 1920 / 2,
			height : 1080 / 2,
			background_col : Pixel::new(0.5, 0.75, 0.9, 0.5),
			show_tri_div : true,
		}
	}
}
