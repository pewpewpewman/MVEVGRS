mod renderer;
mod pixel;
mod triangle;
mod window_render_target;

use std::time::Instant;

use glam::Vec3;

use crate::renderer::{Renderer, RendererSettings};
use crate::triangle::Triangle;
use crate::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let mut t : f32 = 0_f32;

	let start_time : Instant = Instant::now();

	let mut renderer : Renderer = Renderer::new(
		RendererSettings::default(),
		vec![Triangle::new(
			Vec3::new(0.5, 0.5, 1.0),
			Vec3::new(0.5, -0.5, 1.0),
			Vec3::new(-0.5, -0.5, 1.0),
		)],
		Some(Box::new(move |r : &mut Renderer| -> () {
			r.tris[0].points[1].x = 2_f32 * f32::cos(t);

			t = Instant::now().duration_since(start_time).as_secs_f32();
		})),
	);

	WindowRenderTarget::new(&mut renderer).expect("bruhhh");

	Ok(())
}
