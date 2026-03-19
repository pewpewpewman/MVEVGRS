// Module that handles the main loop of
// drawing and rendering.

mod camera;

use std::ops::Not;

use camera::Camera;
use glam::{Mat4, UVec2, Vec3, Vec3Swizzles, Vec4};

use crate::mesh::{Mesh, Triangle};
use crate::pixel::Pixel;

pub struct Renderer {
	// Main frame buffer that is written to
	pub frame_buffer : Vec<Pixel>,
	// Depth buffer that is used for knowing what tris are visible
	pub depth_buffer : Vec<f32>,
	// Settings for how to draw things
	pub renderer_settings : RendererSettings,
	// Camera that holds the camera and projection matrix
	pub camera : Camera,
	// Triangles to be rastered
	pub meshes : Vec<Mesh>,
	// Update function to run before drawing each frame
	update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
}

impl Renderer {
	pub fn new(
		renderer_settings : RendererSettings,
		meshes : Vec<Mesh>,
		update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
	) -> Renderer {
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

	pub fn width(self: &Renderer) -> u32 { self.renderer_settings.width }

	pub fn height(self: &Renderer) -> u32 { self.renderer_settings.height }

	// Helpful conversion functions between
	// NDC and pixel coordinates and vice
	// versa
	pub fn screen_x_to_ndx(
		self: &Renderer,
		x : u32,
	) -> f32 {
		x as f32 / self.width() as f32 * 2_f32 - 1_f32
	}

	pub fn screen_y_to_ndy(
		self: &Renderer,
		y : u32,
	) -> f32 {
		(1_f32 - (y as f32 / self.height() as f32)) * 2_f32 - 1_f32
	}

	pub fn screen_coords_to_ndc(
		self: &Renderer,
		c : UVec2,
	) -> Vec3 {
		Vec3::new(self.screen_x_to_ndx(c.x), self.screen_y_to_ndy(c.y), 0_f32)
	}

	pub fn ndx_to_screen_x(
		self: &Renderer,
		x : f32,
	) -> u32 {
		f32::round(self.width() as f32 * ((1_f32 + x) / 2_f32)) as u32
	}

	pub fn ndy_to_screen_y(
		self: &Renderer,
		y : f32,
	) -> u32 {
		f32::round(self.height() as f32 * (1_f32 - ((1_f32 + y) / 2_f32))) as u32
	}

	pub fn ndc_to_screen_coords(
		self: &Renderer,
		p : &Vec3,
	) -> UVec2 {
		UVec2::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	//Test to see if a given point is on screen
	fn point_in_ndc(p : &Vec3) -> bool {
		p.xy()
			.to_array()
			.iter()
			.any(|f : &f32| -> bool { *f < -1_f32 || *f > 1_f32 })
			.not()
	}

	fn tri_in_ndc(t : &Triangle) -> bool {
		t.0.iter().any(Renderer::point_in_ndc)
	}

	// Draw a single triangle to the
	// frame_buffer
	fn raster_tri(
		self: &mut Renderer,
		tri : &Triangle,
	) -> () {
		if !Renderer::tri_in_ndc(tri) {
			return;
		}

		let mut x_sorted : [&Vec3; 3] = tri.0.each_ref();

		x_sorted.sort_by(|a : &&Vec3, b : &&Vec3| -> std::cmp::Ordering {
			a.x.total_cmp(&b.x)
		});

		let mut y_sorted : [&Vec3; 3] = tri.0.each_ref();

		y_sorted.sort_by(|a : &&Vec3, b : &&Vec3| -> std::cmp::Ordering {
			b.y.total_cmp(&a.y)
		});

		// Screen coordinate of scanline bounds
		let top_y : u32 =
			u32::clamp(self.ndy_to_screen_y(y_sorted[0].y), 0, self.height());

		let bot_y : u32 =
			u32::clamp(self.ndy_to_screen_y(y_sorted[2].y), 0, self.height());

		let mid_y : u32 =
			u32::clamp(self.ndy_to_screen_y(y_sorted[1].y), 0, self.height());

		// slice of bounds so the two iterations
		// can be under one loop
		let bounds : [u32; 3] = [top_y, mid_y, bot_y];

		for i in 0..=1_usize {
			let initial_y : u32 = bounds[i];

			let final_y : u32 = if top_y != mid_y {
				bounds[i + 1]
			} else {
				bot_y
			};

			//Prevents near invisible triangles that are drawn as long lines accross the entire
			//screen
			if initial_y == final_y {
				break;
			}

			// Iterate over lines of triangle
			for y in initial_y..=final_y {
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

				let lef_x : u32 =
					u32::clamp(self.ndx_to_screen_x(lef_x), 0, self.width());

				let rig_x : u32 =
					u32::clamp(self.ndx_to_screen_x(rig_x), 0, self.width());

				for x in lef_x..=rig_x {
					//PER PIXEL OPERATIONS HERE! :D

					//Horizontal interp
					let t : f32 = (x - lef_x) as f32 / (rig_x - lef_x) as f32;

					let z : f32 = 1.0
						/ <f32 as glam::FloatExt>::lerp((1.0 / lef_z), (1.0 / rig_z), t);

					let pixel_fb_idx : usize = usize::min(
						(y * self.width() + u32::min(x, self.width())) as usize,
						self.frame_buffer.len() - 1,
					);

					if z <= self.depth_buffer[pixel_fb_idx] {
						self.frame_buffer[pixel_fb_idx] =
							if !self.renderer_settings.show_tri_div {
								Pixel::new(z, 0.0, 0.0, 1.0)
							} else {
								if i == 0 {
									Pixel::new(0.0, 0.0, 1.0, 1.0)
								} else {
									Pixel::new(0.0, 1.0, 0.0, 1.0)
								}
							};

						self.depth_buffer[pixel_fb_idx] = z;
					}
				}
			}
		}
	}

	pub fn frame_step(self: &mut Renderer) -> () {
		// Raster all triangles
		self
			.frame_buffer
			.fill(self.renderer_settings.background_col);

		self.depth_buffer.fill(f32::MAX);

		let proj_cam_mat : Mat4 = self.camera.proj_mat * self.camera.camera_mat;

		self.meshes.clone().into_iter().for_each(|m : Mesh| -> () {
			let proj_cam_model_mat : Mat4 = proj_cam_mat * m.model_mat;

			m.tris.iter().for_each(|t : &Triangle| -> () {
				self.raster_tri(&Triangle::new(
					proj_cam_model_mat.project_point3(t.0[0]),
					proj_cam_model_mat.project_point3(t.0[1]),
					proj_cam_model_mat.project_point3(t.0[2]),
				));
			});
		});

		//Calling a function that acts on its own struct causes some borrow checker problems, let's
		//do some shenanigans to please it
		let mut temp : Option<Box<dyn FnMut(&mut Renderer) -> ()>> =
			self.update_fn.take();

		if let Some(f) = &mut temp {
			let f : &mut Box<dyn FnMut(&mut Renderer) -> ()> = f;
			(f)(self);
		}

		self.update_fn = temp;
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
			width : 720,
			height : 480,
			background_col : Pixel::new(1.0, 0.5, 0.75, 1.0),
			show_tri_div : true,
		}
	}
}
