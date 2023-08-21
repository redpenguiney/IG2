// this little trick lets us basically have multiple files in same crate so we don't have really long namespaces
use libc::{c_uint, c_float, c_int, c_void};
type GLuint = c_uint;
type GLfloat = c_float;
type GLsizei = isize;
type GLint = c_int;

include!("gl_error_checking.rs");
include!("camera.rs");
include!("mesh.rs");
include!("texture.rs");
include!("framebuffer.rs");
include!("shader_program.rs");
include!("meshpool.rs");
include!("graphics_engine.rs");