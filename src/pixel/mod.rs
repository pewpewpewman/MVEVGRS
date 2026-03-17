// Represents pixels, pixels that are
// probably on screen, I'm even willing
// to bet the screen is the one right in
// front of you. Values should only
// range from 0.0..=1.0

use glam::Vec4;

//I really want Pixels to be backed by glam's very advanced and fancy Vec4 type, but that comes at
//the cost of not being able to use .rgba, we have to use .xyzw, quite sad :(
pub type Pixel = Vec4;
