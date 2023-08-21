use std::collections::VecDeque;

const TARGET_MESHPOOL_BASE_SIZE: isize = (2 as isize).pow(24); // ~16MB

// Contains a bunch of meshes of the same size for rapid drawing with special opengl stuff. internally used by MeshMaster
struct MeshPool {
    mesh_vertex_nbytes: isize,
    mesh_index_nbytes: isize,
    base_mesh_capacity: isize,
    mesh_capacity: isize,
    base_instanced_data_capacity: isize,
    instance_nbytes: isize,
    instanced_data_capacity: isize,

    draw_commands: Vec<IndirectDrawCommand>,
    drawcount: i32,
    // model_matrices: Vec<glm::Mat4>, // todo: potentially convert to vector of floats sp we can instance texture array index / color vertex attributes

    vbo : GLuint,  // hold vertices
    mvbo : GLuint, // secondary vbo with an instanced arrays of model matrices, color, texturez in that order
    ibo : GLuint,  // holds indices
    vao : GLuint,  // tells opengl how vertices are formatted
    indbo: GLuint, // stores rendering commands
    //ssbo: GLuint, // (shader storage buffer object) stores one model matrix per mesh

    pool_vertices : *mut ::libc::c_void,
    pool_instanced_data : *mut ::libc::c_void,
    pool_indices : *mut ::libc::c_void,
    pool_commands: *mut ::libc::c_void,

    available_slots : VecDeque<i32>, // represent portions of memory in the vbo where there is room for a mesh
                                     // supposed to let us not syncronize stuff,
    // available_instanced_data_slots : VecDeque<i32>, // same but for mvbo; ok deque is slow, i'm just going to manually sync mvbo instead of using deque
    available_instanced_data_slots_vec : Vec<bool>, // indices are instance slots, values are true if they are free
                                                    // used for quickly finding continuous portions of memory in mvbo
    n_available_instance_slots: usize,

    vertex_slots_to_instanced_slots : HashMap<i32, i32>, // should probably be a vector but im but lazy
    slot_contents: HashMap<i32, i32>, // key is slot, value is mesh uuid; only contains static meshes
}

impl MeshPool {
    fn new(gl: &gl46::GlFns, n_meshes: isize, n_instances: isize, sizeof_verts: isize, sizeof_indices: isize) -> Self { // should probably make indbo/ssbo here instead of in expand()
        // println!("capacity {}, bytes per is {}", n_meshes, sizeof_verts);
        
        let mut new_pool = Self {
            mesh_vertex_nbytes: sizeof_verts,
            mesh_index_nbytes: sizeof_indices, 
            base_mesh_capacity: n_meshes,
            mesh_capacity: 0,
            base_instanced_data_capacity: n_instances,
            instance_nbytes: 64 + 16 + 4, // size of mat 4x4 + size of vec4 color/transparency + sizeof float for texture z
            instanced_data_capacity: 0,

            draw_commands: Vec::new(),
            drawcount: 0,
            
            // model_matrices: Vec::new(),

            vbo: 0,
            mvbo: 0,
            ibo: 0,
            vao: 0,
            indbo: 0,
            //ssbo: 0,

            pool_vertices: std::ptr::null_mut(),
            pool_instanced_data: std::ptr::null_mut(),
            pool_indices: std::ptr::null_mut(),
            pool_commands: std::ptr::null_mut(),

            available_slots: VecDeque::new(), 
            // available_instanced_data_slots: VecDeque::new(),
            available_instanced_data_slots_vec: Vec::new(),
            n_available_instance_slots: 0,

            vertex_slots_to_instanced_slots: HashMap::new(),
            slot_contents: HashMap::new()
        };
        new_pool.expand_vbo(gl);
        MeshPool::expand_mvbo(gl, 1, &mut new_pool.instanced_data_capacity, new_pool.base_instanced_data_capacity, &mut new_pool.n_available_instance_slots, /*&mut new_pool.available_instanced_data_slots,*/ new_pool.mesh_capacity, &mut new_pool.mvbo, &mut new_pool.pool_instanced_data, new_pool.instance_nbytes, &mut new_pool.available_instanced_data_slots_vec);
        return new_pool;
    }

    fn cleanup(&mut self, gl: &gl46::GlFns) {
        self.del_vertex_buffers(gl);
        MeshPool::del_instanced_buffers(gl, &mut self.mvbo);
    }

    // mvbo size should be indepedant of vbo/ibo size because instancing can cause many models w/ different matrices to be drawn with same vertices
    // this one needs a size argument becuase if we add like 20000 instances and the base is 5000, expanding once wouldn't be enough and it would segfault
    // because of the borrow checker this function doesn't use self and instead requires a bunch of self's members to be passed to it
    pub fn expand_mvbo(gl: &gl46::GlFns, size: isize, instanced_data_capacity: &mut isize, base_instanced_data_capacity: isize, n_available_instances: &mut usize, /*available_instanced_data_slots: &mut VecDeque<i32>,*/ mesh_capacity: isize, mvbo: &mut GLuint, pool_instanced_data: &mut *mut ::libc::c_void, instance_nbytes: isize, available_instanced_data_slots_vec: &mut Vec<bool>) {
        // println!("Expanding mvbo by {} * {}", size, base_instanced_data_capacity);

        let old_cap = *instanced_data_capacity;

        // for i in 0..base_instanced_data_capacity*size {
        //     available_instanced_data_slots.push_back((i + *instanced_data_capacity).try_into().unwrap());
        // }

        *instanced_data_capacity += base_instanced_data_capacity*size;
        *n_available_instances += base_instanced_data_capacity as usize*size as usize;
        available_instanced_data_slots_vec.resize(*instanced_data_capacity as usize, true);
        let flags = GL_MAP_PERSISTENT_BIT|/*GL_MAP_COHERENT_BIT|*/GL_MAP_WRITE_BIT;
        // self.model_matrices.resize_with(self.instanced_data_capacity as usize, ||{glm::identity()});

        unsafe {


            let mut newmvbo: GLuint = 0; 
            let mvboptr: *mut GLuint = &mut newmvbo;
            gl.GenBuffers(1, mvboptr);

            // secondary array buffer (mvbo) contains model matrices
            gl.BindBuffer(GL_ARRAY_BUFFER, newmvbo);
            gl.BufferStorage(GL_ARRAY_BUFFER, instance_nbytes * *instanced_data_capacity, std::ptr::null(), flags);
            let new_instanced_data = gl.MapBufferRange(GL_ARRAY_BUFFER, 0, *instanced_data_capacity * instance_nbytes, flags);

            gl.EnableVertexAttribArray(1); // color rgba 
            gl.VertexAttribPointer(1, 4, GL_FLOAT, 0, instance_nbytes as i32, 64 as *mut _);
            gl.VertexAttribDivisor(1, 1);

            gl.EnableVertexAttribArray(8); // texture z
            gl.VertexAttribPointer(8, 1, GL_FLOAT, 0, instance_nbytes as i32, (64 + 16) as *mut _);
            gl.VertexAttribDivisor(8, 1);

            // each vertex attribute has to be no more than a vec4, so for a whole matrix we do 4 attributes
            gl.EnableVertexAttribArray(4); // model matrix
            gl.EnableVertexAttribArray(5); 
            gl.EnableVertexAttribArray(6); 
            gl.EnableVertexAttribArray(7);

            gl.VertexAttribPointer(4, 4, GL_FLOAT, 0, instance_nbytes as i32, 0 as *mut _);
            gl.VertexAttribPointer(5, 4, GL_FLOAT, 0, instance_nbytes as i32, 16 as *mut _);
            gl.VertexAttribPointer(6, 4, GL_FLOAT, 0, instance_nbytes as i32, 32 as *mut _);
            gl.VertexAttribPointer(7, 4, GL_FLOAT, 0, instance_nbytes as i32, 48 as *mut _);

            gl.VertexAttribDivisor(4, 1); // this divisor thing makes these attributes per instance/mesh instead of per vertex
            gl.VertexAttribDivisor(5, 1);
            gl.VertexAttribDivisor(6, 1);
            gl.VertexAttribDivisor(7, 1); 

            // copy data from old mvbo if needed
            if *mvbo != 0 {
                libc::memcpy(new_instanced_data, *pool_instanced_data, (old_cap * 64) as usize);

                // Delete old buffers
                MeshPool::del_instanced_buffers(gl, mvbo);
            }

            *mvbo = newmvbo;
            // println!("Old pointer to instanced data was {:?}", *pool_instanced_data);
            *pool_instanced_data = new_instanced_data;
            // println!("New pointer is {:?}", *pool_instanced_data);
        }
    }

    fn set_transform(&self, slot: i32, instance: i32, matrix: &glm::Mat4) {
        unsafe {
            let instance_slot = (self.vertex_slots_to_instanced_slots[&slot] + instance) as isize;
            assert!(instance_slot < self.instanced_data_capacity, "Error: We were told to modify the transform for slot {}, but there are only {} slots.", instance_slot, self.instanced_data_capacity);
            // if instance != 0 {
            // println!("For instance {} of slot {}, it's {}", instance, slot, instance_slot); 17164
            // }
            //println!("Destination = {:?}", self.pool_instanced_data.offset(instance_slot * self.instance_nbytes));
            // println!("offset={}", self.instance_nbytes * instance_slot);
            libc::memcpy(self.pool_instanced_data.offset(instance_slot * self.instance_nbytes), &matrix[0] as *const f32 as *const c_void, 64);
            //  println!("Transformed slot {} with matrix {:?}", slot, matrix);   
            //println!("Binding buffer {}", self.ssbo);         
            //gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, self.ssbo);
            //gl.BufferSubData(GL_SHADER_STORAGE_BUFFER, slot as isize * 64 , 64, &matrix[0] as *const f32 as *const c_void);
        }
    }

    fn set_rgba(&self, slot: i32, instance: i32, color: &glm::Vec4) {
        unsafe {
            let instance_slot = (self.vertex_slots_to_instanced_slots[&slot] + instance) as isize;
            assert!(instance_slot < self.instanced_data_capacity);
            libc::memcpy(self.pool_instanced_data.offset(instance_slot * self.instance_nbytes + 64), &color[0] as *const f32 as *const c_void, 16);
        }
    }

    fn set_texture_z(&self, slot: i32, instance: i32, tex: &f32) { 
        unsafe {
            let instance_slot = (self.vertex_slots_to_instanced_slots[&slot] + instance) as isize;
            assert!(instance_slot < self.instanced_data_capacity);
            libc::memcpy(self.pool_instanced_data.offset(instance_slot * self.instance_nbytes + 64 + 16), tex as *const f32 as *const c_void, 4);
        }
    }

    // fn resize_ssbo(&self) {
    //     unsafe {
    //         gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, self.ssbo);
    //         //println!("Buffering ssbo data! {:?}", self.model_matrices);
    //         gl.BufferData(GL_SHADER_STORAGE_BUFFER, self.mesh_capacity * 64, self.model_matrices.as_ptr() as *const c_void, GL_DYNAMIC_DRAW);
    //         gl.BindBufferBase(GL_SHADER_STORAGE_BUFFER, 3, self.ssbo);
    //         check_graphics_errors();
    //     }
    // }

    fn del_vertex_buffers(&mut self, gl: &gl46::GlFns) {
        unsafe {
            let vaoptr: *mut GLuint = &mut self.vao;
            gl.DeleteVertexArrays(1, vaoptr);
            let vboptr: *mut GLuint = &mut self.vbo;
            gl.DeleteBuffers(1, vboptr);    
            let indboptr: *mut GLuint = &mut self.indbo;
            gl.DeleteBuffers(1, indboptr);   
            let iboptr: *mut GLuint = &mut self.ibo;
            gl.DeleteBuffers(1, iboptr);
            
        }
    }

    // *buffer
    // also not a method because of the borrow checker
    fn del_instanced_buffers(gl: &gl46::GlFns, mvbo: &mut GLuint) {
        unsafe {
            let mvboptr: *mut GLuint = mvbo;
            gl.DeleteBuffers(1, mvboptr);
        }
    }

    // called when mesh pool runs out of space
    // TODO: should check opengl version and not use coherent_bit/persistent_bit and use glBufferSubData if persistent mapping is unsupported
    fn expand_vbo(&mut self, gl: &gl46::GlFns) {
        // println!("Expanding mesh pool.");
        let old_cap = self.mesh_capacity;
        for i in 0..self.base_mesh_capacity {
            self.available_slots.push_back((i + self.mesh_capacity).try_into().unwrap());
        }

        self.mesh_capacity += self.base_mesh_capacity;
        self.draw_commands.resize_with(self.mesh_capacity as usize, IndirectDrawCommand::new);
        

        let flags = GL_MAP_PERSISTENT_BIT|GL_MAP_COHERENT_BIT|GL_MAP_WRITE_BIT;
        
        unsafe {

            //self.resize_ssbo();
            //self.resize_indbo();
            // println!("resized");

            // Because we said glBufferStorage, we can't resize our previous buffers and must replace them
            //todo: should really gen all 4 buffers with one call lol
            let mut newvao: GLuint = 0;
            let vaoptr: *mut GLuint = &mut newvao;
            gl.GenVertexArrays(1, vaoptr); //idk if deleting the vao is neccesary

            let mut newvbo: GLuint = 0;
            let vboptr: *mut GLuint = &mut newvbo;
            gl.GenBuffers(1, vboptr);   
            let mut newibo: GLuint = 0; 
            let iboptr: *mut GLuint = &mut newibo;
            gl.GenBuffers(1, iboptr);
            let mut newindbo: GLuint = 0;
            let indboptr: *mut GLuint = &mut newindbo;
            gl.GenBuffers(1, indboptr);
            // gl.GenBuffers(3, [vboptr, iboptr, mvboptr].as_ptr() as *mut u32);
            
            gl.BindVertexArray(newvao);

            // array buffer (vbo) contains mesh vertices
            gl.BindBuffer(GL_ARRAY_BUFFER, newvbo);
            gl.BufferStorage(GL_ARRAY_BUFFER, self.mesh_vertex_nbytes * self.mesh_capacity, std::ptr::null(), flags);
            let new_pool_vertices = gl.MapBufferRange(GL_ARRAY_BUFFER, 0, self.mesh_capacity * self.mesh_vertex_nbytes, flags); //mapbufferrange gives a pointer that we put the vertices/indices in

            // element array buffer (ibo) contains mesh indices
            gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, newibo);
            gl.BufferStorage(GL_ELEMENT_ARRAY_BUFFER, self.mesh_index_nbytes * self.mesh_capacity, std::ptr::null(), flags);
            let new_pool_indices = gl.MapBufferRange(GL_ELEMENT_ARRAY_BUFFER, 0, self.mesh_capacity * self.mesh_index_nbytes, flags);
            
            // draw indirect buffer (indbo) contains drawing commands
            gl.BindBuffer(GL_DRAW_INDIRECT_BUFFER, newindbo);
            gl.BufferStorage(GL_DRAW_INDIRECT_BUFFER, 20 * self.mesh_capacity, std::ptr::null(), flags);
            let new_pool_commands = gl.MapBufferRange(GL_DRAW_INDIRECT_BUFFER, 0, self.mesh_capacity * 20, flags);
            
            // make sure to associate other buffers with vao too? do i care about that?
            //gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, self.ssbo);
            //gl.BindBuffer(GL_DRAW_INDIRECT_BUFFER, self.indbo);

            // Tell opengl how the vertices inside the vbo are formatted
            gl.EnableVertexAttribArray(0);
            gl.EnableVertexAttribArray(2); // attrib 1 is taken by color lol
            gl.EnableVertexAttribArray(3);

            gl.VertexAttribFormat(0, 3, GL_FLOAT, false as u8, 0); // First 3 floats of each vertex is the position
            gl.VertexAttribFormat(2, 3, GL_FLOAT, false as u8, 0); // Then normals
            gl.VertexAttribFormat(3, 3, GL_FLOAT, false as u8, 0); // Last 2 are texture

            gl.VertexBindingDivisor(0, 0);
            gl.VertexBindingDivisor(2, 0);
            gl.VertexBindingDivisor(3, 0);

            gl.VertexAttribBinding(0, 0);
            gl.VertexAttribBinding(2, 2);
            gl.VertexAttribBinding(3, 3);

            gl.BindVertexBuffer(0, newvbo, 0, N_FLOATS_PER_VERTEX as i32 * 4);
            gl.BindVertexBuffer(2, newvbo, 12, N_FLOATS_PER_VERTEX as i32 * 4);
            gl.BindVertexBuffer(3, newvbo, 24, N_FLOATS_PER_VERTEX as i32 * 4);

            // Move contents of the old vbo/ibo into the new one
            if self.vao != 0 || self.vbo != 0 || self.ibo != 0 {
                // println!("Copying old data!");
                libc::memcpy(new_pool_vertices, self.pool_vertices, (old_cap * self.mesh_vertex_nbytes) as usize);
                libc::memcpy(new_pool_indices, self.pool_indices, (old_cap * self.mesh_index_nbytes) as usize);
                libc::memcpy(new_pool_commands, self.pool_commands, (old_cap * 20) as usize);
                
                // Delete old buffers
                self.del_vertex_buffers(gl);
            }

            self.vao = newvao;

            self.vbo = newvbo;
            self.ibo = newibo;         
            self.indbo = newindbo;

            self.pool_vertices = new_pool_vertices;
            self.pool_indices = new_pool_indices;
            self.pool_commands = new_pool_commands;

            //libc::memmove(dest, src, n)
            //dealloc(ptr, layout);

            // println!("Finished expanding mesh pool.");
            }
    }

    // TODO: Should indbo use persistently mapped buffers? (and more importantly ssbo)
    // idk how neccesary this function is
    // fn resize_indbo(&mut self) {
    //     unsafe {
    //         // let mut commands: Vec<IndirectDrawCommand> = Vec::new();
    //         // for i in 0..self.draw_commands.len() {
    //         //     if self.draw_commands[i].indice_count != 0 {
    //         //         commands.push(self.draw_commands[i].clone());
    //         //     }     
    //         // }
    //         //println!("Updating INDBO with contents: {:?}", commands);

    //         //self.draw_count = commands.len() as i32;

    //         //println!("Drawcount = {}", self.draw_count);
    //         //println!("Size = {}", self.draw_count * size_of::<IndirectDrawCommand>() as i32);
    //         gl.BindBuffer(GL_DRAW_INDIRECT_BUFFER, self.indbo);
    //         gl.BufferData(GL_DRAW_INDIRECT_BUFFER, (size_of::<IndirectDrawCommand>() as isize * self.mesh_capacity) as isize, self.draw_commands.as_ptr() as *const c_void, GL_DYNAMIC_DRAW);
    //     } 
    // }

    fn update_indbo(&mut self, slot: i32) {
        unsafe {
            // println!("INDBO: Buffering 20 bytes at offset {}, including a count of {}", slot * 20, &self.draw_commands[slot as usize].indice_count);
            //gl.BufferSubData(GL_DRAW_INDIRECT_BUFFER, (slot * 20) as isize,size_of::<IndirectDrawCommand>() as isize, self.draw_commands.as_ptr().offset(slot as isize).cast());
            libc::memcpy(self.pool_commands.offset(slot as isize * 20), self.draw_commands.as_ptr().offset(slot as isize).cast(), 20);
        }
    }

    // when adding static meshes, this function will try to merge them with identical static meshes 
    // returns false if fail
    fn try_extend_instance(&mut self, gl: &gl46::GlFns, count: u32, mesh_uuid: i32) -> (bool, i32, i32) {
        // let mut indices_to_pop: Vec<isize> = Vec::with_capacity(count as usize);
        let mut indices_to_falsify: Vec<usize> = Vec::with_capacity(count as usize);

        'outer: for (slot, uuid) in self.slot_contents.iter() {
            if mesh_uuid == *uuid { // find slot with identical mesh
                // see if there's enough room for the additional instance matrices
                let start = self.vertex_slots_to_instanced_slots[slot] + self.draw_commands[*slot as usize].instance_count as i32;
                let gap = (start + 1 + count as i32) - self.n_available_instance_slots as i32;
                // println!("Gap = {} for adding count {} to pool with space {} when starting at {}", gap, count, self.n_available_instance_slots, start);
                if gap > 0 { // if the mvbo is tiny expand it, this technically will run when it doesn't need to but it would just run in find_instance_slot so its ok
                    // println!("We gotta expand!");
                    MeshPool::expand_mvbo(gl, (gap as f32 / self.base_instanced_data_capacity as f32).ceil() as isize, &mut self.instanced_data_capacity, self.base_instanced_data_capacity, &mut self.n_available_instance_slots,  /*&mut self.available_instanced_data_slots,*/ self.mesh_capacity, &mut self.mvbo, &mut self.pool_instanced_data, self.instance_nbytes, &mut self.available_instanced_data_slots_vec);
                }
                //println!("expansion successful");
                'inner: for i in start+1..(start+1+count as i32) {
                    //println!("stepping {}", i);
                    let free = self.available_instanced_data_slots_vec[i as usize]; 
                    if free {
                        // indices_to_pop.push(slot_index);
                        indices_to_falsify.push(i as usize);
                        // print!("b");
                    }
                    else {
                        // println!("a");
                        // indices_to_pop.clear();
                        indices_to_falsify.clear();
                        // println!("fail");
                        continue 'outer;
                    }
                }
                // println!("iteratino complete");
                // if it got through the for loop it works!

                // mark slots as taken
                //remove biggest indices first cuz otherwise indices would change
                // indices_to_pop.sort_unstable_by(|a, b| b.cmp(a));
                // for slot in indices_to_pop {
                //     //println!("popping {}", self.available_instanced_data_slots[slot as usize]);
                //     self.available_instanced_data_slots.remove(slot as usize);
                    
                // }

                self.n_available_instance_slots -= indices_to_falsify.len();
                for slot in indices_to_falsify {
                    self.available_instanced_data_slots_vec[slot] = false;
                }
                

                self.draw_commands[*slot as usize].instance_count += count;
                // println!("returning success on extension");
                return (true, *slot, start);
            }
        }
        return (false, -1, -1);
    }

    // finds first location in the mvbo that will accomodate count instances, expanding the mvbo if needed
    // removes items from available_instance_data_slots for you
    // this is an extremely dumb, unoptimized way of doing that but oh well
    // fortunately the overhead is only there when count > 1 so if ur not instancing its fine
    fn find_instance_slot(&mut self, gl: &gl46::GlFns, count: u32) -> i32 {
        //let tail_length = count - 1;
        //let mut index = 0;
        // let mut indices_to_pop: Vec<isize> = Vec::with_capacity(count as usize);
        let mut indices_to_falsify: Vec<usize> = Vec::with_capacity(count as usize);
        let mut found = -1;
        let mut slot = -1;
        let mut gap = 0;
        'outer: for is_free in &self.available_instanced_data_slots_vec {
            slot+=1;
            if *is_free == true {
                if count == 1 {
                    found = slot;
                    // indices_to_pop.push(index);
                    indices_to_falsify.push(slot as usize);
                    break 'outer;
                }
                else {
                    'inner: for index2 in 0..count {
                        if slot as usize + index2 as usize >= self.available_instanced_data_slots_vec.len() {
                            gap = (slot + index2 as i32 + 1) - self.available_instanced_data_slots_vec.len() as i32;
                            break 'outer;
                        }
                        
                        if !self.available_instanced_data_slots_vec[slot as usize + index2 as usize] {
                            slot += 1;
                            // indices_to_pop.clear();
                            indices_to_falsify.clear();
                            continue 'outer;
                        }
                        else {
                            // indices_to_pop.push(index_deque(&self.available_instanced_data_slots, slot + index2 as i32));
                            indices_to_falsify.push(slot as usize + index2 as usize);
                            slot += 1;
                        }
                    }
                    found = slot;
                    break 'outer;
                    // index += 1;
                }
            }
        }

        if gap < 0 {
            // println!("FIS: EXPANDING BY {}", gap);
            MeshPool::expand_mvbo(gl, (gap as f32 / self.base_instanced_data_capacity as f32).ceil() as isize, &mut self.instanced_data_capacity, self.base_instanced_data_capacity, &mut self.n_available_instance_slots,  /*&mut self.available_instanced_data_slots,*/ self.mesh_capacity, &mut self.mvbo, &mut self.pool_instanced_data, self.instance_nbytes, &mut self.available_instanced_data_slots_vec);
            return self.find_instance_slot(gl, count);
        }

        // mark slots as taken
        //remove biggest indices first cuz otherwise indices would change
        // indices_to_pop.sort_unstable_by(|a, b| b.cmp(a));
        // for slot in indices_to_pop {
        //     //println!("popping {}", self.available_instanced_data_slots[slot as usize]);
        //     self.available_instanced_data_slots.remove(slot as usize);
            
        // }

        self.n_available_instance_slots -= indices_to_falsify.len();
        for slot in indices_to_falsify {
            self.available_instanced_data_slots_vec[slot] = false;
        }
        

        if found == -1 {
            MeshPool::expand_mvbo(gl, (count as f32 / self.base_instanced_data_capacity as f32).ceil() as isize, &mut self.instanced_data_capacity, self.base_instanced_data_capacity, &mut self.n_available_instance_slots, /*&mut self.available_instanced_data_slots,*/ self.mesh_capacity, &mut self.mvbo, &mut self.pool_instanced_data, self.instance_nbytes, &mut self.available_instanced_data_slots_vec);
            // println!("Having to expand the mvbo because we found nada.");
            return self.find_instance_slot(gl, count);
        }
        else {
            return found;
        }
    }

    // Adds count identical meshes to pool
    // if the mesh is not dynamic (meaning its vertices will never be modified), then it might be instanced for optimization
    // returns a tuple of (slot, instance) although instance will always be 0 unless it was instanced 
        // if count > 0, instance will be for the first one
    fn add_mesh(&mut self, gl: &gl46::GlFns, mesh_uuid: i32, vertices:&Vec<GLfloat>, indices:&Vec<GLuint>, count: u32, dynamic: bool) -> (i32, i32) {
        let now = std::time::Instant::now();
        let result : (bool, i32, i32) = self.try_extend_instance(gl, count, mesh_uuid);
        // println!("Attempting to extend the instance took {:?} seconds.", now.elapsed());
        if dynamic || !result.0 {
            

            self.drawcount += 1;
            // println!("Adding mesh, drawcount now {}", self.drawcount);
            if self.available_slots.len() == 0 {
                self.expand_vbo(gl);
            }
            let vertex_slot = self.available_slots.pop_front().unwrap();
            let instance_slot = self.find_instance_slot(gl, count); //println!("Found instance slot {} for {} instances.", instance_slot, count);

            self.vertex_slots_to_instanced_slots.insert(vertex_slot, instance_slot);
            // println!("Vertex slot {} corresponds to instance slot {}", vertex_slot, instance_slot);
            
            self.update_mesh(vertex_slot, vertices, indices);
            self.draw_commands[vertex_slot as usize].instance_count = count;
            self.draw_commands[vertex_slot as usize].base_instance = instance_slot as u32;
            // println!("Draw command at slot {} is {:?}", vertex_slot, self.draw_commands[vertex_slot as usize]);
            
            if (!dynamic) {
                self.slot_contents.insert(vertex_slot, mesh_uuid);
            }
            return (vertex_slot, 0);
        }
        else {
            // println!("Successfully extended previously existing instanced mesh by count {}.", count);
            // println!("\tDraw command at slot {} is {:?}", result.1, self.draw_commands[result.1 as usize]);
            return (result.1, result.2);
        }
    }

    fn update_mesh(&mut self, slot:i32, vertices:&Vec<GLfloat>, indices:&Vec<GLuint>) {
        // println!("Updating slot {}", slot);
        unsafe {
            let v_destination = self.pool_vertices.offset(slot as isize * self.mesh_vertex_nbytes as isize);
            libc::memcpy(v_destination, vertices.as_ptr().cast(), vertices.len() * core::mem::size_of::<GLfloat>());
            let i_destination = self.pool_indices.offset(slot as isize * self.mesh_index_nbytes as isize);
            //println!("Offset by {}", slot as isize * self.mesh_index_nbytes as isize);
            libc::memcpy(i_destination, indices.as_ptr().cast(), indices.len() * core::mem::size_of::<GLuint>());
            // if slot == 1 {
            //     println!("Data from ptr: {:?}", Vec::from_raw_parts(i_destination as *mut u32, indices.len() as usize, indices.len() as usize))
            //     // let ptr = self.pool_vertices as *mut GLfloat {

            //     // }
            // }

            
        }

        self.draw_commands[slot as usize].indice_count = indices.len() as u32;
        self.draw_commands[slot as usize].start = (slot * self.mesh_index_nbytes as i32) as u32;
        self.draw_commands[slot as usize].base_vertex = slot * (self.mesh_vertex_nbytes/(core::mem::size_of::<GLfloat>() * N_FLOATS_PER_VERTEX) as isize) as i32; 
        
        self.update_indbo(slot);
    }

    fn remove_mesh(&mut self, slot:i32) {
        self.drawcount -= 1;
        if self.slot_contents.contains_key(&slot) {
            self.slot_contents.remove(&slot);
        }
        self.draw_commands[slot as usize].indice_count = 0; // if n_indices == 0, we treat that as no command
        self.available_slots.push_back(slot);
        self.update_indbo(slot);
    }
    
    fn draw(&self, gl: &gl46::GlFns) { 
        //println!("Drawing!");
        unsafe {
             
            // gl.PointSize(20.0);

            let now = std::time::Instant::now();
            gl.BindVertexArray(self.vao); 
            gl.BindBuffer(GL_DRAW_INDIRECT_BUFFER, self.indbo);
            gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, self.ibo);

            let mut index_counts: Vec<i32> = Vec::new();
            let mut index_starts: Vec<*const c_void> = Vec::new();
            let mut base_vertices: Vec<i32> = Vec::new();
            // let mut index = 0;
            for command in &self.draw_commands {
                if command.indice_count != 0 {
                    index_counts.push(command.indice_count as i32);
                    index_starts.push(command.start as *const c_void);
                    base_vertices.push(command.base_vertex as i32);
                    // gl.DrawElementsIndirect(GL_POINTS, GL_UNSIGNED_INT, index as *const c_void);
                    // index += 20;
                    gl.DrawElementsInstancedBaseVertexBaseInstance(GL_TRIANGLES, command.indice_count as i32, GL_UNSIGNED_INT, command.start as *const c_void, command.instance_count as i32, command.base_vertex, command.base_instance);
                }  
            }

            // index_counts.remove(0);
            // index_starts.remove(0);
            // base_vertices.remove(0);
            
           // gl.BindBuffer(GL_SHADER_STORAGE_BUFFER, self.ssbo);
            //gl.BindBufferBase(GL_SHADER_STORAGE_BUFFER, 3, self.ssbo);
            //println!("Count {}", self.drawcount);
            //gl.DrawElementsInstanced(GL_TRIANGLES, self.mesh_capacity as i32, GL_UNSIGNED_INT, null(), 2);
            // gl.MultiDrawElementsIndirect(GL_TRIANGLES, GL_UNSIGNED_INT, 0 as *const c_void, self.drawcount as i32, 0);
            // println!("draw command elapsed {}ms", now.elapsed().as_millis());
            //println!("counts {:?}", index_counts);
            //println!("starts {:?}", index_starts);
            //println!("bases {:?}", base_vertices);
           //gl.MultiDrawElementsBaseVertex(GL_TRIANGLES, index_counts.as_ptr(), GL_UNSIGNED_INT, index_starts.as_ptr(), index_counts.len() as i32, base_vertices.as_ptr());
        
        }
    }
}

// Each IDC represents one thing we're finna draw
#[derive(Debug)]
#[repr(C)] // make sure rust doesn't rearrange the order of the stuff in here and break everything, since we memcpy this into opengl
struct IndirectDrawCommand {
    indice_count : u32, // number of indices to draw
    instance_count : u32, // will always be 1 because we aren't instancing
    start : u32, // position in buffer of first index
    base_vertex : i32, // number added to every index on draw (so we don't have to modify each mesh's indices)
    base_instance : u32,
}

impl IndirectDrawCommand {
    fn new() -> Self {
        return Self {indice_count:0, instance_count:1, start:0, base_vertex:0, base_instance:0};
    }

    fn clone(&self) -> IndirectDrawCommand {
        return IndirectDrawCommand { indice_count: self.indice_count, instance_count: self.instance_count, start: self.start, base_vertex: self.base_vertex, base_instance: self.base_instance }
    }
}