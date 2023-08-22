use crate::transform::*;
use glm::{Vec4, vec4};

use crate::{impl_renderable, impl_transform, impl_collides, impl_gameobject};

use super::{renderable::Renderable, collisions::ColliderType};

pub struct PhysMeshObject {
    pub name: String,
    pub transform: Transform,
    color: Vec4,
    texture_z: f32,
    mesh_id: usize, // uuid of Mesh, so we know when two cube meshes/etc. are the same and can be instanced
    draw_id: usize, //uuid for this individual object that will be drawn so it can be removed
    color_changed: bool,
    texture_z_changed: bool,

    collider_type: ColliderType,

    pub friction: f32,
    pub elasticity: f32,
    
}

impl PhysMeshObject {
    pub fn new(mesh_id: usize, collider_type: ColliderType) -> Self {
        let mut obj = Self { 
            name: String::from("PhysMeshObject"),
            mesh_id:  mesh_id,
            draw_id: 0, 

            transform: Transform::empty(), 
            color: vec4(0.6, 0.6, 0.6, 0.5),
            texture_z: -1.0,

            color_changed: true,
            texture_z_changed: true,
            collider_type: collider_type,

            elasticity: 0.4,
            friction: 0.4,
        };
        
        return obj;
    }
}

impl_gameobject!(PhysMeshObject);
impl_renderable!(PhysMeshObject);
impl_transform!(PhysMeshObject);
impl_collides!(PhysMeshObject);