// Checks if there's an openGL error and panics if yes
// pub fn check_graphics_errors(gl: &GlFns) {
//     unsafe {
//         let error = gl.GetError();
//         //println!("Error check got error");
//         if error != GL_NO_ERROR {
//             //let message = gl.as_ref().unwrap().GetDebugMessageLog()
//             panic!("OpenGL error {}", error.0);
//         }
//     }
// }

use gl46::*;
use crate::graphics::*;
// opengl calls this when error
pub unsafe extern  "system" fn opengl_debug_callback(source: GLenum, _error_type: GLenum, _id: GLuint, _severity: GLenum, length: i32, message: *const u8, _userdata: *const c_void) {
    panic!("an OpenGL error:\n\tSource: {:?}, \n\tMessage: {}", source, String::from_raw_parts(message.cast_mut(), length as usize, 1024))
}