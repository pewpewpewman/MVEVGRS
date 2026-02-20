// Module that handles the main loop of
// drawing and rendering.

use std::cmp::Ordering;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use softbuffer::{Buffer, Context, SoftBufferError, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::error::{EventLoopError, OsError};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{
	ActiveEventLoop,
	ControlFlow,
	EventLoop,
	OwnedDisplayHandle,
};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::monitor::VideoModeHandle;
use winit::window::{
	BadIcon,
	Fullscreen,
	Icon,
	Window,
	WindowAttributes,
	WindowId,
};

use crate::pixel::Pixel;
use crate::triangle::{Point, ScreenCoord, Triangle};

pub struct Renderer {
	// Winit window to draw to - could prolly be done with just a refernce and
	// lifetimes but bro i dont wanna write all of the life time annotations
	// :sob:
	window : Rc<Window>,
	// Suface that pixel data is written to
	surface : Surface<OwnedDisplayHandle, Rc<Window>>,
	// Main frame buffer that is written to
	pub framebuffer : Vec<Pixel>,
	// Go to value for filling the frame buffer
	pub background_col : Pixel,
	// Triangles to be rastered
	pub tris : Vec<Triangle>,
	// Update function to run before drawing each frame
	pub update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
}

impl Renderer {
	pub fn run<F : FnOnce(&mut Renderer) -> ()>(
		init_fn : F,
		update_fn : Option<Box<dyn FnMut(&mut Renderer) -> ()>>,
	) -> Result<(), String> {
		let win_w : u32 = 1024;
		let win_h : u32 = 768;

		let background_col : Pixel = Pixel::new(1.0, 0.0, 0.0, 1.0);

		let framebuffer : Vec<Pixel> =
			vec![background_col; (win_w * win_h) as usize];

		let tris : Vec<Triangle> = Vec::new();

		let event_loop : EventLoop<()> = EventLoop::new()
			.map_err(|e : EventLoopError| -> String { e.to_string() })?;

		event_loop.set_control_flow(ControlFlow::Poll);

		// Winit docs says making the window
		// from the EventLoop (as opposeed to
		// the ActiveEventLoop) is bad and
		// depricated, however I don't want the
		// Renderer to redundantly store things
		// the winit window is already
		// tracking so the only way to
		// initialize those values is applying
		// them in the Renderer's contructor
		let window : Rc<Window> = Rc::new(
			event_loop
				.create_window(
					WindowAttributes::default()
						.with_inner_size(Size::Physical(PhysicalSize::new(win_w, win_h)))
						.with_title(String::from("MVEVGS BIATCH!!"))
						.with_window_icon(Some({
							use image::error::ImageError;
							use image::RgbaImage;

							let (rgba, width, height) : (Vec<u8>, u32, u32) = {
								let image : RgbaImage =
									image::load_from_memory(include_bytes!("../../icon.png"))
										.map_err(|e : ImageError| -> String { e.to_string() })?
										.into_rgba8();

								let (width, height) : (u32, u32) = image.dimensions();

								let rgba : Vec<u8> = image.into_raw();
								dbg!(rgba.len());
								(rgba, width, height)
							};

							Icon::from_rgba(rgba, width, height)
								.map_err(|e : BadIcon| -> String { e.to_string() })?
						})),
				)
				.map_err(|e : OsError| -> String { e.to_string() })?,
		);

		// Softbuffer context
		let context : Context<OwnedDisplayHandle> =
			Context::new(event_loop.owned_display_handle())
				.map_err(|e : SoftBufferError| -> String { e.to_string() })?;

		// Softbuffer surface pixels are written
		// to
		let surface : Surface<OwnedDisplayHandle, Rc<Window>> =
			Surface::new(&context, window.clone())
				.map_err(|e : SoftBufferError| -> String { e.to_string() })?;

		let mut renderer : Renderer = Renderer {
			surface,
			window,
			framebuffer,
			background_col,
			tris,
			update_fn,
		};

		(init_fn)(&mut renderer);

		event_loop
			.run_app(&mut renderer)
			.map_err(|e : EventLoopError| -> String { e.to_string() })
	}

	pub fn width(self: &Renderer) -> u32 { self.window.inner_size().width }

	pub fn height(self: &Renderer) -> u32 { self.window.inner_size().height }

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
		c : ScreenCoord,
	) -> Point {
		Point::new(self.screen_x_to_ndx(c.x), self.screen_y_to_ndy(c.y), 0_f32)
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
		p : &Point,
	) -> ScreenCoord {
		ScreenCoord::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	// Draw a single triangle to the
	// framebuffer
	fn raster_tri(
		self: &mut Renderer,
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
				let mut x1 : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[i].x, y_sorted[i + 1].x, t);

				let t : f32 = (y - top_y) as f32 / (bot_y - top_y) as f32;

				let mut x2 : f32 =
					<f32 as glam::FloatExt>::lerp(y_sorted[0].x, y_sorted[2].x, t);

				if x1 > x2 {
					std::mem::swap(&mut x1, &mut x2);
				}

				let lef_x : usize = usize::min(
					(self.ndx_to_screen_x(x1) + (y * self.width())) as usize,
					self.framebuffer.len() - 1,
				);

				let rig_x : usize = usize::min(
					(self.ndx_to_screen_x(x2) + (y * self.width())) as usize,
					self.framebuffer.len() - 1,
				);

				self.framebuffer[lef_x..=rig_x].fill(Pixel::new(0.0, 0.0, 1.0, 1.0));
				//self.framebuffer[lef_x..=rig_x].fill(if i == 0 {
				//	Pixel::new(0.0, 0.0, 1.0, 1.0)
				//} else {
				//	Pixel::new(0.0, 1.0, 0.0, 1.0)
				//});
			}
		}
	}

	// Raster all triangles
	fn raster(self: &mut Renderer) -> () {
		self.framebuffer.fill(self.background_col);

		self.tris.clone().iter().for_each(|t : &Triangle| -> () {
			self.raster_tri(t);
		});
	}
}

// Make renderer make good use of winit
impl ApplicationHandler for Renderer {
	fn resumed(
		self: &mut Renderer,
		_event_loop : &ActiveEventLoop,
	) -> () {
	}

	fn window_event(
		self: &mut Renderer,
		event_loop : &ActiveEventLoop,
		_id : WindowId,
		event : WindowEvent,
	) -> () {
		match event {
			WindowEvent::RedrawRequested => {
				//Perform user written update function
				//Borrow checker gets mad at me for accessing the function
				//field and passing it the mut ref so we gotta do some trickery
				let mut temp : Option<Box<dyn FnMut(&mut Renderer) -> ()>> =
					self.update_fn.take();
				//It is possible the user didn't even provide an update fn
				if let Some(update_fn) = &mut temp {
					let update_fn : &mut Box<dyn FnMut(&mut Renderer) -> ()> = update_fn;

					(update_fn)(self);
				}
				self.update_fn = temp;

				// Correct internal surface size
				self
					.surface
					.resize(
						NonZeroU32::new(self.width()).expect("Width should be non-zero"),
						NonZeroU32::new(self.height()).expect("Height should be non-zero"),
					)
					.expect("Surface should be resizable");

				self
					.framebuffer
					.resize((self.width() * self.height()) as usize, self.background_col);

				self.raster();

				let mut buffer : Buffer<OwnedDisplayHandle, Rc<Window>> = self
					.surface
					.buffer_mut()
					.expect("Buffer should be accessible");

				assert_eq!(
					buffer.len(),
					self.framebuffer.len(),
					"Softbuffer buffer and renderer's framebuffer must be the same size!",
				);

				buffer.iter_mut().zip(self.framebuffer.iter()).for_each(
					|(u, p) : (&mut u32, &Pixel)| -> () {
						*u = ((p.r * u8::MAX as f32).round() as u32) << 16
							| ((p.g * u8::MAX as f32).round() as u32) << 8
							| ((p.b * u8::MAX as f32).round() as u32)
					},
				);

				buffer.present().expect("Buffer presenting should bot fail");

				self.window.request_redraw();
			},

			WindowEvent::KeyboardInput {
				event,
				..
			} => {
				let event : KeyEvent = event;

				if event.physical_key == PhysicalKey::Code(KeyCode::F11)
					&& !event.repeat
					&& event.state == ElementState::Pressed
				{
					self
						.window
						.set_fullscreen(self.window.fullscreen().map_or_else(
							|| -> Option<Fullscreen> { Some(Fullscreen::Borderless(None)) },
							|_ : Fullscreen| -> Option<Fullscreen> { None },
						));

					self.window.fullscreen();
				}
			},

			WindowEvent::CloseRequested => {
				event_loop.exit();
			},
			_ => {},
		}
	}
}
