
// TODO: right now only vertex + fragment supported, a tesselation shader would be nice for LOD or geometry shader for idk

use std::fs;

use gl46::*;
use crate::graphics::*;

struct Shader {
    shader_type : GLenum,
    gl_shader : GLuint
}

impl Shader {
    fn new(gl: &gl46::GlFns, s_type: GLenum, path: &'static str) -> Self {
        unsafe {
            // // check_graphics_errors(gl);
            let shader: u32 = gl.CreateShader(s_type); 

            let result = fs::read_to_string(path);
            if result.is_err() {
                panic!("Failure to find or read shader file located at {}", path);
            }

            let source = result.unwrap();
            let length : GLint = source.len().try_into().unwrap();
            let ptr = source.as_ptr();
            let ptr_to_ptr: *const *const u8 = &ptr;

            let mut compile_success : GLint = -1;

            gl.ShaderSource(shader, 1, ptr_to_ptr, &length as *const GLint);
            gl.CompileShader(shader); 
            gl.GetShaderiv(shader, GL_COMPILE_STATUS, &mut compile_success as *mut i32); 
            if compile_success == 0 {

                let length: GLint = 0;
                let written: GLint = 0;
                gl.GetShaderiv(shader, GL_INFO_LOG_LENGTH, &length as *const i32 as *mut i32);

                if length > 0
                {
                    let mut info_log: Vec<u8> = Vec::new();
                    info_log.resize(length as usize, 0);
                    gl.GetShaderInfoLog(shader, length, &written as *const i32 as *mut i32, info_log.as_mut_ptr());
                    println!("Shader Info Log:\n{}", String::from_utf8_lossy(&info_log));
                };
                panic!("Shader compilation failure for file {}", path);
            }

            return Self {
                shader_type : s_type,
                gl_shader : shader
            }
        }    
    }

    fn cleanup(&self, gl: &gl46::GlFns) {
        gl.DeleteShader(self.gl_shader);
    }
}

pub struct ShaderProgram {
    pub program : GLuint,
    vertex : Shader,
    fragment : Shader,
    uniform_locations : std::collections::HashMap<String, GLint>,
    pub auto_proj: bool, // if these are true, the Camera struct will automatically fill the "proj" and "camera" uniforms respectively
    pub auto_cam: bool, // keep false if its a gui or something idk

    pub cast_shadows: bool,
    pub shadowmap_texture_index: i32, // automatically set to index of string "shadowmap" in texture_names argument of constructor, -1 if not there
                                      // used by mesh_master.rs to make sure this program gets its shadow texture if it needs it

}

impl ShaderProgram {
    pub fn new(gl: &gl46::GlFns, vertex_path : &'static str, fragment_path : &'static str, texture_names: Vec<&str>) -> Self {
        unsafe {
            assert!(texture_names.len() <= 8);
            //println!("Making program.");
            let program = gl.CreateProgram();
            let vertex = Shader::new(gl, GL_VERTEX_SHADER, vertex_path); 
            let fragment= Shader::new(gl, GL_FRAGMENT_SHADER, fragment_path); 
            gl.AttachShader(program, vertex.gl_shader); 
            gl.AttachShader(program, fragment.gl_shader);
            gl.BindFragDataLocation(program, 0, std::ffi::CStr::from_bytes_with_nul_unchecked(b"color").as_ptr() as *const u8); 
            gl.LinkProgram(program);

            let mut success : GLint = -1;
            gl.GetProgramiv(program, GL_LINK_STATUS, &success as *const i32 as *mut i32); 
            if success == 0 {
                panic!("Failed to link shader program!");
            } 

            gl.UseProgram(program);

            // Tell OpenGL which texture binding locations go to which variables
            let mut shadowmap_index = -1;
            for i in 0..texture_names.len() {
                if texture_names[i] == "shadowmap" {shadowmap_index = i as i32}
                let location = gl.GetUniformLocation(program, (texture_names[i].to_owned() + "\0").as_ptr());
                gl.Uniform1i(location, i as i32); //println!("uniform{}",i);
            }
            //println!("Shadowmap at {}", shadowmap_index);

            return Self {
                program : program,
                vertex : vertex,
                fragment : fragment,
                uniform_locations : std::collections::hash_map::HashMap::new(),
                auto_cam: true,
                auto_proj: true,
                cast_shadows: true,

                shadowmap_texture_index: shadowmap_index
            }
        }

    }

    pub fn matrix4x4(&mut self, gl: &gl46::GlFns, uniform_name: &String, matrix: &nalgebra_glm::Mat4, transpose: bool) {
        //println!("Matrix is {:?}", matrix);
        unsafe {
            self.r#use(gl);
            if !self.uniform_locations.contains_key(uniform_name) {
                self.uniform_locations.insert(uniform_name.clone(), gl.GetUniformLocation(self.program, (uniform_name.to_owned() + "\0").as_ptr() as *const u8));
            }   
            let location = self.uniform_locations[uniform_name];
            // check_graphics_errors(gl);
            gl.UniformMatrix4fv(location, 1, transpose as u8, matrix.as_ptr());
            // check_graphics_errors(gl);
        }
    }

    pub fn r#use(&self, gl: &gl46::GlFns) {
        unsafe {
            gl.UseProgram(self.program);
            // check_graphics_errors(gl);
        }
    }

    pub fn cleanup(&self, gl: &gl46::GlFns) {
        unsafe { 
            gl.DeleteProgram(self.program);
        }
    }
     
}