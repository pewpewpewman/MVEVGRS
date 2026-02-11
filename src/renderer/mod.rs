// Module that handles the main loop of
// drawing and rendering.

use crate::pixel::Pixel;
use crate::triangle::{Point, ScreenCoords, Triangle, lin_interp};
use crate::window_system::WindowSystem;

pub struct Renderer<T : WindowSystem> {
	// Windowing syste,
	windowing : T,
	// Main frame buffer that is written to
	pub framebuffer : Vec<Pixel>,
	// Go to value for filling the frame buffer
	pub background_col : Pixel,
	// Triangles to be rastered
	pub tris : Vec<Triangle>,
}

impl<T : WindowSystem> Renderer<T> {
	pub fn new() -> Result<Renderer<T>, String> {
		let win_w : u32 = 940;
		let win_h : u32 = 480;

		let windowing : T =
			<T as WindowSystem>::init((win_w, win_h), "MVEVGRS BITCH!")?;

		let background_col : Pixel = Pixel::new(0.0, 0.0, 0.0, 1.0);

		let framebuffer : Vec<Pixel> =
			vec![background_col.clone(); (win_w * win_h) as usize];

		let tris : Vec<Triangle> = Vec::new();

		Ok(Renderer {
			windowing,
			framebuffer,
			background_col,
			tris,
		})
	}

	// Helpful conversion functions between
	// NDC and pixel coordinates and vice
	// versa
	pub fn screen_x_to_ndx(
		self: &Renderer<T>,
		x : u32,
	) -> f32 {
		x as f32 / self.windowing.get_win_w() as f32 * 2_f32 - 1_f32
	}

	pub fn screen_y_to_ndy(
		self: &Renderer<T>,
		y : u32,
	) -> f32 {
		(1_f32 - (y as f32 / self.windowing.get_win_h() as f32)) * 2_f32 - 1_f32
	}

	pub fn screen_coords_to_ndc(
		self: &Renderer<T>,
		c : ScreenCoords,
	) -> Point {
		Point::new(self.screen_x_to_ndx(c.x), self.screen_y_to_ndy(c.y), 0_f32)
	}

	pub fn ndx_to_screen_x(
		self: &Renderer<T>,
		x : f32,
	) -> u32 {
		f32::round(self.windowing.get_win_w() as f32 * ((1_f32 + x) / 2_f32)) as u32
	}

	pub fn ndy_to_screen_y(
		self: &Renderer<T>,
		y : f32,
	) -> u32 {
		f32::round(
			self.windowing.get_win_h() as f32 * (1_f32 - ((1_f32 + y) / 2_f32)),
		) as u32
	}

	pub fn ndc_to_screen_coords(
		self: &Renderer<T>,
		p : &Point,
	) -> ScreenCoords {
		ScreenCoords::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	// Draw a single triangle to the
	// framebuffer
	fn raster_tri(
		self: &mut Renderer<T>,
		tri : &Triangle,
	) -> () {
		let mut x_sorted : [&Point; 3] = tri.points.each_ref();

		x_sorted.sort_by(|a : &&Point, b : &&Point| -> std::cmp::Ordering {
			a.x.total_cmp(&b.x)
		});

		let mut y_sorted : [&Point; 3] = tri.points.each_ref();

		y_sorted.sort_by(|a : &&Point, b : &&Point| -> std::cmp::Ordering {
			b.y.total_cmp(&a.y)
		});

		// Screen coordinate of scanline bounds
		let top_y : u32 = self.ndy_to_screen_y(y_sorted[0].y);

		let bot_y : u32 = self.ndy_to_screen_y(y_sorted[2].y);

		let mid_y : u32 = self.ndy_to_screen_y(y_sorted[1].y);

		// slice of bounds so the two iterations
		// can be under one loop
		let bounds : [u32; 3] = [top_y, mid_y, bot_y];

		// Values in NDC that are needed for
		// point slope later Highest Y value
		let y_t : f32 = y_sorted[0].y;

		// Lowest Y value
		let y_b : f32 = y_sorted[2].y;

		// X coord of upper Y point
		let x_t : f32 = y_sorted[0].x;

		// X coord of lower Y Point
		let x_b : f32 = y_sorted[2].x;

		for i in 0..=1 {
			let i : usize = i;

			let initial_y : u32 = bounds[i];
			let final_y : u32 = bounds[i + 1];

			if initial_y == final_y {
				break;
			}

			// Iterate over lines of triangle
			for y in initial_y..=final_y {
				let t : f32 = (y - initial_y) as f32 / (final_y - initial_y) as f32;

				// We can easily find the y coordinate
				// from the side formed by 2 lines
				let mut x1 : f32 = lin_interp(y_sorted[i].x, y_sorted[i + 1].x, t);

				let t : f32 = (y - top_y) as f32 / (bot_y - top_y) as f32;

				let mut x2 : f32 = lin_interp(y_sorted[0].x, y_sorted[2].x, t);

				if x1 > x2 {
					std::mem::swap(&mut x1, &mut x2);
				}

				let lef_x : usize = usize::min(
					(self.ndx_to_screen_x(x1) + (y * self.windowing.get_win_w()))
						as usize,
					self.framebuffer.len() - 1,
				);

				let rig_x : usize = usize::min(
					(self.ndx_to_screen_x(x2) + (y * self.windowing.get_win_w()))
						as usize,
					self.framebuffer.len() - 1,
				);

				self.framebuffer[lef_x..=rig_x].fill(if i == 0 {
					Pixel::new(0.0, 0.0, 1.0, 1.0)
				} else {
					Pixel::new(1.0, 0.0, 0.0, 1.0)
				});
			}
		}
	}

	// Raster all triangles
	fn raster(self: &mut Renderer<T>) -> () {
		self.framebuffer.fill(self.background_col.clone());

		self.tris.clone().iter().for_each(|t : &Triangle| -> () {
			self.raster_tri(t);
		});
	}

	pub fn step_frame(self: &mut Renderer<T>) -> () {
		// Before we get to drawing, we must
		// handle all X11 events
		self.windowing.handle_events(&mut self.framebuffer);

		// Raster triangles
		self.raster();

		self.windowing.draw_frame(&mut self.framebuffer);
	}
}
