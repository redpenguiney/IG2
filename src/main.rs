#![allow(non_snake_case)]

use crate::gameobjects::{GameObject, Renderable};

mod graphics;
mod windowing;
mod gameobjects;
fn main() {
    application();
}

fn application() {
    println!("Initializing application.");

    let mut GAMEOBJECTS: Vec<Box<dyn GameObject>> = Vec::new();
    let mut MESHOBJECTS: Vec<Box<dyn Renderable>> = Vec::new();
    let mut GE = graphics::GraphicsEngine::new();
    let mut WINDOW = windowing::Window::new(String::from("POG"));

    println!("Starting main loop");
    
    while !WINDOW.should_close() {
        WINDOW.update();



        WINDOW.swap_buffers(); // will block because vsync
    }
    
    println!("Cleaning up.");

    GE.cleanup();
    WINDOW.cleanup();

    println!("Application ran successfully! :)")
}