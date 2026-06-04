use std::ops::{Add, Mul};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec3, Vec4};
use mvevgrs::mesh::{Mesh, Tri};
use mvevgrs::renderer::user_stage::{ColorContext, VertContext, VertTransOut};
use mvevgrs::renderer::Renderer;
use mvevgrs::window_render_target::WindowRenderTarget;

fn main() -> Result<(), ()> {
	let start_time : Instant = Instant::now();

	let mut frame_start_time : Instant = Instant::now();

	let mut last_frame_duration : Duration = Duration::from_secs(0);

	let mut renderer : Renderer<BasicV, BasicP, BasicUE> = Renderer::new(
		640,
		480,
		basic_vertex_transformer,
		basic_pixel_colorer,
		BasicUE::default(),
		vec![Mesh::<BasicV>::new(
			Mesh::<Vec3>::unit_cube()
				.tris
				.into_iter()
				.map(|v : Tri<Vec3>| -> [Vec3; 3] { v.0 })
				.flatten()
				.enumerate()
				.map(|(i, v) : (usize, Vec3)| -> BasicV {
					BasicV {
						position : v,
						color : {
							let mut res : Vec3 = Vec3::ZERO;
							if i & 1 != 0 {
								res += Vec3::new(0.0, 0.0, 1.0);
							}
							if i & 2 != 0 {
								res += Vec3::new(0.0, 1.0, 0.0);
							}
							if i & 4 != 0 {
								res += Vec3::new(1.0, 0.0, 0.0);
							}
							res
						},
					}
				})
				.collect::<Vec<BasicV>>()
				.chunks(3)
				.map(|vs : &[BasicV]| -> Tri<BasicV> { Tri::new(vs[0], vs[1], vs[2]) })
				.collect::<Vec<Tri<BasicV>>>(),
			Mat4::IDENTITY,
		)],
		Some(Box::new(
			move |r : &mut Renderer<BasicV, BasicP, BasicUE>| -> () {
				last_frame_duration = Instant::now().duration_since(frame_start_time);
				frame_start_time = Instant::now();
				let t : f32 = Instant::now().duration_since(start_time).as_secs_f32();
				dbg!(1.0 / last_frame_duration.as_secs_f32());

				let pos : Vec3 = Vec3::new(0.0, 0.0, 3.0);
				let t_x : f32 = t;
				let t_y : f32 = t;
				let t_z : f32 = t;
				let scale : f32 = 1.0;

				r.meshes[0].model_mat = Mat4::from_translation(pos)
					* Mat4::from_rotation_x(t_x)
					* Mat4::from_rotation_y(t_y)
					* Mat4::from_rotation_z(t_z)
					* Mat4::from_scale(Vec3::splat(scale));

				r.user_func_env = BasicUE {
					p_mat : r.camera.proj_mat,
					cm_mat : r.camera.camera_mat * r.meshes[0].model_mat,
					brightness : 0.5,
				};
			},
		)),
	);

	WindowRenderTarget::<BasicV, BasicP, BasicUE>::new(&mut renderer)
		.expect("bruhhh");

	Ok(())
}

//A basic examples of the types put into Renderer's generics
#[derive(Clone, Copy, Debug)]
struct BasicV {
	position : Vec3,
	color : Vec3,
}

//TODO: Look into derive ops crate
#[derive(Clone, Copy, Debug)]
struct BasicP {
	color : Vec3,
}

impl Mul<f32> for BasicP {
	type Output = BasicP;

	fn mul(
		self: BasicP,
		rhs : f32,
	) -> Self::Output {
		BasicP {
			color : self.color * rhs,
		}
	}
}

impl Add for BasicP {
	type Output = BasicP;

	fn add(
		self: BasicP,
		rhs : BasicP,
	) -> Self::Output {
		BasicP {
			color : self.color + rhs.color,
		}
	}
}

#[derive(Clone, Copy, Debug, Default)]
struct BasicUE {
	p_mat : Mat4,
	cm_mat : Mat4,
	brightness : f32,
}

fn basic_vertex_transformer(
	vert_data : &BasicV,
	user_env : &BasicUE,
	_vert_cont : &VertContext,
) -> VertTransOut<BasicP> {
	VertTransOut {
		pos : user_env.p_mat
			* user_env.cm_mat
			* Vec4::from((vert_data.position, 1_f32)),
		colorer_data : BasicP {
			color : vert_data.color,
		},
	}
}

fn basic_pixel_colorer(
	color_data : &BasicP,
	user_env : &BasicUE,
	_col_context : &ColorContext,
) -> Vec4 {
	user_env.brightness * Vec4::from((color_data.color, 1.0))
}
