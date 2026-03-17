// Module that handles the main loop of
// drawing and rendering.

mod camera;

use camera::Camera;
use glam::{Mat4, UVec2, Vec3, Vec4, Vec4Swizzles};

use crate::pixel::Pixel;
use crate::triangle::Triangle;

pub struct Renderer {
	// Main frame buffer that is written to
	pub framebuffer : Vec<Pixel>,
	// Settings for how to draw things
	pub renderer_settings : RendererSettings,
	// Camera that holds the camera and projection matrix
	pub camera : Camera,
	// Triangles to be rastered
	pub tris : Vec<Triangle>,
	// Update function to run before drawing each frame
	update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
}

impl Renderer {
	pub fn new(
		renderer_settings : RendererSettings,
		tris : Vec<Triangle>,
		update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
	) -> Renderer {
		Renderer {
			framebuffer : vec![
				renderer_settings.background_col;
				(renderer_settings.width * renderer_settings.height)
					as usize
			],
			renderer_settings,
			camera : Camera::default(),
			tris,
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

	// Draw a single triangle to the
	// framebuffer
	fn raster_tri(
		self: &mut Renderer,
		tri : &Triangle,
	) -> () {
		let mut x_sorted : [&Vec3; 3] = tri.points.each_ref();

		x_sorted.sort_by(|a : &&Vec3, b : &&Vec3| -> std::cmp::Ordering {
			a.x.total_cmp(&b.x)
		});

		let mut y_sorted : [&Vec3; 3] = tri.points.each_ref();

		y_sorted.sort_by(|a : &&Vec3, b : &&Vec3| -> std::cmp::Ordering {
			b.y.total_cmp(&a.y)
		});

		// Screen coordinate of scanline bounds
		let top_y : u32 = self.ndy_to_screen_y(y_sorted[0].y);

		let bot_y : u32 = self.ndy_to_screen_y(y_sorted[2].y);

		let mid_y : u32 = self.ndy_to_screen_y(y_sorted[1].y);

		// slice of bounds so the two iterations
		// can be under one loop
		let bounds : [u32; 3] = [top_y, mid_y, bot_y];

		for i in 0..=1 {
			let i : usize = i;

			let initial_y : u32 = bounds[i];
			let final_y : u32 = if top_y != mid_y {
				bounds[i + 1]
			} else {
				bot_y
			};

			if initial_y == final_y {
				break;
			}

			// Iterate over lines of triangle
			for y in initial_y..=final_y {
				let t : f32 = (y - initial_y) as f32 / (final_y - initial_y) as f32;

				// We can easily find the y coordinate
				// from the side formed by 2 lines
				let mut x1 : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[i].x, y_sorted[i + 1].x, t);

				let t : f32 = (y - top_y) as f32 / (bot_y - top_y) as f32;

				let mut x2 : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[0].x, y_sorted[2].x, t);

				if x1 > x2 {
					std::mem::swap(&mut x1, &mut x2);
				}

				let lef_x : usize = usize::min(
					(y * self.width() + u32::min(self.ndx_to_screen_x(x1), self.width()))
						as usize,
					self.framebuffer.len() - 1,
				);

				let rig_x : usize = usize::min(
					(y * self.width() + u32::min(self.ndx_to_screen_x(x2), self.width()))
						as usize,
					self.framebuffer.len() - 1,
				);

				if !self.renderer_settings.show_tri_div {
					self.framebuffer[lef_x..=rig_x].fill(Pixel::new(0.0, 0.0, 1.0, 1.0));
				} else {
					self.framebuffer[lef_x..=rig_x].fill(if i == 0 {
						Pixel::new(0.0, 0.0, 1.0, 1.0)
					} else {
						Pixel::new(0.0, 1.0, 0.0, 1.0)
					});
				}
			}
		}
	}

	// Raster all triangles
	fn raster(self: &mut Renderer) -> () {
		self.framebuffer.fill(self.renderer_settings.background_col);

		let cam_proj_mat : Mat4 = self.camera.proj_mat * self.camera.camera_mat;

		self.tris.clone().iter().for_each(|t : &Triangle| -> () {
			self.raster_tri(&Triangle::new(
				(cam_proj_mat * Vec4::from((t.points[0], 1_f32))).xyz(),
				(cam_proj_mat * Vec4::from((t.points[1], 1_f32))).xyz(),
				(cam_proj_mat * Vec4::from((t.points[2], 1_f32))).xyz(),
			));
		});
	}

	pub fn frame_step(self: &mut Renderer) -> () {
		self.raster();

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
			width : 500,
			height : 500,
			background_col : Pixel::new(1.0, 0.5, 0.75, 1.0),
			show_tri_div : false,
		}
	}
}
