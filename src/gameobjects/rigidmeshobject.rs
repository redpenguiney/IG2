use glm::{Vec4, vec4, I64Vec3, Vec3, vec3};

use crate::transform::*;
use crate::gameobjects::*;

pub struct RigidMeshObject {
    pub name: String,
    pub transform: Transform,
    color: Vec4,
    texture_z: f32,
    mesh_id: usize, // uuid of Mesh, so we know when two cube meshes/etc. are the same and can be instanced
    draw_id: usize, //uuid for this individual object that will be drawn so it can be removed
    color_changed: bool,
    texture_z_changed: bool,

    pub density: f32,
    pub friction: f32,
    pub elasticity: f32,
    pub velocity: I64Vec3,
    pub angular_velocity: Vec3,

    collider_type: ColliderType,
    inertia_tensor: Vec3,

}

impl RigidMeshObject {
    pub fn new(mesh_id: usize, collider_type: ColliderType) -> Self {
        let mut obj = Self { 
            
            name: String::from("RigidMeshObject"),
            mesh_id: mesh_id,
            draw_id: 0,

            transform: Transform::empty(), 
            color: vec4(0.6, 0.6, 0.6, 0.5),
            texture_z: -1.0,

            color_changed: true,
            texture_z_changed: true,
            collider_type: collider_type,

            density: 1.0,
            friction: 0.4,
            elasticity: 0.3,
            velocity: i64vec3(0, 0, 0),
            angular_velocity: vec3(0.0, 0.0, 0.0),
            
            inertia_tensor: vec3(0.0, 0.0, 0.0),
        };

        
        
        println!("I AM AT {:?}", &mut obj as *mut _);
        println!("\tDEREF THAT AND POS IS {:?}", unsafe{&*(&mut obj as *mut dyn Collides)}.transform().pos());
        //println!("BUT AS RIGIDBODY I AM AT {:?}", &mut obj as *mut dyn RigidBody);
        
        
        return obj;
    }
}

crate::impl_gameobject!(RigidMeshObject);
crate::impl_collides!(RigidMeshObject);
crate::impl_renderable!(RigidMeshObject);
crate::impl_rigidbody!(RigidMeshObject);
crate::impl_transform!(RigidMeshObject);
