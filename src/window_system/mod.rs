pub mod x11;
pub mod wayland;

use crate::pixel::Pixel;

pub trait WindowSystem: Sized {
	fn init(
		screen_size : (u32, u32),
		window_name : &str,
	) -> Result<Self, String>;

	fn handle_events(
		self: &mut Self,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> ();

	fn draw_frame(
		self: &mut Self,
		rend_framebuffer : &mut Vec<Pixel>,
	) -> ();

	fn get_win_w(self: &Self) -> u32;
	fn get_win_h(self: &Self) -> u32;
}
