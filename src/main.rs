#![allow(non_snake_case)]

extern crate nalgebra_glm as glm;

use std::{cell::RefCell, rc::Rc};

use glm::{vec3, vec4};

use crate::{gameobjects::{GameObject, Renderable, RigidBody, Collides, PhysMeshObject}, graphics::{Mesh, Texture}, transform::dvec3};

mod transform;
mod graphics;
mod windowing;
mod gameobjects;
mod phys;

pub const WINDOW_NAME: &str = "IG2";

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    application();
}

// URGENT TODO: some structure for rapid removal of gameobjects from these vectors
fn application() {
    println!("Initializing application.");
    let mut RIGIDBODIES: Vec<Rc<RefCell<dyn RigidBody>>> = Vec::new();
    let mut SAS = phys::SpatialAccelerationStructure::new();
    let mut WINDOW = windowing::Window::new(String::from("POG"));
    let mut GE = graphics::GraphicsEngine::new(WINDOW.create_opengl_context(), WINDOW.resolution as (u32, u32));
    GE.freecam_override_enabled = true;

    println!("Starting main loop");

    let (grass_id, _size) = GE.load_texture_from_file("textures/grass.png", graphics::TextureType::Tex2D);
    let mesh = Mesh::from_obj("models/icosphere.obj", grass_id, GE.world_shader_id);
    for x in -10..10 {
        for y in -10..10 {
            for z in 5..25 {
                let test = Rc::new(RefCell::new(gameobjects::MeshObject::new(mesh)));
                GE.add_renderable(test.clone());
                test.borrow_mut().transform.setpos_meters(dvec3(x as f64 * 3.0, y as f64 * 3.0, z as f64 * 3.0));
            }
        }
    }

    
    let mesh2 = Mesh::from_obj("models/rainbowcube.obj", grass_id, GE.world_shader_id);
    let floor = Rc::new(RefCell::new(PhysMeshObject::new(mesh2, gameobjects::ColliderType::Box)));
    floor.borrow_mut().set_rgba(vec4(0.4, 0.6, 0.4, 1.0));
    floor.borrow_mut().transform.setpos_meters(dvec3(0.0, -10.0, 0.0));
    floor.borrow_mut().transform.setscl(vec3(10.0, 1.0, 10.0));
    SAS.insert(floor.clone());
    GE.add_renderable(floor);
    
    //GAMEOBJECTS.push(test.clone());
    
    //GE.camera.transform.setpos_meters(dvec3(0.0, 0.0, 10.0));

    while !WINDOW.should_close() {
        WINDOW.update();

        phys::do_physics(&mut SAS, &RIGIDBODIES);

        GE.update(WINDOW.resolution);
        GE.draw();

        WINDOW.swap_buffers(); // will block because vsync
    }
    
    println!("Cleaning up.");

    GE.cleanup();
    WINDOW.cleanup();

    println!("Application ran successfully! :)")
}