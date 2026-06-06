use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::mesh::{Mesh, Tri};

#[derive(Debug, Default, Clone, Copy)]
pub struct ObjV {
	pub pos : Vec4,
	pub uv : Vec3,
	pub norm : Vec3,
}

pub fn load_obj_file(file_name : &str) -> Result<Mesh<ObjV>, String> {
	let obj_file : BufReader<File> = BufReader::<File>::new(
		File::open(file_name)
			.map_err(|e : io::Error| -> String { e.to_string() })?,
	);

	let mut pos : Vec<Vec4> = Vec::new();
	let mut uv : Vec<Vec3> = Vec::new();
	let mut norm : Vec<Vec3> = Vec::new();
	let mut tri : Vec<Tri<ObjV>> = Vec::new();

	obj_file
		.lines()
		.enumerate()
		.map(
			|(n, r) : (usize, Result<String, io::Error>)| -> (usize, String) {
				(n + 1, r.unwrap())
			},
		)
		.try_for_each(|(n, l) : (usize, String)| -> Result<(), String> {
			if !l.is_ascii() {
				return Err(format!(
					"Error on line {n} of {file_name}: line must only be ascii for ease \
					 of parsing."
				));
			}

			//We don't want things like blanks lines or comments to
			//to be upheld to the standards of real lines and have
			//them throw errors
			if l.len() == 0 || l.starts_with("#") {
				return Ok(());
			}

			//Split line into indentifier and actual content
			let (iden, data) : (&str, &str) =
				l.split_once(char::is_whitespace).ok_or_else(|| -> String {
					format!(
						"Error on line {n} of {file_name}: line does not contain \
						 whitespace between the line identifier and its content."
					)
				})?;

			if iden != "f" {
				//Parse a new vector
				let new_v : Vec4 = data.split_whitespace().enumerate().try_fold(
					Vec4::new(0.0, 0.0, 0.0, 1.0),
					|mut new_v : Vec4, (m, s) : (usize, &str)| -> Result<Vec4, String> {
						new_v[m] = *s
							.parse::<f32>()
							.as_ref()
							.map_err(<ParseFloatError as ToString>::to_string)?;
						Ok(new_v)
					},
				)?;

				//Add the new vector to the write category
				match iden {
					"v" => {
						pos.push(new_v);
					},
					"vt" => {
						uv.push(new_v.xyz());
					},
					"vn" => {
						norm.push(new_v.xyz());
					},
					_ => {
						return Err(format!(
							"Error on line {n} of {file_name}: parser only supports \
							 vertices, texture coords, normals and face elements."
						));
					},
				}
			} else {
				//Insert an existing vector into the new mesh
				let new_t : Tri<ObjV> = data.split_whitespace().enumerate().try_fold(
					Tri::<ObjV>::default(),
					|mut t : Tri<ObjV>,
					 (m, s) : (usize, &str)|
					 -> Result<Tri<ObjV>, String> {
						t.0[m] = s.split('/').enumerate().try_fold(
							ObjV::default(),
							|mut ov : ObjV, (o, s) : (usize, &str)| -> Result<ObjV, String> {
								let idx : usize = *s
									.parse::<usize>()
									.as_ref()
									.map_err(<ParseIntError as ToString>::to_string)?;

								let err_f = || -> String {
									format!(
										"Error on line {n} of {file_name}: {} # {idx} does not \
										 exist. NOTE: face elements are parsed as they're \
										 recieved, so the vertex data cannot be defined after the \
										 face element that uses it.",
										match o {
											0 => "position",
											1 => "uv",
											2 => "norm",
											_ => unreachable!(),
										}
									)
								};

								match o {
									0 => {
										ov.pos = *pos.get(idx - 1).ok_or_else(err_f)?;
									},
									1 => {
										ov.uv = *uv.get(idx - 1).ok_or_else(err_f)?;
									},
									2 => {
										ov.norm = *norm.get(idx - 1).ok_or_else(err_f)?;
									},
									_ => {
										return Err(format!(
											"Error on line {n} of {file_name}: face elements data \
											 contains too many '/' characters."
										));
									},
								};

								Ok(ov)
							},
						)?;

						Ok(t)
					},
				)?;
				tri.push(new_t);
			}

			Ok(())
		})?;

	Ok(Mesh::new(tri, Mat4::IDENTITY))
}
