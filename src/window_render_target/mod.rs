use std::num::NonZeroU32;
use std::rc::Rc;

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
use crate::renderer::Renderer;

struct WindowState {
	window : Rc<Window>,
	surface : Surface<OwnedDisplayHandle, Rc<Window>>,
}

impl WindowState {
	fn new(
		window : Rc<Window>,
		surface : Surface<OwnedDisplayHandle, Rc<Window>>,
	) -> WindowState {
		WindowState {
			window,
			surface,
		}
	}
}

pub struct WindowRenderTarget<'a> {
	source : &'a mut Renderer,
	//Internal windowing systems need to be inited by winit's application handler callback, so it
	//must be made an option
	window_state : Option<WindowState>,
}

impl<'a> WindowRenderTarget<'a> {
	pub fn new(source : &'a mut Renderer) -> Result<WindowRenderTarget, String> {
		let event_loop : EventLoop<()> = EventLoop::new().unwrap();

		event_loop.set_control_flow(ControlFlow::Poll);

		let mut ret : WindowRenderTarget = WindowRenderTarget {
			source,
			window_state : None,
		};

		event_loop
			.run_app(&mut ret)
			.map_err(|e : EventLoopError| -> String { e.to_string() })?;

		Ok(ret)
	}
}

impl<'a> ApplicationHandler for WindowRenderTarget<'a> {
	fn resumed(
		self: &mut WindowRenderTarget<'a>,
		event_loop : &ActiveEventLoop,
	) -> () {
		//Initialize the windowstate now that we have the event loop do the window creating
		let window : Rc<Window> = Rc::<Window>::new(
			event_loop
				.create_window(
					WindowAttributes::default()
						.with_inner_size(Size::Physical(PhysicalSize::new(
							self.source.width(),
							self.source.height(),
						)))
						.with_title(String::from("MVEVGS BIATCH!!"))
						.with_window_icon(Some({
							use image::error::ImageError;
							use image::RgbaImage;

							let (rgba, width, height) : (Vec<u8>, u32, u32) = {
								let image : RgbaImage = image::load_from_memory(
									include_bytes!("../../scaled_icon.png"),
								)
								.expect("Icon should be loadable!")
								.into_rgba8();

								let (width, height) : (u32, u32) = image.dimensions();

								let rgba : Vec<u8> = image.into_raw();
								(rgba, width, height)
							};

							Icon::from_rgba(rgba, width, height)
								.expect("Icon should be creatable!")
						}))
						.with_transparent(true)
						.with_visible(false),
				)
				.expect("Window creation should be unfailable!"),
		);

		// Softbuffer context
		let context : Context<OwnedDisplayHandle> =
			Context::new(event_loop.owned_display_handle()).unwrap();

		// Softbuffer surface pixels are written
		// to
		let surface : Surface<OwnedDisplayHandle, Rc<Window>> =
			Surface::new(&context, Rc::clone(&window))
				.expect("Surface creation should be unfailable!");

		self.window_state = Some(WindowState::new(window, surface));
	}

	fn window_event(
		self: &mut WindowRenderTarget<'a>,
		event_loop : &ActiveEventLoop,
		_id : WindowId,
		event : WindowEvent,
	) -> () {
		match event {
			WindowEvent::RedrawRequested => {
				//Advanced render update function and have it draw to its internal frame buffer
				self.source.frame_step();

				// Correct internal surface size
				self
					.window_state
					.as_mut()
					.expect("Window should be inited by first draw request")
					.surface
					.resize(
						NonZeroU32::new(self.source.width())
							.expect("Width should be non-zero"),
						NonZeroU32::new(self.source.height())
							.expect("Height should be non-zero"),
					)
					.expect("Surface should be resizable");

				/*
				self
					.
					.framebuffer
					.resize((self.source.width() * self.source.height()) as usize, self.source.render_settings.background_col);  */

				let mut buffer : Buffer<OwnedDisplayHandle, Rc<Window>> = self
					.window_state
					.as_mut()
					.expect("Window should be inited by first draw request")
					.surface
					.buffer_mut()
					.expect("Buffer should be accessible");

				/*
				assert_eq!(
					buffer.len(),
					self.source.framebuffer.len(),
					"Softbuffer buffer and renderer's framebuffer must be the same size!",
				);
				*/

				buffer
					.iter_mut()
					.zip(self.source.framebuffer.iter())
					.for_each(|(u, p) : (&mut u32, &Pixel)| -> () {
						*u = ((p.r * u8::MAX as f32).round() as u32) << 16
							| ((p.g * u8::MAX as f32).round() as u32) << 8
							| ((p.b * u8::MAX as f32).round() as u32);
					});

				buffer.present().expect("Buffer presenting should not fail");

				self
					.window_state
					.as_mut()
					.expect("Renderer should have valid window state by now")
					.window
					.request_redraw();
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
					self.window_state.as_ref().unwrap().window.set_fullscreen(
						self
							.window_state
							.as_ref()
							.unwrap()
							.window
							.fullscreen()
							.map_or_else(
								|| -> Option<Fullscreen> { Some(Fullscreen::Borderless(None)) },
								|_ : Fullscreen| -> Option<Fullscreen> { None },
							),
					);

					self.window_state.as_ref().unwrap().window.fullscreen();
				}
			},

			WindowEvent::CloseRequested => {
				event_loop.exit();
			},

			_ => {},
		}
	}
}
