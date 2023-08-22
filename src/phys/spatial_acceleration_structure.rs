// based on https://www.cs.nmsu.edu/~joshagam/Solace/papers/master-writeup-print.pdf
// used for fast broadphase collision detection, efficiently figuring out what could be colliding without doing expensive math
// todo: this uses so many pointers and is so unsafe
// ALSO NOT THREAD SAFE
// although the given pointers are just to ObjectTransform, they MUST actually point to things that implement Collides

#[allow(invalid_reference_casting)] // TODO: LINE 141 SPATIAL ACCELERATION STRUCTURE; use unsafe cell instead of casting &T to &mut T which is apparently UB in the nightlies

const SPLIT_THRESHOLD : usize = 10; // leaf node becomes internal and its contents are divided across 8 new leaves when it contains this many gameobjects

use std::{ops::Add, ptr::{null_mut, null}, collections::HashMap, fmt::Display};

use glm::{I64Vec3, vec3};

use crate::{gameobjects::GameObject, transform::{Transform, ObjectTransform, i64vec3}, gameobjects::Collides};

pub struct SpatialAccelerationStructure { // the root is always index 0
    nodes: Vec<Node>,
    obj_locations: HashMap<*mut dyn Collides, *mut Node>,
    //free_indices: Vec<i32> // before extending nodes, this vec should be emptied
}

impl SpatialAccelerationStructure {
    // fn GenLeafNode(&mut self, obj: *mut dyn GameObject, bbox: AABB) -> i32 { // returns index of newly generated node
    //     self.nodes.push(Node { aabb: (), parent_index: (), children_indices: (), gameobject_ptr: (), is_leaf: () });
    // }

    pub fn new() -> Self {
        return Self {nodes: Vec::new(), obj_locations: HashMap::new()};
    }

    pub fn insert(&mut self, obj: *mut dyn Collides, transform: &Transform) {
        println!("BEGINNING INSERTION");
        let bbox = AABB::new(transform);

        if self.nodes.is_empty() {
            self.nodes.push(Node { 
                aabb: bbox.clone(), 
                parent: null_mut(),
                children: Vec::new(), 
                gameobject_ptrs: vec![obj],
                gameobject_aabbs: vec![bbox],
                cannot_split: false
            }); 
            return;
        }

        

        // Traverse tree to find best leaf node to put aabb in

        let mut backtrace: Vec<*mut Node> = Vec::new(); // store all the nodes we went through here so we can expand their AABBs when done
        let mut n = &mut self.nodes[0];

        loop {
            if n.children.is_empty() { // if we're at a leaf in the tree then the object goes in here
                backtrace.push(n as *mut Node);
                n.gameobject_ptrs.push(obj);
                n.gameobject_aabbs.push(bbox.clone());
                n.cannot_split = false;
                self.obj_locations.insert(obj, n as *mut Node);
                break;
            }
            else { 
                let (x, y, z);
                // select best child to insert object into by getting octree coords
                // TODO: icoseptree would be better according to paper
                if (n).aabb.center.x > bbox.center.x {
                    x = 0;
                }
                else {
                    x = 1;
                }

                if (n).aabb.center.y > bbox.center.y {
                    y = 0;
                }
                else {
                    y = 1;
                }

                if (n).aabb.center.z > bbox.center.z {
                    z = 0;
                }
                else {
                    z = 1;
                }

                unsafe {
                    backtrace.push(n.children[x*4+y*2+z]);
                    n = &mut*n.children[x*4+y*2+z];
                    
                }
            }
        }

        // as needed expand all nodes to fit the additional aabb
        for node in backtrace {
            unsafe {
                println!("Prev size was {}", (*node).aabb);
                (*node).aabb.fit(&bbox);
                println!("Fitted node {:?} and is now size {} to fit aabb {}", node, (*node).aabb, bbox);
            }
        }


    }

    pub fn remove(&mut self, obj: *mut dyn ObjectTransform) {
        
    }

    // Querying is also when splitting nodes with too many objects happens
    // fn query_near(&mut self, obj: *mut dyn ObjectTransform, distance: i64) -> Vec<*mut dyn ObjectTransform> {
    //     let distance_squared = distance * distance;
    //     let mut n = self.obj_locations[&obj].clone();

    //     // set n to parent of n until its AABB is big enough to accomodate all 
        
    // }

    pub fn query_aabb(&mut self, bbox: &AABB) -> Vec<*mut dyn Collides> {
        if self.nodes.is_empty() {return Vec::new()}
        //println!("Querying acceleration structure for bounding box {}", bbox);
        let mut n = &self.nodes[0]; // start at root node
        'outer: loop {
            'inner: for child in n.children.iter() {
                if unsafe {&**child}.aabb.contains(bbox) {
                    n = unsafe{&**child};
                    continue 'outer;
                }
            }
            break 'outer // once we run out of children, n is the node with the smallest aabb that fully contains bbox
        }
        let ptr = n as *const Node as *mut Node;

        let mut touching = Vec::new();
        // once we have that node, recursively get all aabbs within it that even touch the bbox
        self.get_touching(n, bbox, &mut touching);
        
        // decide whether we need to split the leaf node by making it an empty node with 8 leaf children
        unsafe {
            let node = &*ptr;
            if !node.cannot_split && node.children.len() > SPLIT_THRESHOLD {
                println!("WE SHOULD SPLIT THE SAS BUT I DIDN\'T ADD THAT YET OH NO");
            }
        }
        

        return touching;
        
    }

    fn get_touching(&self, node: &Node, aabb: &AABB, vec: &mut Vec<*mut dyn Collides>) {
        for child in &node.children {
            self.get_touching(unsafe{&**child}, aabb, vec);
            //println!("i will be mad if this prints");
        }
        let mut i = 0;
        
        for obj_aabb in node.gameobject_aabbs.iter() {
            //println!(" a");
            if obj_aabb.touches(aabb) {
                vec.push(node.gameobject_ptrs[i]);
            }
            else {
                //println!("AABB {} doesn't touch AABB {}", obj_aabb, aabb);
            }
            i += 1;
            //println!("\t its {}", i);
        }
    }

}

// node stores children/parent as indices into nodes array
// todo: could use union (unsafe) to cut node size in half if the SAS takes up too much ram (unlikely lol)
struct Node {
    aabb: AABB,

    parent: *mut Node,
    children: Vec<*mut Node>,

    gameobject_ptrs: Vec<*mut dyn Collides>,
    gameobject_aabbs: Vec<AABB>,

    cannot_split: bool // if splitting tried to put all gameobjects in the same child (bc they're all in exact same pos, this is set to true until node is modified and we can try again)
}

impl Node {}

#[derive(Clone)]
pub struct AABB { // might have off-by-one errors with integer coords?
    min: I64Vec3,
    max: I64Vec3,
    center: I64Vec3
}

// aabb are oversized so rotation need not be considered and also because i haven't actually solved how collisions and velocity is gonna work
    // this will be very bad for something like a VERY long cylinder
// TODO: aabb assumes meshes are a base size of 1m^3 which they aren't
// we gotta make it so when meshes are loaded, they are scaled to fit into a 1m^3 volume
impl AABB {
    pub fn new(transform: &Transform) -> Self {
        let center = transform.pos();
        let size = transform.scl().x.max(transform.scl().y.max(transform.scl().z)) as f64;
        let distance = i64vec3((size * 1000000.0) as i64, (size * 1000000.0) as i64, (size * 1000000.0) as i64);
        return Self { min: center - distance, max: center + distance, center};
    }

    fn center(min: I64Vec3, max: I64Vec3) -> I64Vec3 {
        return i64vec3(max.x - min.x/2, max.y - min.y/2, max.z - min.z/2);
    }

    fn update_center(&mut self) {
        self.center = AABB::center(self.min, self.max);
    }

    fn fit(&mut self, other: &AABB) { //if self cannot contain other, self will expand such that it can
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
    }

    fn contains(&self, other: &AABB) -> bool { // returns true if self can contain other
        return self.min.x < other.min.x && self.min.y < other.min.y && self.min.z < other.min.z && self.max.x > other.max.x && self.max.y > other.max.y && self.max.z > other.max.z; 
    }

    fn touches(&self, other: &AABB) -> bool { // returns true if self is touching other
        return
            self.min.x < other.max.x &&
            self.max.x > other.min.x &&
            self.min.y < other.max.y &&
            self.max.y > other.min.y &&
            self.min.z < other.max.z &&
            self.max.z > other.min.z;
    }
}

impl Display for AABB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "min = {:?} max = {:?}", self.min, self.max)
    }
}