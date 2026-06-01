// Module that handles the main loop of
// drawing and rendering.

mod camera;

use std::cmp::Ordering;
use std::ops::{Add, BitAnd, BitOr, Div, Mul};

use camera::Camera;
use glam::{IVec2, Mat3, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

use crate::mesh::{
	Mesh,
	PixelColorer,
	Triangle,
	VertTransOut,
	VertexTransformer,
};
use crate::pixel::Pixel;

type UpdateFunc<V, TE, P, CE> =
	Box<dyn FnMut(&mut Renderer<V, TE, P, CE>) -> ()>;

//The main renderer. For information on what these type generics do, please refer to
//./src/mesh/mod.rs
pub struct Renderer<V, TE, P, CE> {
	// Main frame buffer that is written to
	pub frame_buffer : Vec<Pixel>,
	// Depth buffer that is used for knowing what tris are visible
	pub depth_buffer : Vec<f32>,
	// Settings for how to draw things
	pub renderer_settings : RendererSettings,
	// Camera that holds the camera and projection matrix
	pub camera : Camera,
	// Triangles to be rastered
	pub meshes : Vec<Mesh<V, TE, P, CE>>,
	// Update function to run before drawing each frame
	update_fn : Option<UpdateFunc<V, TE, P, CE>>,
}

impl<V, TE, P, CE> Renderer<V, TE, P, CE>
where
	V : Clone + Copy,
	TE : Clone,
	P : Clone + Copy + Mul<f32, Output = P> + Add<Output = P>,
	CE : Clone,
{
	pub fn new(
		renderer_settings : RendererSettings,
		meshes : Vec<Mesh<V, TE, P, CE>>,
		update_fn : Option<Box<dyn FnMut(&mut Renderer<V, TE, P, CE>) -> ()>>,
	) -> Renderer<V, TE, P, CE> {
		let pix_area : usize =
			(renderer_settings.width * renderer_settings.height) as usize;

		Renderer {
			frame_buffer : vec![renderer_settings.background_col; pix_area],
			depth_buffer : vec![f32::MAX; pix_area],
			renderer_settings,
			camera : Camera::default(),
			meshes,
			update_fn,
		}
	}

	pub fn width(self: &Renderer<V, TE, P, CE>) -> u32 {
		self.renderer_settings.width
	}

	pub fn height(self: &Renderer<V, TE, P, CE>) -> u32 {
		self.renderer_settings.height
	}

	// Helpful conversion functions between
	// NDC and pixel coordinates and vice
	// versa
	pub fn screen_x_to_ndx(
		self: &Renderer<V, TE, P, CE>,
		x : i32,
	) -> f32 {
		x as f32 / self.width() as f32 * 2_f32 - 1_f32
	}

	pub fn screen_y_to_ndy(
		self: &Renderer<V, TE, P, CE>,
		y : i32,
	) -> f32 {
		(1_f32 - (y as f32 / self.height() as f32)) * 2_f32 - 1_f32
	}

	pub fn screen_coords_to_ndc(
		self: &Renderer<V, TE, P, CE>,
		c : IVec2,
	) -> Vec3 {
		Vec3::new(self.screen_x_to_ndx(c.x), self.screen_y_to_ndy(c.y), 0_f32)
	}

	pub fn ndx_to_screen_x(
		self: &Renderer<V, TE, P, CE>,
		x : f32,
	) -> i32 {
		f32::round((self.width() - 1) as f32 * ((1_f32 + x) / 2_f32)) as i32
	}

	pub fn ndy_to_screen_y(
		self: &Renderer<V, TE, P, CE>,
		y : f32,
	) -> i32 {
		f32::round((self.height() - 1) as f32 * (1_f32 - ((1_f32 + y) / 2_f32)))
			as i32
	}

	pub fn ndc_to_screen_c(
		self: &Renderer<V, TE, P, CE>,
		p : Vec2,
	) -> IVec2 {
		IVec2::new(self.ndx_to_screen_x(p.x), self.ndy_to_screen_y(p.y))
	}

	//Calculates the Cohen-Sutherland reigon code of p
	fn reigon_code(p : &Vec4) -> u8 {
		(((p.x > p.w) as u8) << 0)
			.bitor(((p.x < -p.w) as u8) << 1)
			.bitor(((p.y > p.w) as u8) << 2)
			.bitor(((p.y < -p.w) as u8) << 3)
			.bitor(((p.z > p.w) as u8) << 4)
			.bitor(((p.z < -p.w) as u8) << 5)
	}

	//Clips point_1 and point_2 along edge described by egde. Edge is a bit mask
	//that describes one of the clip space planes and most be a result of reigon_code.
	//The points are clipped along the least significant 1 in that bit mask.
	fn clip_point(
		point_1 : &Vec4,
		point_2 : &Vec4,
		edge : u8,
	) -> Vec4 {
		//Bit index gives us information about what to clip
		//and how. bit_idx / 2 is which axis a point is out of
		//bounds on, x, y or z. bit_idx % 2 tells us if value
		//along the axis is too much or too little
		let bit_idx : u8 = u8::trailing_zeros(edge) as u8;
		let axis : usize = (bit_idx / 2) as usize;
		let sign : f32 = if bit_idx % 2 == 0 {
			1.0
		} else {
			-1.0
		};

		//Clipping formula and approach from
		//https://www.cs.ucr.edu/~shinar/courses/cs130-winter-2021/content/clipping.pdf
		//
		//The bastards at UCR denied me in 2024 - it pains me so
		//deeply to go to them in my hour of need
		//
		//The near  plane poses a problem as z_clip is set to
		//be [0, 1] because of the perspective matrix we chose
		//to use so that case uses a modified formula for alpha
		let alpha : f32 = if bit_idx != 4 {
			(sign * point_2.w - point_2[axis])
				/ (point_1[axis] - sign * point_1.w + sign * point_2.w - point_2[axis])
		} else {
			-point_2.z / (point_1.z - point_2.z)
		};

		let mut new_point : Vec4 = alpha * point_1 + (1.0 - alpha) * point_2;

		//This sucks but without this we get rounding errors that
		//cause an infinite loop where alpha = 1 so no real progress
		//is made
		if bit_idx != 4 {
			new_point[axis] =
				new_point[axis].clamp(-new_point.w.abs(), new_point.w.abs());
		} else {
			new_point[axis] = new_point[axis].clamp(0.0, new_point.w);
		}

		new_point
	}

	// Draw a single triangle to the
	// frame_buffer
	fn raster_tri(
		self: &mut Renderer<V, TE, P, CE>,
		tri : &Triangle<V>,
		vertex_transformer : VertexTransformer<V, TE, P, CE>,
		transformer_env : &TE,
		pixel_colorer : PixelColorer<V, TE, P, CE>,
		colorer_env : &CE,
	) -> () {
		//Vertex information from the vertex transformer
		let trans_out : [VertTransOut<P>; 3] =
			tri.0.map(|v : V| -> VertTransOut<P> {
				vertex_transformer(&v, transformer_env, self)
			});

		let mut poly_points : Vec<Vec4> = trans_out.iter().fold(
			Vec::with_capacity(6),
			|mut pp : Vec<Vec4>, vto : &VertTransOut<P>| -> Vec<Vec4> {
				pp.push(vto.pos);
				pp
			},
		);

		//The matrix that converts a point in projected space to a vector of the world space
		//barycentric coords. The convention we will use is poly_points[0] as "a", poly_points[1]
		//as "b" and poly_points[2] as "c". Formula is from
		//https://andrewkchan.dev/posts/perspective-interpolation.html
		let bary_mat : Mat3 = Mat3::mul_mat3(
			&Mat3::from_diagonal(Vec3::new(
				1_f32 / poly_points[0].z,
				1_f32 / poly_points[1].z,
				1_f32 / poly_points[2].z,
			)),
			&Mat3::inverse(&Mat3::from_cols(
				poly_points[0].xyz() / poly_points[0].w,
				poly_points[1].xyz() / poly_points[1].w,
				poly_points[2].xyz() / poly_points[2].w,
			)),
		);

		//Cohen-Sutherland reigon code approach from
		//https://www.slideshare.net/slideshow/clipping-presentation/878649
		//That website is so ass bro wth
		let mut reigon_codes : Vec<u8> = poly_points
			.iter()
			.map(Self::reigon_code)
			.collect::<Vec<u8>>();

		//Sutherland–Hodgman clipping algorithm and implementation is from
		//https://en.wikipedia.org/wiki/Sutherland%E2%80%93Hodgman_algorithm#Pseudocode
		(0..=5).into_iter().for_each(|i : u8| -> () {
			let edge_mask : u8 = 1 << i;

			let mut input_points : Vec<Vec4> = poly_points.clone();
			poly_points.clear();
			let mut input_codes : Vec<u8> = reigon_codes.clone();
			reigon_codes.clear();

			(0..input_points.len())
				.into_iter()
				.for_each(|j : usize| -> () {
					let prev_idx : usize =
						((j as isize - 1).rem_euclid(input_points.len() as isize)) as usize;
					let curr_point : Vec4 = input_points[j];
					let curr_code : u8 = input_codes[j];
					let prev_point : Vec4 = input_points[prev_idx];
					let prev_code : u8 = input_codes[prev_idx];

					if curr_code & edge_mask == 0 {
						if prev_code & edge_mask != 0 {
							let new_point : Vec4 =
								Self::clip_point(&curr_point, &prev_point, edge_mask);
							let new_code : u8 = Self::reigon_code(&new_point);
							poly_points.push(new_point);
							reigon_codes.push(new_code);
						}
						poly_points.push(curr_point);
						reigon_codes.push(curr_code);
					} else if prev_code & edge_mask == 0 {
						let new_point : Vec4 =
							Self::clip_point(&curr_point, &prev_point, edge_mask);
						let new_code : u8 = Self::reigon_code(&new_point);
						poly_points.push(new_point);
						reigon_codes.push(new_code);
					}
				});
		});

		if poly_points.len() < 3 {
			return;
		}

		//Perspective divide
		poly_points.iter_mut().for_each(|v : &mut Vec4| -> () {
			*v = Vec4::from((v.xyz() / v.w, v.w))
		});

		//Put points in clockwise order - compare formula
		//from https://stackoverflow.com/a/6989416
		let poly_center : Vec2 = poly_points
			.iter()
			.map(|v : &Vec4| -> Vec2 { v.xy() })
			.sum::<Vec2>()
			.div(poly_points.len() as f32);

		poly_points.sort_by(|a : &Vec4, b : &Vec4| -> Ordering {
			let a_center : Vec2 = a.xy() - poly_center;
			let b_center : Vec2 = b.xy() - poly_center;

			//TODO: With a bit if headscratching, this can
			//probably be written more rustaciously using
			//things like Ordering::then()
			if a_center.x >= 0.0 && b_center.x < 0.0 {
				Ordering::Less
			} else if a_center.x < 0.0 && b_center.x >= 0.0 {
				Ordering::Greater
			} else if a_center.x == 0.0 && b_center.x == 0.0 {
				b.y.total_cmp(&a.y)
			} else {
				let det : f32 = a_center.x * b_center.y - b_center.x * a_center.y;

				if det < 0.0 {
					Ordering::Less
				} else if det > 0.0 {
					Ordering::Greater
				} else {
					let d1 : f32 = a_center.x * a_center.x + a_center.y * a_center.y;
					let d2 : f32 = b_center.x * b_center.x + b_center.y * b_center.y;

					d1.total_cmp(&d2)
				}
			}
		});

		//Traiangulation algorithm I spent a good few days of my summer vacation
		//trying to figure out on my own that's so obvious it's in the first few
		//sentances and given little fanfare is from
		//https://swaminathanj.github.io/cg/PolygonTriangulation.html
		//TODO: See if there's a way to make this cache a little nicer
		(1..=poly_points.len() - 2)
			.into_iter()
			.for_each(|i : usize| -> () {
				let mut y_sorted : [Vec2; 3] = [
					poly_points[0].xy(),
					poly_points[i].xy(),
					poly_points[i + 1].xy(),
				];
				y_sorted
					.sort_by(|a : &Vec2, b : &Vec2| -> Ordering { b.y.total_cmp(&a.y) });

				let y_sorted_screen : [i32; 3] =
					y_sorted.map(|v : Vec2| -> i32 { self.ndy_to_screen_y(v.y) });

				//Iterate over the triangle in two segments
				(0..=1_usize).for_each(|j : usize| -> () {
					//Iterate the top to the mid point in the first iteration
					//and then from the mid point to the bottom in the second
					let init_y : i32 = y_sorted_screen[j];
					let fina_y : i32 = y_sorted_screen[j + 1];

					(init_y..=fina_y).for_each(|y_screen : i32| -> () {
						let y_ndc : f32 = self.screen_y_to_ndy(y_screen);

						//t used to lerp along the unbroken edge of the triangle
						let full_t : f32 =
							(y_ndc - y_sorted[0].y) / (y_sorted[2].y - y_sorted[0].y);

						//Prevent sliver triangles that are draw too far
						//along the width of the screen - TODO: make this
						//stop the iteration of y values entirely with try_for_each()
						//TODO: Be less shit
						if full_t < 0.0 || full_t > 1.0 {
							return;
						}

						//t used to lerp along the 2 broken edges of the triangle
						let part_t : f32 =
							(y_ndc - y_sorted[j].y) / (y_sorted[j + 1].y - y_sorted[j].y);

						//Ditto
						if part_t < 0.0 || part_t > 1.0 {
							return;
						}

						let mut full_x : f32 =
							y_sorted[0].x + (y_sorted[2].x - y_sorted[0].x) * full_t;

						let mut part_x : f32 =
							y_sorted[j].x + (y_sorted[j + 1].x - y_sorted[j].x) * part_t;

						let (init_x, fina_x) : (f32, f32) = if full_x > part_x {
							(part_x, full_x)
						} else {
							(full_x, part_x)
						};

						let init_x : i32 = self.ndx_to_screen_x(init_x);
						let fina_x : i32 = self.ndx_to_screen_x(fina_x);

						(init_x..=fina_x).for_each(|x_screen : i32| -> () {
							let fb_idx : usize =
								(y_screen * (self.width() as i32) + x_screen) as usize;

							let fill_col : Vec4 = Vec4::new(0.85, 0.5, 0.7, 1.0);

							let fill_col : Vec4 =
								if self.renderer_settings.show_tri_div && j == 0 {
									fill_col
								} else {
									Vec4::ONE - fill_col
								};

							//Array access of doom
							self.frame_buffer[fb_idx] = fill_col;

							//if x_screen == init_x {
							//	self.frame_buffer[fb_idx] = Vec4::ONE;
							//}
							//if x_screen == fina_x {
							//	self.frame_buffer[fb_idx] = Vec4::ZERO;
							//}
						});
					});
				});

				//MARK VERTICES ON TRIANGLE IN COLORS
				y_sorted.into_iter().enumerate().for_each(
					|(i, v) : (usize, Vec2)| -> () {
						let y : i32 = self.ndy_to_screen_y(v.y);
						let x : i32 = self.ndx_to_screen_x(v.x);
						(-2..=2).for_each(|x_offset : i32| -> () {
							(-2..=2).for_each(|y_offset : i32| -> () {
								let y : i32 =
									i32::clamp(y + y_offset, 0, self.height() as i32 - 1);
								let x : i32 =
									i32::clamp(x + x_offset, 0, self.width() as i32 - 1);

								let fb_idx : usize = ((y * self.width() as i32) + x) as usize;

								self.frame_buffer[fb_idx] = [
									Vec4::new(1.0, 0.0, 0.0, 1.0),
									Vec4::new(0.0, 1.0, 0.0, 1.0),
									Vec4::new(0.0, 0.0, 1.0, 1.0),
								][i % 3];
							});
						});
					},
				);
				//END OF COLORED  DEBUG VERTS
			});
	}

	pub fn draw(self: &mut Renderer<V, TE, P, CE>) -> () {
		// Raster all triangles
		self
			.frame_buffer
			.fill(self.renderer_settings.background_col);

		self.depth_buffer.fill(f32::MAX);

		let _proj_cam_mat : Mat4 = self.camera.proj_mat * self.camera.camera_mat;

		self
			.meshes
			.clone()
			.into_iter()
			.for_each(|m : Mesh<V, TE, P, CE>| -> () {
				let trans_env : TE = (m.trans_env_updater)(&m, self);

				let color_env : CE = (m.color_env_updater)(&m, self);

				m.tris.iter().for_each(|t : &Triangle<V>| -> () {
					self.raster_tri(
						&t,
						m.vertex_transformer,
						&trans_env,
						m.pixel_colorer,
						&color_env,
					);
				});
			});
	}

	pub fn frame_step(self: &mut Renderer<V, TE, P, CE>) -> () {
		//Calling a function that acts on its own struct causes
		//some borrow checker problems, let's do some shenanigans
		//to please it.
		let mut temp : Option<UpdateFunc<V, TE, P, CE>> = self.update_fn.take();
		if let Some(f) = &mut temp {
			let f : &mut UpdateFunc<V, TE, P, CE> = f;
			(f)(self);
		}
		self.update_fn = temp;

		self.draw();
	}
}

pub struct RendererSettings {
	// INTERNAL render width and height - may or may not match up with what the target for
	// rendering is
	pub width : u32,
	pub height : u32,
	// Go to value for filling the frame buffer
	pub background_col : Pixel,
	// Triangles are drawn in 2 phases, set this to true if you want the second phase to have
	// inverted colors
	pub show_tri_div : bool,
}

impl Default for RendererSettings {
	fn default() -> RendererSettings {
		RendererSettings {
			width : 320 * 2,
			height : 240 * 2,
			background_col : Pixel::new(0.7, 0.6, 0.9, 0.5),
			show_tri_div : false,
		}
	}
}
