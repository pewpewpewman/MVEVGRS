// Most of this is copied from https://github.com/Smithay/wayland-rs/blob/master/wayland-client/examples/simple_window.rs

use std::fs::File;
use std::io::BufWriter;
use std::os::fd::AsFd;
use std::sync::Arc;

use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::{
	wl_buffer,
	wl_compositor,
	wl_keyboard,
	wl_registry,
	wl_seat,
	wl_shm,
	wl_shm_pool,
	wl_surface,
};
use wayland_client::{
	delegate_noop,
	Connection,
	Dispatch,
	EventQueue,
	QueueHandle,
	WEnum,
};
use wayland_protocols::xdg::shell::client::xdg_surface;
use wayland_protocols::xdg::shell::client::xdg_surface::XdgSurface;
use wayland_protocols::xdg::shell::client::xdg_toplevel::{self, XdgToplevel};
use wayland_protocols::xdg::shell::client::xdg_wm_base::{self, XdgWmBase};

use crate::pixel::Pixel;
use crate::window_system::WindowSystem;

pub struct Wayland {
	connection : Connection,
	event_queue : Arc<EventQueue<Wayland>>,
	base_surface : Option<WlSurface>,
	buffer : Option<WlBuffer>,
	wm_base : Option<XdgWmBase>,
	xdg_surface : Option<(XdgSurface, XdgToplevel)>,
	configured : bool,
}

// Global registry events
impl Dispatch<WlRegistry, ()> for Wayland {
	fn event(
		state : &mut Wayland,
		registry : &wl_registry::WlRegistry,
		event : wl_registry::Event,
		_ : &(),
		_ : &Connection,
		qh : &QueueHandle<Wayland>,
	) -> () {
		if let wl_registry::Event::Global {
			name,
			interface,
			..
		} = event
		{
			let name : u32 = name;
			let interface : String = interface;

			match &*interface {
				"wl_compositor" => {
					let compositor : WlCompositor =
						registry.bind::<WlCompositor, (), Wayland>(name, 1, qh, ());

					let surface : WlSurface = compositor.create_surface(qh, ());

					state.base_surface = Some(surface);

					if state.wm_base.is_some() && state.xdg_surface.is_none() {
						state.init_xdg_surface(qh);
					}
				},

				"wl_shm" => {
					let shm =
						registry.bind::<wl_shm::WlShm, (), Wayland>(name, 1, qh, ());

					let (init_w, init_h) = (320, 240);

					let mut file : File = tempfile::tempfile().unwrap();

					draw(&mut file, (init_w, init_h));

					let pool : WlShmPool =
						shm.create_pool(file.as_fd(), (init_w * init_h * 4) as i32, qh, ());

					let buffer : WlBuffer = pool.create_buffer(
						0,
						init_w as i32,
						init_h as i32,
						(init_w * 4) as i32,
						wl_shm::Format::Argb8888,
						qh,
						(),
					);

					state.buffer = Some(buffer.clone());

					if state.configured {
						let surface : &WlSurface = state.base_surface.as_ref().unwrap();
						surface.attach(Some(&buffer), 0, 0);
						surface.commit();
					}
				},

				"wl_seat" => {
					registry.bind::<WlSeat, (), Wayland>(name, 1, qh, ());
				},

				"xdg_wm_base" => {
					let wm_base : XdgWmBase = registry
						.bind::<xdg_wm_base::XdgWmBase, (), Wayland>(name, 1, qh, ());
					state.wm_base = Some(wm_base);

					if state.base_surface.is_some() && state.xdg_surface.is_none() {
						state.init_xdg_surface(qh);
					}
				},

				_ => {},
			}
		}
	}
}

impl Dispatch<XdgWmBase, ()> for Wayland {
	fn event(
		_ : &mut Wayland,
		wm_base : &XdgWmBase,
		event : xdg_wm_base::Event,
		_ : &(),
		_ : &Connection,
		_ : &QueueHandle<Wayland>,
	) -> () {
		if let xdg_wm_base::Event::Ping {
			serial,
		} = event
		{
			let serial : u32 = serial;
			wm_base.pong(serial);
		}
	}
}

impl Dispatch<XdgSurface, ()> for Wayland {
	fn event(
		state : &mut Wayland,
		xdg_surface : &XdgSurface,
		event : xdg_surface::Event,
		_ : &(),
		_ : &Connection,
		_ : &QueueHandle<Wayland>,
	) -> () {
		if let xdg_surface::Event::Configure {
			serial,
			..
		} = event
		{
			let serial : u32 = serial;

			xdg_surface.ack_configure(serial);
			state.configured = true;
			let surface : &WlSurface = state.base_surface.as_ref().unwrap();
			if let Some(buffer) = &state.buffer {
				surface.attach(Some(buffer), 0, 0);
				surface.commit();
			}
		}
	}
}

impl Dispatch<XdgToplevel, ()> for Wayland {
	fn event(
		state : &mut Wayland,
		_ : &XdgToplevel,
		event : xdg_toplevel::Event,
		_ : &(),
		_ : &Connection,
		_ : &QueueHandle<Wayland>,
	) -> () {
		if let xdg_toplevel::Event::Close = event {
			// TODO: state.running = false;
		}
	}
}

impl Dispatch<WlSeat, ()> for Wayland {
	fn event(
		_ : &mut Wayland,
		seat : &wl_seat::WlSeat,
		event : wl_seat::Event,
		_ : &(),
		_ : &Connection,
		qh : &QueueHandle<Wayland>,
	) -> () {
		if let wl_seat::Event::Capabilities {
			capabilities: WEnum::Value(capabilities),
		} = event
		{
			let capabilities : wl_seat::Capability = capabilities;
			if capabilities.contains(wl_seat::Capability::Keyboard) {
				// seat.get_keyboard(qh, ());
				// TODO: Keyboard response
			}
		}
	}
}

// Ignore events from these object types
// in this example.
delegate_noop!(Wayland: ignore wl_compositor::WlCompositor);
delegate_noop!(Wayland: ignore wl_surface::WlSurface);
delegate_noop!(Wayland: ignore wl_shm::WlShm);
delegate_noop!(Wayland: ignore wl_shm_pool::WlShmPool);
delegate_noop!(Wayland: ignore wl_buffer::WlBuffer);

fn draw(
	tmp : &mut File,
	(buf_x, buf_y) : (u32, u32),
) {
	use std::cmp::min;
	use std::io::Write;
	let mut buf : BufWriter<&mut File> = std::io::BufWriter::new(tmp);
	for y in 0..buf_y {
		for x in 0..buf_x {
			let a = 0xFF;
			let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
			let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
			let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
			buf
				.write_all(&[b as u8, g as u8, r as u8, a as u8])
				.unwrap();
		}
	}
	buf.flush().unwrap();
}

impl Wayland {
	fn init_xdg_surface(
		self: &mut Wayland,
		qh : &QueueHandle<Wayland>,
	) -> () {
		let wm_base : &XdgWmBase =
			self.wm_base.as_ref().expect("Should have XdgWmBase");
		let base_surface : &WlSurface = self.base_surface.as_ref().unwrap();

		let xdg_surface : XdgSurface =
			wm_base.get_xdg_surface(base_surface, qh, ());
		let toplevel : XdgToplevel = xdg_surface.get_toplevel(qh, ());
		toplevel.set_title("MVYVGRS".into());

		base_surface.commit();

		self.xdg_surface = Some((xdg_surface, toplevel));
	}
}

impl WindowSystem for Wayland {
	fn init(
		screen_size : (u32, u32),
		window_name : &str,
	) -> Result<Wayland, String> {
		// Establishing Wayland connection,
		// getting display and registering
		// globals
		let conn : Connection = Connection::connect_to_env().map_err(
			|e : wayland_client::ConnectError| -> String { e.to_string() },
		)?;

		let display : WlDisplay = conn.display();

		let mut event_queue : EventQueue<Wayland> = conn.new_event_queue();

		let qh : QueueHandle<Wayland> = event_queue.handle();

		let _registry : WlRegistry = display.get_registry(&qh, ());

		let mut wayland_state : Wayland = Wayland {
			connection : conn,
			event_queue : Arc::new(event_queue),
			base_surface : None,
			buffer : None,
			wm_base : None,
			xdg_surface : None,
			configured : false,
		};

		Arc::<EventQueue<Wayland>>::get_mut(&mut wayland_state.event_queue)
			.expect("No one else should be using the event queue atm")
			.blocking_dispatch(&mut wayland_state)
			.map_err(|e : wayland_client::DispatchError| -> String {
				e.to_string()
			})?;

		Ok(wayland_state)
	}

	fn handle_events(
		self: &mut Wayland,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> () {
		Arc::<EventQueue<Wayland>>::get_mut(&mut self.event_queue)
			.expect("No one else should be using the event queue atm")
			.blocking_dispatch(self)
			.expect("WAYLAND EVENT PROCCESSING SHOULD BE INFAILABLE!");
	}

	fn draw_frame(
		self: &mut Wayland,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> () {
	}

	fn get_win_w(self: &Wayland) -> u32 { 100 }

	fn get_win_h(self: &Wayland) -> u32 { 100 }
}
