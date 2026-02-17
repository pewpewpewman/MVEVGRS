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
			let tri : Triangle = Triangle::new(
				Point::new(-1_f32, 0_f32, 0_f32),
				Point::new(0_f32, 0_f32, 0_f32),
				Point::new(1_f32, 0_f32, 0_f32),
			);

			r.tris.push(tri);
		},
		Some(Box::new(move |r : &mut Renderer| -> () {
			let tri : &mut Triangle = &mut r.tris[0];

			tri.points[0].y = -t.cos();
			tri.points[2].y = t.sin();

			t = Instant::now().duration_since(start_time).as_secs_f32();
		})),
	)
	.expect("ok guys");

	Ok(())
}
