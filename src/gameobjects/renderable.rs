use nalgebra_glm::I64Vec3;

pub trait Renderable: GameObject + crate::transform::ObjectTransform {
    fn set_rgba(&mut self, color: nalgebra_glm::Vec4);
    fn set_texture_z(&mut self, texz: f32);
    fn get_rgba(&self) -> nalgebra_glm::Vec4;
    fn get_texture_z(&self) -> f32;
    fn get_color_changed(&self) -> bool;
    fn get_texture_z_changed(&self) -> bool;
    fn set_color_changed(&mut self, changed: bool);
    fn set_texture_z_changed(&mut self, changed: bool);
    fn get_mesh_id(&self) -> usize; 
    fn set_mesh_id(&mut self, id: usize);
    fn get_draw_id(&self) -> usize; 
    fn set_draw_id(&mut self, id: usize);
}

#[macro_export]
macro_rules! impl_renderable {
    ($structname: ident) => {
        
        
        
        impl crate::Renderable for $structname {
            
            
            // fn mesh_update(&mut self, apis: &mut crate::engine::core::EngineAPIs) {
            //     use {crate::engine::gameobjects::renderable::CAMERA_POS};
            //     let cp = CAMERA_POS.load();
            //     let model = self.transform.get_model(&cp);
                
            //     //println!("Getting model elapsed {:?}", now.elapsed());
            //     apis.mesh_master.set_transform(self.mesh_id, 0, &model);
                
            //     if self.color_changed {
            //         apis.mesh_master.set_rgba(self.mesh_id, 0, self.color);
            //     }
            //     if self.texz_changed {
            //         apis.mesh_master.set_texture_z(self.mesh_id, 0, self.texz);
            //     }
            // }
            fn get_mesh_id(&self) -> usize {
                return self.mesh_id;
            }

            fn get_draw_id(&self) -> usize {
                return self.draw_id;
            }

            fn get_color_changed(&self) -> bool {
                return self.color_changed;
            }

            fn get_texture_z_changed(&self) -> bool {
                return self.texture_z_changed;
            }

            fn set_color_changed(&mut self, changed: bool) {
                self.color_changed = changed;
            }

            fn set_texture_z_changed(&mut self, changed: bool) {
                self.texture_z_changed = changed;
            }

            fn set_rgba(&mut self, color: nalgebra_glm::Vec4) {
                self.color = color;
            }

            fn set_mesh_id(&mut self, id: usize) {
                self.mesh_id = id;
            }

            fn set_draw_id(&mut self, id: usize) {
                self.draw_id = id;
            }
        
            fn set_texture_z(&mut self, texz: f32) {
                self.texture_z = texz;
            }

            fn get_rgba(&self) -> nalgebra_glm::Vec4 {
                return self.color;
            }

            fn get_texture_z(&self) -> f32 {
                return self.texture_z;
            }
        }  
    }
}