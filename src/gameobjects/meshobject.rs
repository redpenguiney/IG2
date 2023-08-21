use crate::transform::Transform;
use nalgebra_glm::{*};

pub struct MeshObject {
    pub name: String,
    pub transform: Transform,
    color: Vec4,
    texture_z: f32,
    mesh_id: usize, // uuid of Mesh, so we know when two cube meshes/etc. are the same and can be instanced
    draw_id: usize, //uuid for this individual object that will be drawn so it can be removed
    color_changed: bool,
    texture_z_changed: bool,
}

impl MeshObject {
    pub fn new(mesh_id: usize) -> Self {
        let obj = Self { 
            name: String::from("MeshObject"),
            mesh_id: mesh_id,
            draw_id: 0,

            transform: Transform::empty(), 
            color: vec4(0.6, 0.6, 0.6, 0.5),
            texture_z: -1.0,

            color_changed: true,
            texture_z_changed: true,
        };
        return obj;
    }
}

impl_gameobject!(MeshObject);
impl_renderable!(MeshObject);
crate::impl_transform!(MeshObject);