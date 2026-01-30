// Represents pixels, pixels that are
// probably on screen, I'm even willing
// to bet the screen is the one right in
// front of you. Values should only
// range from 0.0..=1.0

#[derive(Clone, Default)]

pub struct Pixel {
	pub r : f32,
	pub g : f32,
	pub b : f32,
	pub a : f32,
}

impl Pixel {
	pub fn new(
		r : f32,
		g : f32,
		b : f32,
		a : f32,
	) -> Pixel {
		Pixel {
			r,
			g,
			b,
			a,
		}
	}
}
