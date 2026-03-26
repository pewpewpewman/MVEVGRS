use std::collections::HashSet;
use std::num::NonZeroU32;
use std::rc::Rc;

use glam::{Mat4, Vec3};
use softbuffer::{Buffer, Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::error::EventLoopError;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{
	ActiveEventLoop,
	ControlFlow,
	EventLoop,
	OwnedDisplayHandle,
};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window, WindowAttributes, WindowId};

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
	//must be behind an option
	window_state : Option<WindowState>,
	//Since the winit window events dont fire every frame a key is held, we use this instead
	keyboard_state : HashSet<KeyCode>,
}

impl<'a> WindowRenderTarget<'a> {
	pub fn new(
		source : &'a mut Renderer
	) -> Result<WindowRenderTarget<'a>, String> {
		let event_loop : EventLoop<()> = EventLoop::new().unwrap();

		event_loop.set_control_flow(ControlFlow::Poll);

		let mut ret : WindowRenderTarget = WindowRenderTarget {
			source,
			window_state : None,
			keyboard_state : HashSet::new(),
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
						.with_transparent(true),
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
				//Respond to user input
				let mut camera_pos_change : Vec3 = Vec3::ZERO;
				let mut camera_horiz_angle_change : f32 = 0_f32;
				let mut camera_vert_angle_change : f32 = 0_f32;

				let movement_amount : f32 = 0.01;

				self.keyboard_state.iter().for_each(|kc : &KeyCode| -> () {
					match kc {
						KeyCode::KeyW => {
							camera_pos_change.z -= movement_amount;
						},
						KeyCode::KeyA => {
							camera_pos_change.x += movement_amount;
						},
						KeyCode::KeyS => {
							camera_pos_change.z += movement_amount;
						},
						KeyCode::KeyD => {
							camera_pos_change.x -= movement_amount;
						},

						KeyCode::Space => {
							camera_pos_change.y -= movement_amount;
						},

						KeyCode::ShiftLeft => {
							camera_pos_change.y += movement_amount;
						},

						KeyCode::ArrowLeft => camera_horiz_angle_change -= movement_amount,

						KeyCode::ArrowRight => camera_horiz_angle_change += movement_amount,

						KeyCode::ArrowUp => camera_vert_angle_change += movement_amount,

						KeyCode::ArrowDown => camera_vert_angle_change -= movement_amount,

						_ => {},
					}
				});

				self.source.camera.camera_mat *=
					Mat4::from_translation(camera_pos_change);

				// * Mat4::from_rotation_y(camera_horiz_angle_change)
				//* Mat4::from_rotation_x(camera_vert_angle_change)

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

				let mut buffer : Buffer<OwnedDisplayHandle, Rc<Window>> = self
					.window_state
					.as_mut()
					.expect("Window should be inited by first draw request")
					.surface
					.buffer_mut()
					.expect("Buffer should be accessible");

				buffer
					.iter_mut()
					.zip(self.source.frame_buffer.iter())
					.for_each(|(u, p) : (&mut u32, &Pixel)| -> () {
						*u = ((p.x * u8::MAX as f32).round() as u32) << 16
							| ((p.y * u8::MAX as f32).round() as u32) << 8
							| ((p.z * u8::MAX as f32).round() as u32);
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

				match event {
					KeyEvent {
						physical_key: PhysicalKey::Code(KeyCode::F11),
						repeat: false,
						state: ElementState::Pressed,
						..
					} => {
						self.window_state.as_ref().unwrap().window.set_fullscreen(
							self
								.window_state
								.as_ref()
								.unwrap()
								.window
								.fullscreen()
								.map_or_else(
									|| -> Option<Fullscreen> {
										Some(Fullscreen::Borderless(None))
									},
									|_ : Fullscreen| -> Option<Fullscreen> { None },
								),
						);

						self.window_state.as_ref().unwrap().window.fullscreen();
					},
					KeyEvent {
						physical_key: PhysicalKey::Code(kc),
						state: ElementState::Pressed,
						repeat: false,
						..
					} => {
						self.keyboard_state.insert(kc);
					},
					KeyEvent {
						physical_key: PhysicalKey::Code(kc),
						state: ElementState::Released,
						..
					} => {
						self.keyboard_state.remove(&kc);
					},

					_ => {},
				}
			},

			WindowEvent::CloseRequested => {
				event_loop.exit();
			},

			_ => {},
		}
	}
}
