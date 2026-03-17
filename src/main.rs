mod renderer;
mod pixel;
mod triangle;
mod window_render_target;

use std::time::Instant;

use crate::renderer::{Renderer, RendererSettings};
use crate::triangle::Triangle;
use crate::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let mut t : f32 = 0_f32;

	let start_time : Instant = Instant::now();

	let mut renderer : Renderer = Renderer::new(
		RendererSettings::default(),
		vec![Triangle::default(); 2],
		Some(Box::new(move |r : &mut Renderer| -> () {
			let tri : &mut Triangle = &mut r.tris[0];

			let val : f32 = t * 0.5_f32;

			tri.points[0].x = val.cos();
			tri.points[0].y = -val.cos();

			tri.points[1].x = -val.sin();
			tri.points[1].y = val.cos();

			tri.points[2].x = val.cos();
			tri.points[2].y = val.sin();

			//Second tri
			let tri : &mut Triangle = &mut r.tris[1];

			let val : f32 = t * 0.35_f32;

			tri.points[0].x = val.cos();
			tri.points[0].y = -val.cos();

			tri.points[1].x = -val.sin();
			tri.points[1].y = val.cos();

			tri.points[2].x = val.cos();
			tri.points[2].y = val.sin();

			t = Instant::now().duration_since(start_time).as_secs_f32();
		})),
	);

	WindowRenderTarget::new(&mut renderer).expect("bruhhh");

	Ok(())
}
