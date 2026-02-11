mod renderer;

mod pixel;

mod triangle;

mod window_system;

use crate::renderer::Renderer;
use crate::triangle::{Point, Triangle};
use crate::window_system::wayland::Wayland;
use crate::window_system::x11::X11;

fn main() -> Result<(), ()> {
	let mut renderer : Renderer<Wayland> = Renderer::new().expect("ok guys");

	let tri : Triangle = Triangle::default();

	let mut tri : Triangle = Triangle::new(
		Point::new(-1_f32, 0_f32, 0_f32),
		Point::new(0_f32, 0_f32, 0_f32),
		Point::new(1_f32, 0_f32, 0_f32),
	);

	renderer.tris.push(tri);

	let mut t : f32 = 0_f32;

	loop {
		let start : std::time::Instant = std::time::Instant::now();

		let tri : &mut Triangle = &mut renderer.tris[0];

		tri.points[0].y = -t.cos();
		tri.points[2].y = t.sin();

		renderer.step_frame();

		t += std::time::Instant::now()
			.duration_since(start)
			.as_secs_f32();
	}

	Ok(())
}
