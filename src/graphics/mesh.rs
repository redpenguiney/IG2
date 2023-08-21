use nalgebra_glm::{Vec3, vec3};

// WAIT: WHY DON'T WE INSTANCE DYNAMIC MESHES? IF SOMEONE WANTS TO MODIFY TWO CURRENTLY IDENTICAL CUBES IN DIFFERENT WAYS, THEY SHOULD JUST CLONE THE MESHES, RIGHT?
// TODO: GET THAT WORKING

pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    //format: TODO,
    pub texture_id: u32, // IMPORTANT: if you change texture/shader of a mesh, it must be removed and then readded to mesh manager. 
    pub shader_id: u32,   // todo: need method to do that

    pub uuid: usize, // if meshmaster knows two non-dynamic meshes are equuivalent, it will try to instance them for optimization
    pub dynamic: bool,

    pub original_size: Vec3 // Collision detection relies on meshes being 1m^3 and their size being changed solely through the scale property of transform
                            // Thus, when a mesh is made its vertex positions are scaled into the range -0.5 to 0.5
                            // To make your mesh the right size, you can set its scale to this automatically set property
}

pub const N_FLOATS_PER_VERTEX: usize = 8; 
pub const N_VERTEX_ATTRIBS: usize = 3;

static LAST_MESH_UUID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// TODO: if locking the mutex slows GraphicsEngine::add_renderable(), use unsyncronized version of stuff
static LOADED_MESHES: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashMap<usize, Mesh>>> = once_cell::sync::Lazy::new(|| {std::sync::Mutex::new(std::collections::HashMap::new())}); // key is mesh uuid, value is mesh

// returns value to put into original size
// makes all vertex coords in range -0.5 to 0.5
fn scale_vertices_into_range(vertices: &mut Vec<f32>) -> Vec3 {
    let (mut minx, mut miny, mut minz, mut maxx, mut maxy, mut maxz, mut i) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0);

    // figure out how big mesh is so we know how much to divide positions by
    for v in vertices.iter_mut() {
        if i % N_FLOATS_PER_VERTEX == 0 { // x pos
            minx = minx.min(*v);
            maxx = maxx.max(*v);
        }
        else if i % N_FLOATS_PER_VERTEX == 1 { // y pos
            miny = miny.min(*v);
            maxy =maxy.max(*v);
        }
        else if i % N_FLOATS_PER_VERTEX == 2 { // z pos
            minz = minz.min(*v);
            maxz = maxz.max(*v);
        }

        i+=1;
        if i == 8 {i = 0}
    }

    

    i = 0;
    for v in vertices {
        if i % N_FLOATS_PER_VERTEX == 0 { // x pos
            *v = 1.0*(*v-minx)/(maxx-minx) - 0.5; // i don't really know how this bit works i got it from stack overflow and modified it
        }
        else if i % N_FLOATS_PER_VERTEX == 1 { // y pos
            *v = 1.0*(*v-minx)/(maxx-minx) - 0.5;
        }
        else if i % N_FLOATS_PER_VERTEX == 2 { // z pos
            *v = 1.0*(*v-minx)/(maxx-minx) - 0.5;
        }

        i+=1;
        if i == 8 {i = 0}
    }

    i = 0;

    let size = vec3((maxx-minx).abs(), (maxy-miny).abs(), (maxz-minz).abs());
    return size;
}

impl Mesh {
    pub fn from_vertices(mut vertices: Vec<f32>, indices: Vec<u32>, texture_id: u32, shader_id: u32, dynamic: bool) -> Mesh {
        let og_size = scale_vertices_into_range(&mut vertices);
        return Mesh {
            vertices: vertices,
            indices: indices,
            texture_id: texture_id,
            shader_id: shader_id,
            uuid: LAST_MESH_UUID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            dynamic: dynamic,
            original_size: og_size
        }
    }

    pub fn clone(&self) -> Mesh {
        return Mesh { vertices: self.vertices.clone(), indices: self.indices.clone(), texture_id: self.texture_id, shader_id: self.shader_id, uuid: self.uuid, dynamic: self.dynamic, original_size: self.original_size }
    }

    // returns mesh uuid, mesh itself can be accessed as needed (like if it's a dynamic mesh) by using uuid to index into LOADED_MESHES
    pub fn from_obj(path: &str, textureId: u32, shaderId: u32) -> usize {
        let options = tobj::LoadOptions {single_index: true, triangulate: true, ignore_points: true, ignore_lines: true};
        let (models, materials) = tobj::load_obj(path, &options).expect(&("Failed to load OBJ file at path ".to_owned() + path));
        let model = &models[0];
        
        // TODO: THIS IS PROBABLY SLOW
        // this exists because obj file format be like POSPOSPOSPOSPOS NORMALNORMALNORMAL TEXTURETEXTURETEXTURE and we be like POSNORMALTEXTURE POSNORMALTEXTURE
        let mut verts = Vec::new();
        let mut vert_index = 0;
        let mut pos_index = 0;
        let mut normal_index = 0;
        let mut tex_index = 0; 
        let mut vert_attrib = 0;
        let total = model.mesh.positions.len() * N_FLOATS_PER_VERTEX / N_VERTEX_ATTRIBS;
        while vert_index < total {
            if vert_attrib == N_FLOATS_PER_VERTEX {vert_attrib = 0;}
            if vert_attrib < 3 {verts.push(model.mesh.positions[pos_index]); pos_index += 1;}
            else if vert_attrib < 6 {verts.push(model.mesh.normals[normal_index]); normal_index += 1;}
            else if vert_attrib < 8 {verts.push(model.mesh.texcoords[tex_index]); tex_index += 1;}
            //else {verts.push(textureZ);}
            vert_index += 1;
            vert_attrib += 1;
        }

        println!("Created mesh with {} vertices.", verts.len()/N_FLOATS_PER_VERTEX);
        let og_size = scale_vertices_into_range(&mut verts);
        let m = Mesh {
            vertices: verts,
            indices: model.mesh.indices.clone(),
            texture_id: textureId,
            shader_id: shaderId,
            uuid: LAST_MESH_UUID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            dynamic: false,
            original_size: og_size
        };
        let uuid = m.uuid;
        LOADED_MESHES.lock().unwrap().insert(uuid, m);
        return uuid;
    }
}