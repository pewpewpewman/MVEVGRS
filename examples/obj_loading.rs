use std::ops::{Add, Mul};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4};
use image::{DynamicImage, ImageReader, Pixel, Rgba};
use mvevgrs::mesh::obj_file::{load_obj_file, ObjV};
use mvevgrs::mesh::{Mesh, Tri};
use mvevgrs::renderer::user_stage::{ColorContext, VertContext, VertTransOut};
use mvevgrs::renderer::Renderer;
use mvevgrs::window_render_target::WindowRenderTarget;

fn main() -> Result<(), String> {
	//TODO: Look into derive ops crate
	#[derive(Clone, Copy, Debug)]
	struct BasicP {
		uv : Vec2,
	}

	impl Mul<f32> for BasicP {
		type Output = BasicP;

		fn mul(
			self: BasicP,
			rhs : f32,
		) -> Self::Output {
			BasicP {
				uv : self.uv * rhs,
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
				uv : self.uv + rhs.uv,
			}
		}
	}

	#[derive(Clone, Debug)]
	struct BasicUE {
		p_mat : Mat4,
		cm_mat : Mat4,
		texture : DynamicImage,
	}

	let start_time : Instant = Instant::now();
	let mut frame_start_time : Instant = Instant::now();
	let mut last_frame_duration : Duration = Duration::from_secs(0);

	WindowRenderTarget::<ObjV, BasicP, BasicUE>::new(&mut Renderer::<
		ObjV,
		BasicP,
		BasicUE,
	>::new(
		640,
		480,
		//Vert transformer
		|vd : &ObjV, ue : &BasicUE, _vc : &VertContext| -> VertTransOut<BasicP> {
			VertTransOut {
				pos : ue.p_mat * ue.cm_mat * vd.pos,
				colorer_data : BasicP {
					uv : vd.uv.xy(),
				},
			}
		},
		//Pixel colorer
		|cd : &BasicP, ue : &BasicUE, _cc : &ColorContext| -> Vec4 {
			if let Some(p) =
				image::imageops::sample_nearest(&ue.texture, cd.uv.x, 1.0 - cd.uv.y)
			{
				let rgb : &[u8] = p.channels();
				let col : Vec3 = Vec3::new(
					rgb[2] as f32 / u8::MAX as f32,
					rgb[1] as f32 / u8::MAX as f32,
					rgb[0] as f32 / u8::MAX as f32,
				);

				Vec4::from((col, 1.0))
			} else {
				Vec4::new(1.0, 0.5, 0.3, 1.0)
			}
		},
		BasicUE {
			p_mat : Mat4::default(),
			cm_mat : Mat4::default(),
			texture : image::open("models/spot_texture.png").unwrap(),
		},
		vec![load_obj_file("models/spot.obj")?],
		Some(Box::new(
			move |r : &mut Renderer<ObjV, BasicP, BasicUE>| -> () {
				last_frame_duration = Instant::now().duration_since(frame_start_time);
				frame_start_time = Instant::now();
				let t : f32 = Instant::now().duration_since(start_time).as_secs_f32();

				let pos : Vec3 = Vec3::new(0.0, 0.0, 3.0);
				let t_x : f32 = 0.0;
				let t_y : f32 = t;
				let t_z : f32 = 0.0;
				let scale : f32 = 5.0;

				r.meshes[0].model_mat = Mat4::from_translation(pos)
					* Mat4::from_rotation_x(t_x)
					* Mat4::from_rotation_y(t_y)
					* Mat4::from_rotation_z(t_z)
					* Mat4::from_scale(Vec3::splat(scale));

				r.user_func_env.p_mat = r.camera.proj_mat;
				r.user_func_env.cm_mat = r.camera.camera_mat * r.meshes[0].model_mat;
			},
		)),
	))?;

	Ok(())
}
