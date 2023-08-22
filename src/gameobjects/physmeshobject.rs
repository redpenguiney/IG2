use crate::transform::*;
use glm::{Vec4, vec4};

use crate::{impl_renderable, impl_transform, impl_collides, impl_gameobject};

use super::{renderable::Renderable, collisions::ColliderType};

pub struct PhysMeshObject {
    pub name: String,
    pub transform: Transform,
    color: Vec4,
    texz: f32,
    mesh_id: u32,
    color_changed: bool,
    texz_changed: bool,

    collider_type: ColliderType,

    pub friction: f32,
    pub elasticity: f32,
    
}

impl GameObject for PhysMeshObject {
    impl_gameobject!(PhysMeshObject);

    fn update_threaded(&mut self, apis: &mut EngineAPIs) {

        self.mesh_update(apis);
    }

    fn update(&mut self, apis: &mut EngineAPIs, gameobjects: &mut Vec<Box<dyn GameObject>>) {
        // let touching = apis.sas.borrow_mut().query_aabb(&AABB::new(&self.transform));
        // for ptr in touching {

        // }
    }

    fn destroy(&mut self, apis: &mut EngineAPIs) {
        
    }
}

impl PhysMeshObject {
    pub fn new(apis: &mut EngineAPIs, mesh: Box<&Mesh>, texture_z: f32, collider_type: ColliderType) -> Self {
        let mut obj = Self { 
            name: String::from("PhysMeshObject"),
            mesh_id:  apis.mesh_master.add(&mesh, 1),

            transform: Transform::empty(), 
            color: vec4(0.6, 0.6, 0.6, 0.5),
            texz: texture_z,

            color_changed: true,
            texz_changed: true,
            collider_type: collider_type,

            elasticity: 0.4,
            friction: 0.4,
        };
        
        return obj;
    }
}

impl_renderable!(PhysMeshObject);
impl_transform!(PhysMeshObject);
impl_collides!(PhysMeshObject);