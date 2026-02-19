mod renderer;

mod pixel;

mod triangle;

use std::time::{Duration, Instant};

use crate::renderer::Renderer;
use crate::triangle::{Point, Triangle};

fn main() -> Result<(), ()> {
	let mut t : f32 = 0_f32;

	let mut start_time : Instant = Instant::now();

	Renderer::run(
		|r : &mut Renderer| -> () {
			let tri : Triangle = Triangle::default();

			r.tris.push(tri);
			r.tris.push(tri);
		},
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
	)
	.expect("ok guys");

	Ok(())
}
