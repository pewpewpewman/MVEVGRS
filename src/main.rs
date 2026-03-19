mod renderer;
mod pixel;
mod mesh;
mod window_render_target;

use std::time::Instant;

use glam::{Mat4, Vec3};

use crate::mesh::Mesh;
use crate::renderer::{Renderer, RendererSettings};
use crate::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let start_time : Instant = Instant::now();

	let mut renderer : Renderer = Renderer::new(
		RendererSettings::default(),
		vec![Mesh::unit_cube()],
		Some(Box::new(move |r : &mut Renderer| -> () {
			r.meshes[0].model_mat = Mat4::from_translation(Vec3::new(0.0, 0.0, 5.0))
				* Mat4::from_rotation_y(
					Instant::now().duration_since(start_time).as_secs_f32(),
				);
		})),
	);

	WindowRenderTarget::new(&mut renderer).expect("bruhhh");

	Ok(())
}
