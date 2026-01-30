// Number of bits per pixel
const PIXEL_WIDTH : u32 = u8::BITS * std::mem::size_of::<XPixel>() as u32;

use std::ffi::c_char;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};

use x11::xlib::{Display as XDisplay, GC as XGC, Window as XWindow, *};

use crate::pixel::Pixel;
use crate::renderer::Renderer;
use crate::window_system::WindowSystem;

pub type XPixel = [u8; 4];

pub fn pixel_to_x_pixel(p : Pixel) -> XPixel {
	[
		(p.b * (u8::MAX as f32)) as u8,
		(p.g * (u8::MAX as f32)) as u8,
		(p.r * (u8::MAX as f32)) as u8,
		(p.a * (u8::MAX as f32)) as u8,
	]
}

pub struct X11<'a> {
	// X11 Resources that need to be managed
	x_display : &'a mut XDisplay,
	x_bitmap : &'a mut XImage,
	x_gc : XGC,
	x_window : XWindow,
	// Framebuffer that's sent to X11 bitmap drawing, uses a different format
	// than the regular framebuffer
	x_framebuffer : Vec<XPixel>,
}

impl<'a> WindowSystem for X11<'a> {
	fn init(
		screen_size : (u32, u32),
		window_name : &str,
	) -> Result<X11<'a>, String> {
		// This is a bit ugly, but because of
		// the "declare and fill after" nature
		// of initializing the X11 stuff, this
		// was the best way I could fill in the
		// renderer's fields.

		let x_display : &mut XDisplay;

		let x_bitmap : &mut XImage;

		let x_gc : XGC;

		let x_window : XWindow;

		unsafe {
			// Initialize X11 server connection and
			// create X11 window

			// Open X display
			let p_x_display : *mut XDisplay = XOpenDisplay(null::<c_char>());

			x_display = p_x_display
				.as_mut()
				.ok_or(String::from("Failed to open X11 connection!"))?;

			// Create visual info and fill it with
			// XMatchVisualInfo
			let mut x_vi : MaybeUninit<XVisualInfo> = MaybeUninit::uninit();

			(*x_vi.as_mut_ptr()).visual =
				XDefaultVisual(x_display, XDefaultScreen(x_display));

			XMatchVisualInfo(
				x_display,
				XDefaultScreen(x_display),
				PIXEL_WIDTH as i32,
				TrueColor,
				x_vi.as_mut_ptr(),
			);

			let x_visual_info : XVisualInfo = x_vi.assume_init();

			// Create bit map that X will draw from
			let p_x_bitmap : *mut XImage = XCreateImage(
				x_display,
				XDefaultVisual(x_display, x_visual_info.screen),
				x_visual_info.depth as u32,
				ZPixmap,
				0,
				null_mut::<c_char>(),
				screen_size.0,
				screen_size.1,
				PIXEL_WIDTH as i32,
				0,
			);

			x_bitmap = p_x_bitmap
				.as_mut()
				.ok_or(String::from("Failed to create X11 image for framebuffer"))?;

			let mut x_swa : MaybeUninit<XSetWindowAttributes> = MaybeUninit::uninit();

			let x_cmap : Colormap = XCreateColormap(
				x_display,
				XDefaultRootWindow(x_display),
				x_visual_info.visual,
				AllocNone,
			);

			(*x_swa.as_mut_ptr()).colormap = x_cmap;

			(*x_swa.as_mut_ptr()).background_pixmap = 0;

			(*x_swa.as_mut_ptr()).border_pixel = 0;

			(*x_swa.as_mut_ptr()).event_mask = StructureNotifyMask as i64;

			(*x_swa.as_mut_ptr()).background_pixel = 0;

			let mut x_swa : XSetWindowAttributes = x_swa.assume_init();

			x_window = XCreateWindow(
				x_display,
				XDefaultRootWindow(x_display),
				0,
				0,
				screen_size.0,
				screen_size.1,
				0,
				x_visual_info.depth,
				InputOutput as u32,
				x_visual_info.visual,
				CWColormap | CWBorderPixel | CWBackPixel | CWEventMask,
				&mut x_swa,
			);

			XStoreName(x_display, x_window, c"CPU As A Rendeder BIATCH".as_ptr());

			x_gc = XCreateGC(x_display, x_window, 0, null_mut());

			XMapWindow(x_display, x_window);
		}

		let x_framebuffer : Vec<XPixel> =
			vec![[0, 0, 0, 0]; (screen_size.0 * screen_size.1) as usize];

		Ok(X11 {
			x_display,
			x_bitmap,
			x_gc,
			x_window,
			x_framebuffer,
		})
	}

	fn handle_events(
		self: &mut X11<'a>,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> () {
		unsafe {
			let mut next_event : MaybeUninit<XEvent> = MaybeUninit::uninit();

			while XCheckTypedWindowEvent(
				self.x_display,
				self.x_window,
				ConfigureNotify,
				next_event.as_mut_ptr(),
			) != 0
			{
				let configure : XConfigureEvent =
					*(next_event.as_ptr() as *const XConfigureEvent);

				let (w, h) : (u32, u32) =
					(configure.width as u32, configure.height as u32);

				if w != self.get_win_w() || h != self.get_win_h() {
					// Minimizes some flicker
					XClearWindow(self.x_display, self.x_window);

					// Update the frame buffer sizes
					rend_framebuffer.resize((w * h) as usize, Pixel::default());

					self
						.x_framebuffer
						.resize((w * h) as usize, pixel_to_x_pixel(Pixel::default()));

					self.x_bitmap.width = w as i32;

					self.x_bitmap.height = h as i32;

					self.x_bitmap.bytes_per_line =
						(std::mem::size_of::<XPixel>() as u32 * w) as i32;
				}
			}
		}
	}

	fn draw_frame(
		self: &mut X11<'a>,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> () {
		// Turn our framebuffer into one that
		// can be read by X11
		rend_framebuffer
			.iter()
			.cloned()
			.zip(self.x_framebuffer.iter_mut())
			.for_each(|(p, xp) : (Pixel, &mut XPixel)| -> () {
				*xp = pixel_to_x_pixel(p)
			});

		// Actually draw the frame
		self.x_bitmap.data = self.x_framebuffer.as_ptr() as *mut i8;

		unsafe {
			XPutImage(
				self.x_display,
				self.x_window,
				self.x_gc,
				self.x_bitmap,
				0,
				0,
				0,
				0,
				self.get_win_w(),
				self.get_win_h(),
			);
		}
	}

	fn get_win_w(self: &X11<'a>) -> u32 { self.x_bitmap.width as u32 }

	fn get_win_h(self: &X11<'a>) -> u32 { self.x_bitmap.height as u32 }
}

impl<'a> Drop for X11<'a> {
	fn drop(self: &mut X11<'a>) -> () {
		unsafe {
			// For whatever reason, destorying the
			// image also frees the memory its data
			// field uses. This somehow causes a
			// segfault even when said memory is not
			// used afterwards.
			self.x_bitmap.data = null_mut();

			XDestroyImage(self.x_bitmap);

			XFreeGC(self.x_display, self.x_gc);
		}
	}
}
