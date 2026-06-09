use glam::{EulerRot, Mat4, Quat, Vec3};

//My justification for having this camera struct that is only
//ever used as a field for the Renderer struct is that some
//day I may want to support having several cameras renedering
//to different things and so we'd have to change the camera
//field to a vector of cameras
pub struct Camera {
	//Camera position
	pub pos : Vec3,
	//Horizonal rotation
	pub yaw : f32,
	//Vertical rotation
	pub pitch : f32,
	//Field of view
	pub fov : f32,
	//Aspect Ratio
	pub aspect_ratio : f32,
	//Near clipping plane
	pub near_plane : f32,
	//Far clipping plane
	pub far_plane : f32,
}

impl Default for Camera {
	fn default() -> Camera {
		Camera::new(
			Vec3::ZERO,
			0.0,
			0.0,
			65_f32.to_radians(),
			4_f32 / 3_f32,
			0.001_f32,
			1000_f32,
		)
	}
}

impl Camera {
	pub fn new(
		pos : Vec3,
		yaw : f32,
		pitch : f32,
		fov : f32,
		aspect_ratio : f32,
		near_plane : f32,
		far_plane : f32,
	) -> Camera {
		Camera {
			pos,
			yaw,
			pitch,
			fov,
			aspect_ratio,
			near_plane,
			far_plane,
		}
	}

	//TODO: Implament proper change detection so we can store the proj
	//and cam mats and simply return those when nothing has changed
	pub fn camera_matrix(self: &Camera) -> Mat4 {
		(Mat4::from_translation(self.pos)
			* Mat4::from_rotation_y(self.yaw)
			* Mat4::from_rotation_x(self.pitch))
		.inverse()
	}

	pub fn projection_matrix(self: &Camera) -> Mat4 {
		Mat4::perspective_lh(
			self.fov,
			self.aspect_ratio,
			self.near_plane,
			self.far_plane,
		)
	}
}
