use arc_swap::ArcSwap;
use glm::I64Vec3;

use crate::engine::{gameobject::GameObject, transform::{ObjectTransform, i64vec3}, core::EngineAPIs};

// use super::meshobject::CAMERA_POS;

lazy_static! {
    pub static ref CAMERA_POS: ArcSwap<I64Vec3> = ArcSwap::from_pointee(i64vec3(0, 0, 0)); // using arcswap instead of rwlock was supposed to be faster, no effect lol
}

pub trait Renderable: GameObject + ObjectTransform {
    fn mesh_update(&mut self, apis: &mut EngineAPIs);

    fn set_rgba(&mut self, color: glm::Vec4);

    fn set_texture_z(&mut self, texz: f32);
}

#[macro_export]
macro_rules! impl_renderable {
    ($structname: ident) => {
        
        
        
        impl crate::Renderable for $structname {
            
            
            fn mesh_update(&mut self, apis: &mut crate::engine::core::EngineAPIs) {
                use {crate::engine::gameobjects::renderable::CAMERA_POS};
                let cp = CAMERA_POS.load();
                let model = self.transform.get_model(&cp);
                
                //println!("Getting model elapsed {:?}", now.elapsed());
                apis.mesh_master.set_transform(self.mesh_id, 0, &model);
                
                if self.color_changed {
                    apis.mesh_master.set_rgba(self.mesh_id, 0, self.color);
                }
                if self.texz_changed {
                    apis.mesh_master.set_texture_z(self.mesh_id, 0, self.texz);
                }
            }

            fn set_rgba(&mut self, color: glm::Vec4) {
                self.color = color;
            }
        
            fn set_texture_z(&mut self, texz: f32) {
                self.texz = texz;
            }
        }  
    }
}