pub mod buffer;
pub mod gl_extra;
pub mod image;
pub mod shaders;
pub mod small_array;
pub mod tile_glyph;
pub mod vector;

pub use vector::{Vector2, Vector4};

pub type Color = vector::Vector4<f32>;
pub const BLACK: Color = crate::v4s!(0.0);
pub const WHITE: Color = crate::v4s!(1.0);
