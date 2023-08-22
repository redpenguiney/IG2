use crate::Renderable;
use crate::graphics::*;
use crate::windowing::*;
use crate::transform::*;
use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use gl46::*;
use glm::Vec2;
use glm::vec2;
use glfw::Key;
use glm::vec3;

// there are many different uuids here so for clarification:
// mesh_uuid: each Mesh has one
// draw_uuid: each gameobject implementing Renderable has one
// shaders, textures, & framebuffers have their own ids as well but opengl takes care of that for us



pub struct GraphicsEngine {
    gl: gl46::GlFns,

    resolution: (u32, u32),
    
    pub world_shader_id: u32,
    pub camera: Camera,

    pub freecam_override_enabled: bool,
    freecam_transform: Transform,
    freecam_pitchyaw: Vec2,
    freecam_speed: i64,

    // draw everything to this framebuffer's texture, then render a quad to the screen with that texture so we can do post proc
    postproc_framebuffer_id: u32, 
    postproc_shader_id: u32,

    renderable_gameobjects: HashMap<usize, Rc<RefCell<dyn Renderable>>>,

    pools: HashMap<GLuint /*(shader program id)*/, HashMap<u32 /*(texture/texture array id)*/, Vec<MeshPool>>>,
    shaders: HashMap<u32, ShaderProgram>, // key is program id
    textures: HashMap<u32, Texture>, // key is gl texture id
    framebuffers: HashMap<u32, Framebuffer>, // key is gl framebuffer id

    // tells how to get to the drawing data for a particular object from its draw id
    object_drawing_data_locations: HashMap<usize, (u32, u32, usize, i32, i32)>, // tuple is (programid, textureid, index in vector, slot within meshpool, instance offset)

    cached_meshes_to_add: HashMap<usize, (u32, Vec<usize>)>, // key is [meshuuid or -1], value is (count, vec<keys of mesh_locations that need to get filled out>)
                            // dynamic meshes don't go in here and are added immediately as they cannot be instanced
                                // faster than sorting it when we add meshes
                            // when someone adds a mesh, it goes in here until the engine empties this cache, so it can use meshpool's count arguments to speed stuff up 
                            // hopefully cloning big meshes into this won't be slow, but if it's okay to do it once (to put mesh in vbo) its prob okay to do it twice

    //mesh_id_locations: HashMap<u32, u32>, // key is returned by add(), value is key for mesh_locations

    last_draw_uuid: usize,

    screen_quad_vbo: u32,
    screen_quad_vao: u32,
}

impl GraphicsEngine {
    pub fn new(gl_context: gl46::GlFns, resolution: (u32, u32)) -> Self {
        let mut ge = Self {
            gl: gl_context,

            resolution: resolution,

            world_shader_id: 0,
            camera: Camera::new(),

            freecam_override_enabled: false,
            freecam_pitchyaw: vec2(0.0, 0.0),
            freecam_speed: 0,
            freecam_transform: Transform::new(i64vec3(0, 0, -3000000)),

            postproc_framebuffer_id: 0,
            postproc_shader_id: 0,

            renderable_gameobjects: HashMap::new(),

            pools: HashMap::new(),
            shaders: HashMap::new(),
            textures: HashMap::new(),
            framebuffers: HashMap::new(),
            object_drawing_data_locations: HashMap::new(),

            cached_meshes_to_add: HashMap::new(),

            last_draw_uuid: 1,

            screen_quad_vbo: 0, 
            screen_quad_vao: 0,
        };
        
        ge.postproc_shader_id = ge.load_shader("shaders/postproc_vertex.glsl", "shaders/postproc_fragment.glsl", vec!["screenTexture"]);
        let postproc_framebuffer = Framebuffer::new(&ge.gl, resolution.0, resolution.1, true, false);
        ge.postproc_framebuffer_id = postproc_framebuffer.gl_framebuffer;
        ge.load_framebuffer(postproc_framebuffer);
        ge.world_shader_id = ge.load_shader("shaders/world_vertex.glsl", "shaders/world_fragment.glsl", vec!["textures", "shadowmap"]);
        ge.setup_screen_quad();

        // make opengl track errors for us
        unsafe {
            ge.gl.Enable(gl46::GL_DEBUG_OUTPUT);
            ge.gl.DebugMessageCallback(Some(opengl_debug_callback), std::ptr::null());
            ge.gl.ClearColor(0.5, 0.5, 0.8, 1.0);
        }

        return ge;
    }

    // verts for quad that covers screen
    const SCREEN_QUAD_VERTS : [f32; 24] = [-1.0, -1.0, 0.0, 0.0,
                                        1.0, -1.0, 1.0, 0.0,
                                       -1.0,  1.0, 0.0, 1.0,
                                        1.0, -1.0, 1.0, 0.0,
                                        1.0,  1.0, 1.0, 1.0,
                                       -1.0,  1.0, 0.0, 1.0];
    
     // Sets up vbo/vao for a quad that covers the screen, so we can render a texture to it for postproc
    pub fn setup_screen_quad(&mut self) {
        unsafe {
            self.gl.Enable(GL_DEPTH_TEST);

            self.gl.GenBuffers(1, &mut self.screen_quad_vbo as *const u32 as *mut u32);
            self.gl.GenVertexArrays(1, &mut self.screen_quad_vao as *const u32 as *mut u32);
            self.gl.BindBuffer(GL_ARRAY_BUFFER, self.screen_quad_vbo);
            self.gl.BindVertexArray(self.screen_quad_vao);

            self.gl.BufferData(GL_ARRAY_BUFFER, 24*4, &Self::SCREEN_QUAD_VERTS[0] as *const f32 as *const c_void, GL_STATIC_DRAW);
            
            self.gl.EnableVertexAttribArray(0);
            self.gl.VertexAttribPointer(0, 2, GL_FLOAT, false as u8, 16, std::ptr::null());
            self.gl.EnableVertexAttribArray(1);
            self.gl.VertexAttribPointer(1, 2, GL_FLOAT, false as u8, 16, 8 as *const c_void);
        }
    }

    pub fn update_resolution(&mut self, resolution: (u32, u32)) {
        self.resolution = resolution;
        self.framebuffers[&self.postproc_framebuffer_id].cleanup(&self.gl);
        let postproc_framebuffer = Framebuffer::new(&self.gl, resolution.0, resolution.1, true, false);
        self.postproc_framebuffer_id = postproc_framebuffer.gl_framebuffer;
        self.load_framebuffer(postproc_framebuffer);
    }

    // returns shader id
    pub fn load_shader(&mut self, vertex_path : &'static str, fragment_path : &'static str, texture_names: Vec<&str>) -> u32 {
        let shader_program = ShaderProgram::new(&self.gl, vertex_path, fragment_path, texture_names);
        let id = shader_program.program;
        self.shaders.insert(id, shader_program);
        return id; 
    } 

    // returns (texture id, texture size)
    pub fn load_texture_from_file(&mut self, path: &str, ttype: TextureType) -> (u32, (i32, i32)) {
        let texture = Texture::from_file(&self.gl, path, ttype);
        let id = texture.gl_texture;
        let size = texture.size;
        self.textures.insert(id, texture);
        return (id, size);
    }

    fn load_framebuffer(&mut self, framebuffer: Framebuffer) {
        self.framebuffers.insert(framebuffer.gl_framebuffer, framebuffer);
    }

    // does floating origin, updates instanced data buffers, sets camera matrices, all prerendering work
    pub fn update(&mut self, resolution: (u32, u32)) {
        if self.resolution != resolution {
            self.update_resolution(resolution);
        }
        self.update_freecam();
        self.add_cached_renderables();
        self.update_camera_matrices();

        // Update the data on the gpu for objects that have moved/changed color/etc
        let mut camera_pos = self.camera.transform.pos(); // for floating origin, the objects positions are offset by camera position on the cpu, so camera always at 0,0,0
        if self.freecam_override_enabled {
            camera_pos = self.freecam_transform.pos();
        }
        camera_pos *= -1;

        for (_, obj_refcell) in self.renderable_gameobjects.iter_mut() {
            let obj = obj_refcell.borrow();
            let draw_id = obj.get_draw_id();
            let loc = self.object_drawing_data_locations[&draw_id];
            self.pools.get(&loc.0).unwrap().get(&loc.1).unwrap().get(loc.2).unwrap().set_transform(loc.3, loc.4, &obj.transform().get_model(&camera_pos));
            if obj.get_color_changed() {
                self.pools.get(&loc.0).unwrap().get(&loc.1).unwrap().get(loc.2).unwrap().set_rgba(loc.3, loc.4, &obj.get_rgba());
            };

            if obj.get_texture_z_changed() {
                self.pools.get(&loc.0).unwrap().get(&loc.1).unwrap().get(loc.2).unwrap().set_texture_z(loc.3, loc.4, &obj.get_texture_z());
            };
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            self.gl.Clear(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT);
        }
        self.draw_to_framebuffer(&vec![self.world_shader_id], self.postproc_framebuffer_id, self.resolution);
        self.present_framebuffer(self.postproc_framebuffer_id, self.postproc_shader_id);
    }

    pub fn cleanup(&mut self) {
        for (_, texture) in self.textures.iter() {
            texture.cleanup(&self.gl);
        }

        for (_, shader) in self.shaders.iter() {
            shader.cleanup(&self.gl);
        }

        for (_, framebuffer) in self.framebuffers.iter() {
            framebuffer.cleanup(&self.gl);
        }

        for (_, hashmapthing) in self.pools.iter_mut() {
            for (_, vecofpools) in hashmapthing.iter_mut() {
                for pool in vecofpools.iter_mut() {
                    pool.cleanup(&self.gl);
                }  
            }   
        }
    }

    // Does not actually add the object to a meshpool (unless the mesh is dynamic) but adds them to a cache of meshes to add each frame for performance reasons
    // Sets obj draw_id.
    pub fn add_renderable(&mut self, obj_refcell: Rc<RefCell<dyn Renderable>>)  {
        let mut obj = obj_refcell.borrow_mut();

        let draw_id = self.last_draw_uuid;
        self.last_draw_uuid += 1;
        obj.set_draw_id(draw_id);

        let mesh_id = obj.get_mesh_id();
        
        drop(obj);

        self.renderable_gameobjects.insert(draw_id, obj_refcell); 

        let mut instance_offset = 0;
        let mesh = &LOADED_MESHES.lock().unwrap()[&mesh_id];

        if mesh.dynamic { // can't instance a dynamic mesh since its vertices could change at any time, so no point caching (TODO SEE mesh.rs)
            self.add_mesh_to_pool(mesh_id, 1, vec!(draw_id));
        }
        else if !self.cached_meshes_to_add.contains_key(&mesh_id) {
            self.cached_meshes_to_add.insert(mesh.uuid, (1, vec!(draw_id)));
        }
        else {
            instance_offset = self.cached_meshes_to_add[&mesh.uuid].0;
            self.cached_meshes_to_add.get_mut(&mesh.uuid).unwrap().0 += 1;
            self.cached_meshes_to_add.get_mut(&mesh.uuid).unwrap().1.push(draw_id);
        }

        self.object_drawing_data_locations.insert(draw_id, (0, 0, 0, 0, instance_offset as i32)); // this gets filled out in add_mesh_to_pool, except for the instance offset
    }

    fn add_cached_renderables(&mut self) {
        let data: Vec<_> = self.cached_meshes_to_add.drain().collect();
        for (mesh_id, (count, loc_keys)) in data {
            self.add_mesh_to_pool(mesh_id, count, loc_keys);
        }
    }

    fn add_mesh_to_pool(&mut self, mesh_id: usize, count: u32, location_keys: Vec<usize>) {
        let mesh = &LOADED_MESHES.lock().unwrap()[&mesh_id];

        // if our shader/texture has not been used yet, then create meshpool storage for them
        if !self.pools.contains_key(&mesh.shader_id) {
            self.pools.insert(mesh.shader_id, HashMap::new());
        }
        if !self.pools[&mesh.shader_id].contains_key(&mesh.texture_id) {
            self.pools.get_mut(&mesh.shader_id).unwrap().insert(mesh.texture_id, Vec::new());
        }

        //find smallest pool which will hold mesh
        let mut found_pool = false;   
        let vec: &mut Vec<MeshPool> = self.pools.get_mut(&mesh.shader_id).unwrap().get_mut(&mesh.texture_id).unwrap();
        //println!("There are already {:?} pools here", vec.len());
        let mut i = 0;
        for pool in vec.iter_mut() {
            if pool.mesh_vertex_nbytes as usize >= mesh.vertices.len() * 4 { // if the pool is big enough
                if (pool.mesh_vertex_nbytes as usize) <= mesh.vertices.len() * 16 { // but also if the pool is more than 4 times as big then it needs to be, skip it
                    found_pool = true;
                    let slot = pool.add_mesh(&self.gl, mesh.uuid as i32, &mesh.vertices, &mesh.indices, count, mesh.dynamic);
                    for k in location_keys.iter() {
                        let loc = self.object_drawing_data_locations.get_mut(k).unwrap();
                        loc.0 = mesh.shader_id;
                        loc.1 = mesh.texture_id;
                        loc.2 = i;
                        loc.3 = slot.0;
                        loc.4 += slot.1;
                        //println!("Added stuff to location! {:?}, slot.1 was {}", loc, slot.1);
                    }
                    
                    break;
                }
            }
            else {
                //println!("skipping ")
            }
            i += 1;
        }

        // if no pool was found, create one
        if !found_pool {
            //println!("No available meshpool for mesh with {} vertices; creating new pool", mesh.vertices.len()/N_FLOATS_PER_VERTEX);
            let mut pool_size: usize = N_FLOATS_PER_VERTEX * 4;
            while pool_size < mesh.vertices.len() * 4 {
                pool_size*=2;
            }

            // TODO: instance capacity should REALLY depend (for cubes, instance mcuh > meshes, while for big meshes its about equal)
            let capacity = f32::ceil(TARGET_MESHPOOL_BASE_SIZE as f32/pool_size as f32) as isize;
            // println!("Capacity {}", capacity);
            let mut new_pool = MeshPool::new(&self.gl, capacity, capacity, pool_size as isize, pool_size as isize); // TODO: more intelligently set base capacity OR more intelliently resize pool
            let slot = new_pool.add_mesh(&self.gl, mesh.uuid as i32, &mesh.vertices, &mesh.indices, count, mesh.dynamic);
            self.pools.get_mut(&mesh.shader_id).unwrap().get_mut(&mesh.texture_id).unwrap().push(new_pool);
            for k in location_keys {
                let loc = self.object_drawing_data_locations.get_mut(&k).unwrap();
                loc.0 = mesh.shader_id;
                loc.1 = mesh.texture_id;
                loc.2 = self.pools[&mesh.shader_id][&mesh.texture_id].len() - 1;
                loc.3 = slot.0;
                loc.4 += slot.1;
                //println!("Added stuff to location! {:?}, slot.1 was {}", loc, slot.1);
            }
            
        }

    }

    // Given the id of a framebuffer that covers the WHOLE screen, it will draw a quad with that framebuffer's color texture over the screen using the given shader 
    fn present_framebuffer(&self, buffer_id: u32, shader_id: u32) {
        let buffer = &self.framebuffers[&buffer_id];

        unsafe {
            self.shaders[&shader_id].r#use(&self.gl);
            buffer.color.as_ref().unwrap().r#use(&self.gl, 0);
            self.gl.BindVertexArray(self.screen_quad_vao);
            self.gl.DrawArrays(GL_TRIANGLES, 0, 6);
        }
    }

    // draws everything associated with the given shader programs into the given framebuffer.
    // shaders should probably actually be compatible with the given framebuffer
    fn draw_to_framebuffer(&mut self, shader_ids: &Vec<GLuint>, buffer_id: u32, resolution: (u32, u32)) { 
        unsafe {
            self.gl.CullFace(GL_BACK);
        }
        let buffer = &self.framebuffers[&buffer_id];
        buffer.begin_render(&self.gl);
        for id in shader_ids {
            self.shaders[id].r#use(&self.gl);
            if self.shaders[id].shadowmap_texture_index != -1 {
                //println!("deploying shadowmap at loc {}", self.shaders.get_mut(id).unwrap().shadowmap_texture_index);
                //self.shaders.get_mut(id).unwrap().matrix4x4(&"modelToLightSpace".to_string(), &self.spotlights[0].get_model_to_light_space(), false); // TODO: USE SSBO HERE SO WE CAN HAVE MORE THAN ONE LIGHT LOL
                //self.shadowmap.depth.as_ref().unwrap().r#use(self.shaders.get_mut(id).unwrap().shadowmap_texture_index as u32);
            }
            let map = &self.pools[&id];
            for (texture_id, vec) in map {
                if *texture_id != 0 {
                    self.textures[texture_id].r#use(&self.gl, 0); 
                }
                else { unsafe {
                    //println!("skipping texture binding here lol");
                    self.gl.BindTexture(GL_TEXTURE_2D, 0);
                    self.gl.BindTexture(GL_TEXTURE_2D_ARRAY, 0);
                }}
                for pool in vec {
                    pool.draw(&self.gl); 
                }
            }
        }
        buffer.finish_render(&self.gl, resolution.0, resolution.1)
    }

    fn update_camera_matrices(&mut self) {
        // Pass camera matrices to shader programs that want them
        {
            self.camera.perspective(self.resolution.0 as f32/self.resolution.1 as f32, 70.0, 0.1, 4096.0);
        }
        let mut cam_mat = self.camera.transform.get_model(&self.camera.transform.pos());

        if self.freecam_override_enabled {
            self.update_freecam();
            cam_mat = self.freecam_transform.get_model(&self.freecam_transform.pos());
        }

        for program in self.shaders.iter_mut() {
            if program.1.auto_cam {
                //println!("autocamming {}", program.0);
                program.1.matrix4x4(&self.gl, &String::from("camera"), &cam_mat, false);
            }
            if program.1.auto_proj && self.camera.proj_changed {
                
                program.1.matrix4x4(&self.gl, &String::from("proj"), self.camera.get_proj(), false);
            }
        }
    }

    // a toggleable freecam exists for debugging purposes, this function controls its pos/rotation
    fn update_freecam(&mut self) {
        if self.freecam_override_enabled {

            let right = self.freecam_transform.get_right_vector() * (INPUT.is_pressed(Key::D) as i32 as f32 - INPUT.is_pressed(Key::A) as i32 as f32);
            let up = self.freecam_transform.get_up_vector() * (INPUT.is_pressed(Key::E) as i32 as f32 - INPUT.is_pressed(Key::Q) as i32 as f32);
            let forward = self.freecam_transform.get_look_vector() * (INPUT.is_pressed(Key::S) as i32 as f32 - INPUT.is_pressed(Key::W) as i32 as f32);
            self.freecam_transform.setpos(self.freecam_transform.pos() - (i64vec3_from_vec3(&(&(right/100.0 + forward/100.0 + up/100.0))) * self.freecam_speed));
            self.freecam_pitchyaw += INPUT.mouse_delta().yx() * 0.005;
            self.freecam_pitchyaw.x = self.freecam_pitchyaw.x.clamp(-89.0f32.to_radians(), 89.0f32.to_radians());
            self.freecam_transform.setrotyxz(vec3(self.freecam_pitchyaw.x, self.freecam_pitchyaw.y, 0.0));
            
            if INPUT.is_pressed(Key::A) || INPUT.is_pressed(Key::D) || INPUT.is_pressed(Key::S) || INPUT.is_pressed(Key::W) || INPUT.is_pressed(Key::Q) || INPUT.is_pressed(Key::E) {
                self.freecam_speed += 1;
                println!("OMG ");
            }
            else {
                self.freecam_speed = 0;
            }
            //println!("look vector: {:?}",  self.freecam_transform.get_look_vector());
            //println!("rot:{:?}", self.freecam_transform.rot());
        }
    }

}