use stb_image::image::*;
use gl46::*;
use crate::graphics::*;

#[derive(PartialEq)]
pub enum TextureType {
    TexArray2D,
    Tex2D,
}

pub struct Texture {
    pub gl_texture: GLuint,
    tex_type: TextureType,
    pub size: (i32, i32),
}

// TODO: Options to disable tiling and use GL_NEAREST instead of GL_LINEAR
impl Texture {
    pub fn from_file(gl: &GlFns, path: &str, ttype: TextureType) -> Self {
        // check_graphics_errors(gl);
        unsafe {
            let result = load(path.clone());
            match result {
                LoadResult::Error(message) => {panic!("Failure to load image with path {}, loader said {}", path, message);}
                LoadResult::ImageU8(image) => {
                    //println!("ok");
                    
                    let mut tex: u32 = 0;
                    gl.GenTextures(1, &tex as *const u32 as *mut u32); // check_graphics_errors(gl);
                    let target = if ttype == TextureType::Tex2D {GL_TEXTURE_2D} else {GL_TEXTURE_2D_ARRAY};
                    //figure out if its rgba or rgb by calculating how many many bytes per pixel
                    let source_format = if image.width * image.height * 4 == image.data.len() {GL_RGBA} else {GL_RGB};

                    // check_graphics_errors(gl);
                    gl.BindTexture(target, tex); 
                    // check_graphics_errors(gl);
                    if target == GL_TEXTURE_2D {
                        gl.TexImage2D(target, 0, GL_RGBA.0 as i32, image.width as i32, image.height as i32, 0, GL_RGBA, GL_UNSIGNED_BYTE, image.data.as_ptr() as *const c_void);
                    }     
                    else {
                        gl.TexStorage3D(target, 1, GL_RGBA8, image.width as i32, image.width as i32, (image.height/image.width) as i32);
                        // check_graphics_errors(gl);
                        gl.TexSubImage3D(target, 0, 0, 0, 0, image.width as i32, image.width as i32, (image.height/image.width) as i32, source_format, GL_UNSIGNED_BYTE, image.data.as_ptr() as *const c_void);
                    }               
                    // check_graphics_errors(gl);

                    gl.TexParameteri(target, GL_TEXTURE_WRAP_S, GL_REPEAT.0 as i32);	
                    gl.TexParameteri(target, GL_TEXTURE_WRAP_T, GL_REPEAT.0 as i32);
                    gl.TexParameteri(target, GL_TEXTURE_MIN_FILTER, GL_LINEAR.0 as i32);
                    gl.TexParameteri(target, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);

                    let texture = Self {
                        gl_texture: tex,
                        tex_type: ttype,
                        size: (image.width as i32, image.height as i32)
                    };

                    return texture;
                }
                LoadResult::ImageF32(..) => {panic!("bruh shut up image i dont want floats at path {}", path);}
            }

            
        }
    }

    pub fn empty_depth(gl: &GlFns, width: u32, height: u32) -> Self {
        let mut tex: u32 = 0;
        unsafe{
            gl.GenTextures(1, &tex as *const u32 as *mut u32);
            gl.BindTexture(GL_TEXTURE_2D, tex);
            gl.TexImage2D(GL_TEXTURE_2D, 0,GL_DEPTH_COMPONENT32.0 as i32, width as i32, height as i32, 0,GL_DEPTH_COMPONENT, GL_FLOAT, std::ptr::null());
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_BORDER.0 as i32);	
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_BORDER.0 as i32);
            gl.TexParameterfv(GL_TEXTURE_2D, GL_TEXTURE_BORDER_COLOR, [1.0, 1.0, 1.0, 1.0].as_ptr());
        }
        return Self {
            gl_texture: tex,
            tex_type: TextureType::Tex2D,
            size: (width as i32, height as i32)
        };

    }

    pub fn empty_color(gl: &GlFns, width: u32, height: u32, multisampled: bool) -> Self {
        let mut tex: u32 = 0;
        unsafe{
            gl.GenTextures(1, &tex as *const u32 as *mut u32);
            gl.BindTexture(GL_TEXTURE_2D, tex);
            gl.TexImage2D(GL_TEXTURE_2D, 0, GL_RGBA.0 as i32, width as i32, height as i32, 0,GL_RGBA, GL_UNSIGNED_BYTE, std::ptr::null());
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
            gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);
        }
        return Self {
            gl_texture: tex,
            tex_type: TextureType::Tex2D,
            size: (width as i32, height as i32)
        };
    }
  
    // location is int from 0 to 7 (inclusive), use different locations to bind multiple textures at once
    pub fn r#use(&self, gl: &GlFns, location: u32) {
        unsafe {gl.ActiveTexture(gl46::GLenum(GL_TEXTURE0.0 + location))}
        if self.tex_type == TextureType::Tex2D {
            unsafe {
                //println!("location {}", location);
                gl.BindTexture(GL_TEXTURE_2D, self.gl_texture);
            }
        }
        else {
            unsafe {
                gl.BindTexture(GL_TEXTURE_2D_ARRAY, self.gl_texture);
            }
        }
    }

    pub fn cleanup(&self, gl: &GlFns) {
        unsafe {
            gl.DeleteTextures(1, &self.gl_texture);
        }
    }
}

