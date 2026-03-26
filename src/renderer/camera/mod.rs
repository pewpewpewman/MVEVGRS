use glam::Mat4;
pub struct Camera {
	//Translates points in 3d world space into 3d camera space. Note that the camera matrix is not
	//inverted before application, so if you move or rotate it, make sure you're applying the
	//inverse of those transformations
	pub camera_mat : Mat4,
	//Translates points in 3d camera space into 2d screen space
	pub proj_mat : Mat4,
	//Near clipping plane
	pub near_plane : f32,
}

impl Default for Camera {
	fn default() -> Camera {
		let near_plane : f32 = 0.1_f32;
		Camera {
			camera_mat : Mat4::IDENTITY,
			proj_mat : Mat4::perspective_infinite_lh(
				80_f32.to_radians(),
				16_f32 / 9_f32,
				near_plane,
			),
			near_plane,
		}
	}
}
