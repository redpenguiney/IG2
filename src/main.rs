#![allow(non_snake_case)]

extern crate nalgebra_glm as glm;

use std::{cell::RefCell, rc::Rc};

use crate::{gameobjects::{GameObject, Renderable}, graphics::Mesh};

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
    let mut GE = graphics::GraphicsEngine::new(WINDOW.create_opengl_context());

    println!("Starting main loop");

    let mesh = Mesh::from_obj("models/icosphere.obj", 0, GE.world_shader_id);
    let test = Rc::new(RefCell::new(gameobjects::MeshObject::new(mesh)));
    GE.add_renderable(test.clone());
    //GAMEOBJECTS.push(test.clone());
    
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