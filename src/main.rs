mod renderer;

mod pixel;

mod triangle;

mod window_system;

use crate::renderer::Renderer;
use crate::triangle::{Point, Triangle};
use crate::window_system::x11::X11;

fn main() -> Result<(), ()> {
	let mut renderer : Renderer<X11> = Renderer::new().unwrap();

	let mut tri : Triangle = Triangle::default();

	let _p : Point = Point::default();

	tri.points[2].y = -0.1_f32;

	renderer.tris.push(tri);

	let mut t : f32 = 0_f32;

	loop {
		let start : std::time::Instant = std::time::Instant::now();

		renderer.step_frame();

		renderer.tris[0].points[1].x = t.sin();

		t += std::time::Instant::now()
			.duration_since(start)
			.as_secs_f32();
	}

	Ok(())
}
