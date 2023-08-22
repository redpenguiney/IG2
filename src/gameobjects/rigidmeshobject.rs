use glm::{Vec4, vec4, quat_identity, I64Vec3, Quat, Vec3, vec3};

use crate::{engine::{transform::{Transform, ObjectTransform, i64vec3}, gameobject::GameObject, core::EngineAPIs, graphics::universal::mesh::Mesh, gameobjects::collisions::Collides}, impl_collides, impl_renderable, impl_rigidbody, impl_transform, impl_gameobject};

use super::{renderable::Renderable, collisions::ColliderType, rigidbody::RigidBody};

pub struct RigidMeshObject {
    pub name: String,
    pub transform: Transform,
    color: Vec4,
    texz: f32,
    mesh_id: u32,
    color_changed: bool,
    texz_changed: bool,

    pub density: f32,
    pub friction: f32,
    pub elasticity: f32,
    pub velocity: I64Vec3,
    pub angular_velocity: Vec3,

    collider_type: ColliderType,
    inertia_tensor: Vec3,

}

impl GameObject for RigidMeshObject {
    impl_gameobject!(RigidMeshObject);

    fn update(&mut self, apis: &mut crate::engine::core::EngineAPIs, gameobjects: &mut Vec<Box<dyn GameObject>>) {
        
    }

    fn update_threaded(&mut self, apis: &mut crate::engine::core::EngineAPIs) {
        //println!("SMH i am at {:?}", &self as *const _);
        self.mesh_update(apis);
        // println!("upda");
    }

    fn destroy(&mut self, apis: &mut crate::engine::core::EngineAPIs) {
        
    }
}

impl RigidMeshObject {
    pub fn new(apis: &mut EngineAPIs, mesh: Box<&Mesh>, texture_z: f32, collider_type: ColliderType) -> Self {
        let mut obj = Self { 
            
            name: String::from("RigidMeshObject"),
            mesh_id: apis.mesh_master.add(&mesh, 1),

            transform: Transform::empty(), 
            color: vec4(0.6, 0.6, 0.6, 0.5),
            texz: texture_z,

            color_changed: true,
            texz_changed: true,
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

impl_collides!(RigidMeshObject);
impl_renderable!(RigidMeshObject);
impl_rigidbody!(RigidMeshObject);
impl_transform!(RigidMeshObject);
