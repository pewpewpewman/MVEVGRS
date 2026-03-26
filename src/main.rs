mod renderer;
mod pixel;
mod mesh;
mod window_render_target;

use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};

use crate::mesh::Mesh;
use crate::renderer::{Renderer, RendererSettings};
use crate::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let start_time : Instant = Instant::now();

	let mut frame_start_time : Instant = Instant::now();

	let mut last_frame_duration : Duration = Duration::from_secs(0);

	let fps_debug : bool = false;

	let mut renderer : Renderer = Renderer::new(
		RendererSettings::default(),
		vec![Mesh::unit_cube()],
		Some(Box::new(move |r : &mut Renderer| -> () {
			last_frame_duration = Instant::now().duration_since(frame_start_time);
			frame_start_time = Instant::now();

			if fps_debug {
				dbg!(last_frame_duration);

				println!("frame rate: {}", 1_f32 / last_frame_duration.as_secs_f32());
			}

			let t : f32 = Instant::now().duration_since(start_time).as_secs_f32();
			r.meshes[0].model_mat = Mat4::from_translation(Vec3::new(0.0, 0.0, 2.0))
				//* Mat4::from_rotation_x(t)
				//* Mat4::from_rotation_y(t)
				//* Mat4::from_rotation_z(t);
				;
		})),
	);

	WindowRenderTarget::new(&mut renderer).expect("bruhhh");

	Ok(())
}
