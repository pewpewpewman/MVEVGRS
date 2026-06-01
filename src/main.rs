mod renderer;
mod pixel;
mod mesh;
mod window_render_target;

use std::time::{Duration, Instant};

use glam::{Mat4, Vec3, Vec4};

use crate::mesh::{BasicCE, BasicP, BasicTE, BasicV, Mesh};
use crate::renderer::{Renderer, RendererSettings};
use crate::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let start_time : Instant = Instant::now();

	let mut frame_start_time : Instant = Instant::now();

	let mut last_frame_duration : Duration = Duration::from_secs(0);

	let mut renderer : Renderer<BasicV, BasicTE, BasicP, BasicCE> = Renderer::new(
		RendererSettings::default(),
		//vec![Mesh::<BasicV, BasicTE, BasicP, BasicCE>::default()],
		vec![Mesh::<BasicV, BasicTE, BasicP, BasicCE>::unit_cube()],
		Some(Box::new(
			move |r : &mut Renderer<BasicV, BasicTE, BasicP, BasicCE>| -> () {
				last_frame_duration = Instant::now().duration_since(frame_start_time);
				frame_start_time = Instant::now();
				let t : f32 = Instant::now().duration_since(start_time).as_secs_f32();

				let pos : Vec3 = Vec3::new(0.0, 0.0, 3.0);
				let t_x : f32 = t; //0.0;
				let t_y : f32 = t; //std::f32::consts::PI;
				let t_z : f32 = t; //0.0;
				let scale : f32 = 2.0 * f32::sin(t * 0.5);

				r.meshes[0].model_mat = Mat4::from_translation(pos)
					* Mat4::from_rotation_x(t_x)
					* Mat4::from_rotation_y(t_y)
					* Mat4::from_rotation_z(t_z)
					* Mat4::from_scale(Vec3::splat(scale));
			},
		)),
	);

	WindowRenderTarget::<BasicV, BasicTE, BasicP, BasicCE>::new(&mut renderer)
		.expect("bruhhh");

	Ok(())
}
