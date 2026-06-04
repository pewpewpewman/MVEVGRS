use glam::Vec4;

use crate::renderer::Renderer;

//Module for describing user stage functions.
//
//"User stage functions" is really just what I'm calling shaders.
//It's not a good name, but neither is "shader", so.

//There are currently 2 stages that are input by the user.
//
//Vertext Transformer - transforms vertices into clip space.
//
//Pixel Colorer - colors pixels and takes on data output from
//                the vertex transformer.

//The renderer struct takes on 3 type generics
//
//V  - Vertex data, this is the type that makes up meshes. Includes positional data
//     and any other data that you want interpolated along the mesh for use in the pixel
//     colorer.
//
//P  - Pixel coloring data, this is the type that gets interpolated along the mesh and
//     input to the pixel coloring function. This type is required to impl multiplying
//     with the f32 type and adding with itself for interpolation.
//
//UE - User stage function Enviorment, this type is analogous to uniforms in traditional 3d
//     rendering apis. Both user stages take on a reference to the described UE type and is
//     updated each call if an updating function is given.

//These two types contain info the renderer passes to the user function. The info inside is
//some context about the state of rendering like which number vertex is being proccesed or the
//NDC coord of the current pixel being colored.
pub struct VertContext {}
pub struct ColorContext {}

//Vertex transformer function type. This function is applied to every vertex of each triangle.
//Stuff like model-view-projection matrices are applied to mesh vertices here. Vertex data you
//want simply interpolated across tris is also just passed here.
pub type VertexTransformer<V, P, UE> =
	fn(&V, &UE, &VertContext) -> VertTransOut<P>;

//The pixel colorer function. The output of this function is the rbg values drawn to the screen.
//Color values are between 0 and 1. ALpha channel does nothing at the moment.
pub type PixelColorer<P, UE> = fn(&P, &UE, &ColorContext) -> Vec4;

//The enviorment updater. Once a fram this is ran to give the right values for values avilable to
//the user functions.
pub type UserEnvUpdater<V, P, UE> = fn(&Renderer<V, P, UE>) -> UE;

//The output of the vertex transformer.
#[derive(Debug)]
pub struct VertTransOut<P> {
	//This position field is the vertex's position in clip space.
	//PLEASE NOTE!!! THIS VALUE SHOULD **NOT** BE DIVIDED BY W AFTER BEING MULTIPLIED
	//BY THE PROJECTION MATRIX. GLAM HAS A Mat4::project_point3 function. DO **NOT**
	//USE IT, IT DIVIDES XYZ BY W!!
	pub pos : Vec4,
	//Data to be interpolated and passed to the coloring function
	pub colorer_data : P,
}
