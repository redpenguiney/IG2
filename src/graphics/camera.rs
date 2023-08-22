use glm::Mat4;

use crate::transform::Transform;

pub struct Camera {
    pub transform: Transform,
    pub proj_changed: bool,
    proj: Mat4
}

impl Camera {
    pub fn new() -> Self {
        return Self {
            transform: Transform::empty(),
            proj: glm::identity(),
            proj_changed: false,
        }
    }

    pub fn get_proj(&self) -> &glm::Mat4 {
        return &self.proj;
    }

    pub fn perspective(&mut self, screen_width_over_height: f32, fovy: f32, near: f32, far: f32) {
        self.proj = glm::perspective(screen_width_over_height, fovy, near, far);
        self.proj_changed = true;
    }

    pub fn orthro(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
        self.proj = glm::ortho(left, right, bottom, top, near, far);
        self.proj_changed = true;
    }
} 