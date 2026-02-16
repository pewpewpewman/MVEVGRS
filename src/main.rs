mod renderer;

mod pixel;

mod triangle;

use crate::renderer::Renderer;
use crate::triangle::{Point, Triangle};

fn main() -> Result<(), ()> {
	Renderer::run(
		|r : &mut Renderer| -> () {
			let mut tri : Triangle = Triangle::new(
				Point::new(-1_f32, 0_f32, 0_f32),
				Point::new(0_f32, 0_f32, 0_f32),
				Point::new(1_f32, 0_f32, 0_f32),
			);

			r.tris.push(tri);
		},
		|r : &mut Renderer| -> () {
			let t : f32 = 5_f32;

			let tri : &mut Triangle = &mut r.tris[0];

			tri.points[0].y = -t.cos();
			tri.points[2].y = t.sin();
		},
	)
	.expect("ok guys");

	Ok(())
}
