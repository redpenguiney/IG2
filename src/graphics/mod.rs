// this little trick lets us basically have multiple files in same crate so we don't have really long namespaces
use libc::{c_uint, c_float, c_int, c_void};
pub type GLuint = c_uint;
pub type GLfloat = c_float;
pub type GLsizei = isize;
pub type GLint = c_int;

pub use gl_error_checking::*;
pub use graphics_engine::*;
pub use mesh::*;
pub use meshpool::*;
pub use framebuffer::*;
pub use camera::*;
pub use shader_program::*;
pub use texture::*;
mod gl_error_checking;
mod graphics_engine;
mod mesh;
mod meshpool;
mod framebuffer;
mod camera;
mod shader_program;
mod texture;