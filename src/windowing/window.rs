use glfw::*;

pub struct Window {
    pub name: String,
    pub size: (u32, u32),

    glfw_window: glfw::Window,
    glfw_instance: glfw::Glfw,
    event_reciever: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,

    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_dx: f64,
    pub mouse_dy: f64
}

impl Window {
    pub fn new(name: String) -> Self {
        let mut instance = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        instance.window_hint(glfw::WindowHint::ContextVersion(4, 6)); //todo: no dis bad
        instance.window_hint(glfw::WindowHint::Samples(Some(4)));
        let (mut window, events) = instance.create_window(500, 500, "iron_game test", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);
        window.make_current();
        
        instance.set_swap_interval(glfw::SwapInterval::Adaptive); // VSync

        let pos: (f64, f64) = window.get_cursor_pos();
        return Self {
            name: name,
            size: (500, 500),

            glfw_window: window,
            glfw_instance: instance,
            event_reciever: events,

            mouse_x: pos.0,
            mouse_y: pos.1,
            mouse_dx: 0.0,
            mouse_dy: 0.0
        };
    }

    pub fn create_opengl_context(&self) -> gl46::GlFns {
        let result: Result<gl46::GlFns, &str> = unsafe { gl46::GlFns::load_from(&|u8_ptr|{ 
            let str = std::ffi::CStr::from_ptr(u8_ptr as *const i8);      
            let mut address = self.glfw_instance.get_proc_address_raw(str.to_str().unwrap());
            if address.is_null() {
                println!("WARNING: a OpenGL function's proc address could not be found!");
                address = 0x11111 as *const libc::c_void;
            }
            return address;
        })};

        if result.is_err() {
            panic!("Failure to create opengl context.");
        }
        else {
            return result.unwrap();
        }
    }

    pub fn should_close(&self) -> bool {
        return self.glfw_window.should_close();
    }

    // polls events and handles screen resize
    pub fn update(&mut self) {
        let size = self.glfw_window.get_size();
        self.size = (size.0 as u32, size.1 as u32);
        let pos = self.glfw_window.get_pos();
        // // OpenGL needs to know if the window changed size/position
        // if cfg!(not(target_os = "macos")) {
        //     unsafe {
        //         gl.as_ref().unwrap().Viewport(0, 0, size.0, size.1);
        //     }
        // }
        
        let pos = self.glfw_window.get_cursor_pos();
        self.mouse_dx = pos.0 - self.mouse_x;
        self.mouse_dy = pos.1 - self.mouse_y;
        self.mouse_x = pos.0;
        self.mouse_y = pos.1;
        self.glfw_instance.poll_events();

        // let mut press_began_keys = INPUT.press_began_keys.write().unwrap();
        // let mut press_ended_keys = INPUT.press_ended_keys.write().unwrap();
        // let mut pressed_keys = INPUT.pressed_keys.write().unwrap();

        // press_began_keys.clear();
        // press_ended_keys.clear();
        for (_, event) in glfw::flush_messages(&self.event_reciever) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.glfw_window.set_should_close(true);
                    
                },
                glfw::WindowEvent::Key(Key, _, glfw::Action::Press, _) => {
                    // press_began_keys.insert(Key, true);
                    // pressed_keys.insert(Key, true);
                    //println!("Key {:?} was pressed", Key);
                },
                glfw::WindowEvent::Key(Key, _, glfw::Action::Release, _) => {
                    // press_ended_keys.insert(Key, true);
                    // pressed_keys.insert(Key, false);
                    //println!("Key {:?} was released", Key);
                },
                _ => {}
            }
        }
    }

    pub fn swap_buffers(&mut self) {
        self.glfw_window.swap_buffers();
    }

    pub fn set_title(&mut self, title: &String) {
        
        self.glfw_window.set_title(title);
    }

    // pub fn set_fullscreen(&mut self, state: bool) {
    //     // TODO
    // }

    pub fn cleanup(&mut self) {
        
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // Todo maybe? Not sure what is going to be put in here
    }
}

impl rlua::UserData for Window {

}