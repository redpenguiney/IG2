#![allow(non_snake_case)]

extern crate nalgebra_glm as glm;

use std::{cell::RefCell, rc::Rc};

use glm::vec3;

use crate::{gameobjects::{GameObject, Renderable}, graphics::Mesh, transform::dvec3};

mod transform;
mod graphics;
mod windowing;
mod gameobjects;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    application();
}

fn application() {
    println!("Initializing application.");
    //let mut GAMEOBJECTS: Vec<Rc<RefCell<dyn GameObject>>> = Vec::new();
    let mut WINDOW = windowing::Window::new(String::from("POG"));
    let mut GE = graphics::GraphicsEngine::new(WINDOW.create_opengl_context(), WINDOW.size as (u32, u32));
    GE.freecam_override_enabled = true;

    println!("Starting main loop");

    let mesh = Mesh::from_obj("models/icosphere.obj", 0, GE.world_shader_id);
    let test = Rc::new(RefCell::new(gameobjects::MeshObject::new(mesh)));
    GE.add_renderable(test.clone());
    //GAMEOBJECTS.push(test.clone());
    
    GE.camera.transform.setpos_meters(dvec3(0.0, 0.0, 10.0));

    while !WINDOW.should_close() {
        WINDOW.update();

        GE.update();
        GE.draw();

        WINDOW.swap_buffers(); // will block because vsync
    }
    
    println!("Cleaning up.");

    GE.cleanup();
    WINDOW.cleanup();

    println!("Application ran successfully! :)")
}