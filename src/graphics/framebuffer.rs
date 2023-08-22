use gl46::{GL_FRAMEBUFFER, GL_FRAMEBUFFER_COMPLETE, GL_DEPTH_ATTACHMENT, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, GL_DEPTH_BUFFER_BIT, GL_COLOR_BUFFER_BIT, GL_NONE};
use crate::graphics::*;

// Attach textures and depth buffers to framebuffer, and then render to framebuffer
// Needed for lighting, tv screens, postprocessing, etc.
pub struct Framebuffer {
   pub gl_framebuffer: u32,
   pub gl_depth_renderbuffer: u32, // not to be confused with the depth texture we're writing to, this just allows for depth testing while drawing, although not sure if i actually need it yet
   pub width: u32,
   pub height: u32,
   pub depth: Option<Texture>,
   pub color: Option<Texture>,
}

// TODO: framebuffer won't be happy if windows gets resized, maybe
impl Framebuffer {
    pub fn new(gl: &gl46::GlFns, xsize: u32, ysize: u32, color: bool, depth: bool) -> Self {
        unsafe {
            // check_graphics_errors();

            let mut buffer: u32 = 0; // yes it does need to be mutable 
            gl.GenFramebuffers(1, &mut buffer as *mut u32);
            gl.BindFramebuffer(GL_FRAMEBUFFER, buffer);
            //println!("buffer is now {}", buffer);

            let renderbuffer: u32 = 0;
            // gl.GenRenderbuffers(1, &renderbuffer as *const u32 as *mut u32);
            // gl.BindRenderbuffer(GL_RENDERBUFFER, renderbuffer);
            // gl.RenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT, xsize as i32, ysize as i32);
            // gl.FramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, renderbuffer);

            // should probably be consistent and only use framebuffertexture2d, but whateves
            let mut color_comp = None;
            if color {
                color_comp = Some(Texture::empty_color(gl, xsize, ysize, false));
                //println!("test{}", color_comp.as_ref().unwrap().gl_texture);
                // check_graphics_errors();
                gl.FramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, color_comp.as_ref().unwrap().gl_texture.clone(), 0); 
                // check_graphics_errors();
            };
            let mut depth_comp = None;
            if depth {
                // check_graphics_errors();
                depth_comp = Some(Texture::empty_depth(gl, xsize, ysize));
                gl.FramebufferTexture(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, depth_comp.as_ref().unwrap().gl_texture, 0); // check_graphics_errors();
            };

            if gl.CheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                panic!("Failure to create framebuffer. So sad.");
            }
            // check_graphics_errors();

            if color == false {
                gl.DrawBuffer(GL_NONE);
                gl.ReadBuffer(GL_NONE);
            }

            gl.BindFramebuffer(GL_FRAMEBUFFER, 0);

            // check_graphics_errors();

            return Self {
                gl_framebuffer: buffer,
                gl_depth_renderbuffer: renderbuffer,
                width: xsize,
                height: ysize,
                color: color_comp,
                depth: depth_comp
            };

        };
    }

    
    pub fn blit() {

    }

    // All draw calls will render to this buffer after you call this
    pub fn begin_render(&self, gl: &gl46::GlFns) {
        unsafe {
            // check_graphics_errors();
            gl.BindFramebuffer(GL_FRAMEBUFFER, self.gl_framebuffer);
            // check_graphics_errors();
            gl.Clear(GL_DEPTH_BUFFER_BIT|GL_COLOR_BUFFER_BIT);
            // check_graphics_errors();
            gl.Viewport(0, 0, self.width as i32, self.height as i32);
            // check_graphics_errors();
        }
    }

    pub fn begin_render_at_pos(&self, gl: &gl46::GlFns, x: i32, y: i32, xsize: u32, ysize: u32) {
        assert!(xsize <= self.width && ysize <= self.height, "Invalid values for size in begin_render_at_pos()");

        unsafe {
            // check_graphics_errors();
            gl.BindFramebuffer(GL_FRAMEBUFFER, self.gl_framebuffer);
            // check_graphics_errors();
            gl.Clear(GL_DEPTH_BUFFER_BIT|GL_COLOR_BUFFER_BIT);
            // check_graphics_errors();
            gl.Viewport(x, y, xsize as i32, ysize as i32);
            // check_graphics_errors();
        }
    }

    // Stop rendering to framebufer
    pub fn finish_render(&self, gl: &gl46::GlFns, resX: u32, resY: u32) {
        unsafe {
            gl.BindFramebuffer(GL_FRAMEBUFFER, 0);
            gl.Viewport(0, 0, resX as i32, resY as i32);
            // check_graphics_errors();
        }
    }

    pub fn cleanup(&self, gl: &gl46::GlFns) {
        unsafe {
            gl.DeleteFramebuffers(1, &self.gl_framebuffer);
        }
    }

}