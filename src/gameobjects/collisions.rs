use std::collections::VecDeque;

use glm::{I64Vec3, Vec3, vec3, vec4, Mat4};


use crate::transform::*;


#[derive(Clone, Copy, PartialEq)]
pub enum ColliderType {
    Sphere,
    Convex,
    Box
}

pub struct CollisionInfo {
    pub normal: Vec3,
    pub collision_points: Vec<(I64Vec3, i64)> // vec of (hitPos, hitPenetration)
}

// Used by SAT
#[derive(Clone, Debug)]
struct Plane {
    normal: Vec3,
    pos: Vec3 // any pos
}

impl Plane {
    fn signed_distance(&self, point: &Vec3) -> f32 {
        return self.normal.dot(&(point - self.pos));
    }
}

fn sat_plane_loop(obj1_verts: &Vec<Vec3>, obj2_planes: &Vec<Plane>, least_normal: &mut Vec3, model_to_world: &Mat4, obj1_pos: &I64Vec3, collision_points: &mut Vec<(I64Vec3, i64)>) {
    'outer: for v in obj1_verts.iter() { // if any vertices of obj2 are on wrong side of all of planes1 or vice versa we colliding
        let mut least_penetration = f32::MAX;
        let mut least_normal_in_here = vec3(0.0, 0.0, 0.0);
        //println!("TESTING VERTEX AT {:?}", v);
        for p in obj2_planes.iter() {
            let distance = p.signed_distance(v);
           // println!("\tVertex at {:?} was {} away from plane {:?}", v, distance, p);
            if distance <= 0.0 {
                if distance.abs() < least_penetration {
                    //println!("\tPrev penetration {} has beaten by {}, so normal has gone from {:?} to {:?}", least_penetration, distance, least_normal, p.normal);
                    least_normal_in_here = p.normal;
                    least_penetration = distance.abs();
                }
            }
            else {
                //println!("No penetration.");
                continue 'outer;
            }
        }

        // if we didn't continue, there must have been a collision
        *least_normal = least_normal_in_here;
        let penetration = ((least_penetration * UNITS_PER_METER as f32) as i64);
       // println!("NORMAL IS {:?}, P={}", least_normal, penetration);
        let pos = i64vec3_from_vec3(&(model_to_world * vec4(v.x, v.y, v.z, 1.0)).xyz()) + obj1_pos;
        collision_points.push((pos, penetration));
        
    }
}

pub fn collision_SAT(obj1: &(dyn Collides), obj2: &(dyn Collides)) -> Option<CollisionInfo> {
    // get planes of each cube and their vertices
    // if any of these planes can seperate all vertices of each object, they aren't colliding

    // obj1 is treated as at origin by transforming obj2  
    let mat = obj1.transform().world_to_model();
    let model_to_world = mat.try_inverse().unwrap();
    let obj2_rel_pos = vec3_from_i64vec3(&(obj2.transform().pos() - obj1.transform().pos()));
    let transformed_obj2_rel_pos = (mat * vec4(obj2_rel_pos.x, obj2_rel_pos.y, obj2_rel_pos.z, 1.0)).xyz();

    //println!("CUBE1 AT {:?}", obj1.transform().pos());
    //println!("CUBE2 RELATIVELY AT {:?}, really at {:?}  while cube1 is really at {:?}", transformed_obj2_rel_pos, obj2.transform().pos(), obj1.transform().pos());

    let mut verts2 = Vec::new();
    
    //let mut i = 0;
    for x in [-0.5, 0.5] {
        for y in [-0.5, 0.5] {
            for z in [-0.5, 0.5] {
                verts2.push(transformed_obj2_rel_pos + (mat * obj2.transform().rotscalemat *vec4(x, y, z, 1.0)).xyz());
                //i += 1;
            }
        }
    }
    //println!("Verts2 = {:?}", verts2);
    

    let verts1 = vec!(
        glm::vec3(-0.5, -0.5, -0.5),
        glm::vec3(-0.5, 0.5, -0.5),
        glm::vec3(0.5, -0.5, -0.5),
        glm::vec3(0.5, 0.5, -0.5),
        glm::vec3(-0.5, -0.5, 0.5),
        glm::vec3(-0.5, 0.5, 0.5),
        glm::vec3(0.5, -0.5, 0.5),
        glm::vec3(0.5, 0.5, 0.5),
    );

    let planes1: Vec<Plane> = vec![
        Plane {normal: vec3( 0.0,  1.0,  0.0), pos: vec3( 0.0,  0.5,  0.0)},
        Plane {normal: vec3( 0.0, -1.0,  0.0), pos: vec3( 0.0, -0.5,  0.0)},
        Plane {normal: vec3( 1.0,  0.0,  0.0), pos: vec3( 0.5,  0.0,  0.0)},
        Plane {normal: vec3(-1.0,  0.0,  0.0), pos: vec3(-0.5,  0.0,  0.0)},
        Plane {normal: vec3( 0.0,  0.0,  1.0), pos: vec3( 0.0,  0.0,  0.5)},
        Plane {normal: vec3( 0.0,  0.0, -1.0), pos: vec3( 0.0,  0.0, -0.5)}
    ];

    let mut planes2 = Vec::new();

    for mut p in planes1.clone() {
        p.normal = (obj2.transform().rotatemat() * mat * vec4(p.normal.x, p.normal.y, p.normal.z, 1.0)).xyz();
        // if obj2.transform().pos().x < 0 {p.normal.x *= -1.0};
        // if obj2.transform().pos().y < 0 {p.normal.y *= -1.0};
        // if obj2.transform().pos().z < 0 {p.normal.z *= -1.0};
        //p.pos.component_mul_assign(&obj2.transform().scl()); //println!("multiply by {:?}", obj2.transform().scl());
        p.pos = transformed_obj2_rel_pos + (mat * obj2.transform().rotscalemat *vec4(p.pos.x, p.pos.y, p.pos.z, 1.0)).xyz();
        planes2.push(p);
    }

    //println!("PLANES2 = {:?}", planes2);

    let mut obj1_collision_points: Vec<(I64Vec3, i64)> = Vec::new();
    let mut obj1_least_normal = vec3(0.0, 0.0, 0.0);
    sat_plane_loop(&verts1, &planes2, &mut obj1_least_normal, &model_to_world, &obj1.transform().pos(), &mut obj1_collision_points);
    
    let mut obj2_collision_points: Vec<(I64Vec3, i64)> = Vec::new();
    let mut obj2_least_normal = vec3(0.0, 0.0, 0.0);
    sat_plane_loop(&verts2, &planes1, &mut obj2_least_normal, &(&obj2.transform().world_to_model().try_inverse().unwrap()), &obj2.transform().pos(), &mut obj2_collision_points);    

    //panic!();
    // 
    if obj1_collision_points.len() + obj2_collision_points.len() == 0 {
        //println!("SAT NO FOUND COLLISION");
        return None;
    }
    else {
        if obj1_collision_points.len() == 0 {
            obj1_least_normal = multiply_vec_by_matrix(&-obj2_least_normal, &obj2.transform().rotatemat());
        }
        else {
            obj1_least_normal = multiply_vec_by_matrix(&obj1_least_normal, &obj1.transform().rotatemat())
        }
        obj1_collision_points.extend(obj2_collision_points);
        
        // for point in obj1_collision_points {
        //     point
        // }

        //println!("SAT CONFIRMED COLLISION, normal={:?}", obj1_least_normal);
        //loop {}
        return Some(CollisionInfo { normal: obj1_least_normal, collision_points: obj1_collision_points});
    }
    
}

pub trait Collides: for<'a> ObjectTransform + crate::GameObject{
    fn get_collider_type(&self) -> ColliderType;
    fn collides_with(&self, other: &(dyn Collides)) -> Option<CollisionInfo>; 
    // fn get_colliding(&self, )

    fn elasticity(&self) -> f32;
    fn friction(&self) -> f32;
    
    fn friction_mut(&mut self) -> &mut f32;
    fn elasticity_mut(&mut self) -> &mut f32;

    fn set_elasticity(&mut self, elasticity: f32);
    fn set_friction(&mut self, friction: f32); 
}

// todo: use ints maybe
// needed for GJK
//returns the point on shape which has the highest dot product with direction
pub fn find_furthest_point(verts: &Vec<Vec3>, direction: Vec3) -> Vec3 {
    let mut dot_product = -10.0;
    let mut best_point = vec3(-1.0, -1.0, -1.0);
    for v in verts {
        if v.dot(&direction) > dot_product {
            best_point = *v;
            dot_product = v.dot(&direction);
        }
    }
    return best_point;
    //for ()
}

pub fn support(verts1: &Vec<Vec3>, verts2: &Vec<Vec3>, direction: Vec3) -> Vec3 {
    //println!("SUPPORT IS {:?}", find_furthest_point(verts1, direction) - find_furthest_point(verts2, -direction));
    return find_furthest_point(verts1, direction) - find_furthest_point(verts2, -direction);
}

// pub fn simplex_contains_origin(simplex: &Vec<Vec3>) -> bool {
//     return false;
// }

// pub fn get_simplex_direction(simplex: &Vec<Vec3>) -> Vec3 {
//     return vec3(0.0, 0.0, 0.0);
// }

// returns true if collision detected, otherwise sets direction and modifies simplex for next iteration of GJK
pub fn next_simplex(simplex: &mut VecDeque<Vec3>, direction: &mut Vec3) -> bool {
    match simplex.len() {
        2 => { // line case
            line(simplex, direction)
        }
        3 => {
            triangle(simplex, direction)
        }
        4 => {
            tetrahedron(simplex, direction)
        }
        _ => panic!("Simplex had wrong number of points??? {:?}", simplex)
    }
}

fn line(simplex: &mut VecDeque<Vec3>, direction: &mut Vec3) -> bool { // helper function for next_simplex
    //println!("REACHED LINE CASE");

    let a = simplex[0];
    let b = simplex[1];
    let a_to_b = b - a;
    let a_to_origin = -a;
    if a_to_b.dot(&a_to_origin) > 0.0 {
        *direction = a_to_b.cross(&a_to_origin).cross(&a_to_b)
    }
    else {
        simplex.pop_back();
        *direction = a_to_origin;
    }
    false
}

fn triangle(simplex: &mut VecDeque<Vec3>, direction: &mut Vec3) -> bool { // helper function for next_simplex
    //println!("REACHED TRI CASE");

    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];

    let a_to_b = b - a;
    let a_to_c = c - a;
    let a_to_origin = -a;

    let ab_to_c = a_to_b.cross(&a_to_c);

    if ab_to_c.cross(&a_to_c).dot(&a_to_origin) > 0.0 {
        //println!(" failed first");
        if a_to_c.dot(&a_to_origin) > 0.0 {
            simplex.remove(1);
            *direction = a_to_c.cross(&a_to_origin).cross(&a_to_c);
        }
        else {
            simplex.pop_back();
            return line(simplex, direction);
        }
    }
    else {
        if a_to_b.cross(&ab_to_c).dot(&a_to_origin) > 0.0 {
            simplex.pop_back();
            return line(simplex, direction)
        }
        else {
            
            if ab_to_c.dot(&a_to_origin) > 0.0 {
                *direction = ab_to_c;
                //println!("direction!!!! set to {:?}", ab_to_c);
            }
            else {
                
                *simplex = [a, c, b].into();
                *direction = -ab_to_c;
            }
        }
    }

    false
}

fn tetrahedron(simplex: &mut VecDeque<Vec3>, direction: &mut Vec3) -> bool { // helper function for next_simplex
    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];
    let d = simplex[3];

    let a_to_b = b - a;
    let a_to_c = c - a;
    let a_to_d = d - a;
    let a_to_origin = -a;

    let tri_abc = a_to_b.cross(&a_to_c);
    let tri_acd = a_to_c.cross(&a_to_d);
    let tri_adb = a_to_d.cross(&a_to_b);

   // println!("REACHED TETRA CASE");

    if tri_abc.dot(&a_to_origin) > 0.0 {
        *simplex = vec![a, b, c].into();
       // println!("same direction as abc");
        return triangle(simplex, direction);
        
    }

    if tri_acd.dot(&a_to_origin) > 0.0 {
        *simplex = vec![a, c, d].into();
        //println!("same direction as acd");
        return triangle(simplex, direction);
    }

    if tri_adb.dot(&a_to_origin) > 0.0 {
        *simplex = vec![a, d, b].into();
       // println!("same direction as adb");
        return triangle(simplex, direction);
    }

    //println!("PASSED TETRA CASE");
    true
}

pub fn get_face_normals(polyhedron: &VecDeque<glm::Vec3>, faces: &Vec<usize>) -> (Vec<glm::Vec4>, usize) { // helper function for EPA 
    let mut normals: Vec<glm::Vec4> = Vec::new();
    let mut min_triangle = 0;
    let mut min_distance = f32::MAX;

    println!("Getting face normals...");
    for i in (0..faces.len()).step_by(3) {
        let a = polyhedron[faces[i]];
        let b = polyhedron[faces[i + 1]];
        let c = polyhedron[faces[i + 2]];

        let mut normal = (b - a).cross(&(c - a)).normalize();
        let mut distance = normal.dot(&a);

        if distance < 0.0 {
            normal *= -1.0;
            distance *= -1.0;
        }

        normals.push(vec4(normal.x, normal.y, normal.z, distance)); println!("Pushed simplex normal");

        if distance < min_distance {
            min_triangle = 1/3;
            min_distance = distance;
        }
    }

    println!("Got face normals");
    return (normals, min_triangle)
}

pub fn add_if_unique_edge(edges: &mut Vec<(usize, usize)>, faces: &Vec<usize>, a: usize, b: usize) {
    let reverse_index = edges.iter().position(|&r| r == (b, a));
    if reverse_index.is_some() {
        println!("Did not find unique edge");
        edges.remove(reverse_index.unwrap());
    }
    else {
        println!("Found unique edge, indexing with {} and {} into \n\t{:?}", a, b, faces);
        edges.push((faces[a], faces[b]));
    }
}

#[macro_export]
macro_rules! impl_collides {
    ($structname: ident) => {
        
        impl crate::gameobjects::Collides<> for $structname {
            
            fn get_collider_type(&self) -> crate::gameobjects::ColliderType {
                return self.collider_type;
            }

            

            fn friction(&self) -> f32 {
                return self.friction;
            }

            

            fn friction_mut(&mut self) -> &mut f32 {
                return &mut self.friction;
            }

            fn set_friction(&mut self, friction: f32) {
                self.friction = friction;
            }

            fn elasticity_mut(&mut self) -> &mut f32 {
                return &mut self.elasticity;
            }

            fn set_elasticity(&mut self, elasticity: f32) {
                self.elasticity = elasticity;
            }

            fn elasticity(&self) -> f32 {
                return self.elasticity;
            }

            // returns none if no collision, returns Some((penetrationDepth, collisionNormal, localCollisionPoint)) if there was one
            fn collides_with(&self, other: &(dyn crate::gameobjects::Collides)) -> Option<crate::gameobjects::CollisionInfo> {

                //convex to convex, uses GJK from https://www.youtube.com/watch?v=MDusDn8oTSE&ab_channel=Winterdev
                // this algorithm works for anything convex
                // if self.collider_type == crate::gameobjects::ColliderType::Convex && other.get_collider_type() == crate::gameobjects::ColliderType::Convex { 
                //     //  translate self to origin (or pretend we did) and translate other by same amount to avoid FP errors
                //     panic!("CONVEX DOESNT WORK DONT USE IT");
                //     let mat = other.transform().world_to_model();
                //     let mut verts1 = Vec::new();
                //     let other_rel_pos = vec3_from_i64vec3(&(other.transform().pos() - self.transform().pos()));
                //     let transformed_other_rel_pos = (mat * vec4(other_rel_pos.x, other_rel_pos.y, other_rel_pos.z, 1.0)).xyz();
                //     // get vertex locations of other to calculate minoswki difference
                //     //let mut i = 0;
                //     for x in [-0.5, 0.5] {
                //         for y in [-0.5, 0.5] {
                //             for z in [-0.5, 0.5] {
                //                 verts1.push(transformed_other_rel_pos + (mat * self.transform().rotscalemat * vec4(x, y, z, 1.0)).xyz());
                //                 //i += 1;
                //             }
                //         }
                //     }

                //     println!("{:?}",verts1);

                //     let verts2 = vec!(
                //         glm::vec3(-0.5, -0.5, -0.5),
                //         glm::vec3(-0.5, 0.5, -0.5),
                //         glm::vec3(0.5, -0.5, -0.5),
                //         glm::vec3(0.5, 0.5, -0.5),
                //         glm::vec3(-0.5, -0.5, 0.5),
                //         glm::vec3(-0.5, 0.5, 0.5),
                //         glm::vec3(0.5, -0.5, 0.5),
                //         glm::vec3(0.5, 0.5, 0.5),
                //     );

                //     let mut direction = glm::vec3(1.0, 0.0, 1.0); // arbitrary starting search direction
                //     let mut simplex = std::collections::VecDeque::with_capacity(4); // tetrahedron that envelopes origin if colliding, made from 4 supports
                //     let mut support = crate::gameobjects::support(&verts1, &verts2, direction); // support is farthest vertex along direction
                //     simplex.push_front(support);
                //     direction = -support;
                //     //println!("BEGINNING TEST");
                //     loop {
                //         support = crate::gameobjects::support(&verts1, &verts2, direction);

                //         if support.dot(&direction) <= 0.0 {
                //            // println!("THAT SUPPORT DOTS WITH DISTANCE {:?} TO {:?}", direction, support.dot(&direction));
                //             return None;
                //         }

                //         //println!("ITS ALRIGHT NEVER GONNA GIVE YOU UP");

                //         simplex.push_front(support);
                //         if crate::gameobjects::next_simplex(&mut simplex, &mut direction) {
                //             break;
                //         }
                //     }

                //     println!("CUBE COLLIDES");
                //     // GJK only tells us if the polygon is colliding, we have to use EPA (https://blog.winter.dev/2020/epa-algorithm/) to get normal
                //     let mut polyhedron = simplex;
                //     let mut faces = vec!( // faces of polyhedron not the meshes
                //         0, 1, 2, 
                //         0, 3, 1,
                //         0, 2, 3,
                //         1, 3, 2
                //     );

                //     // vec4 of normal + distance
                //     let (mut normals, mut min_face) = crate::gameobjects::collisions::get_face_normals(&polyhedron, &faces);
                //     println!("NORMALS GENERATED WERE {:?}", normals);
                //     let mut min_normal = glm::vec3(0.0, 0.0, 0.0);
                //     let mut min_distance = f32::MAX;
                //     while min_distance == f32::MAX {
                //         min_normal = normals[min_face].xyz();
                //         min_distance = normals[min_face].w;
                        
                //         let support2 = crate::gameobjects::collisions::support(&verts1, &verts2, min_normal);
                //         let s_distance = min_normal.dot(&support2);

                //         if (s_distance - min_distance).abs() > 0.001 {
                //             println!("a");
                //             min_distance = f32::MAX;

                //             let mut unique_edges: Vec<(usize, usize)> = Vec::new();

                //             let mut i = 0;
                //             println!("beginning b-loop");
                //             assert!(normals.len() * 3 <= faces.len(), " WE DIDN'T EVEN START, there are {} normals and {} faces ", normals.len(), faces.len());
                //             while i < normals.len() {
                //                 assert!(normals.len() * 3 <= faces.len(), " BRUH ");
                //                 println!("b.1");
                //                 if normals[i].xyz().dot(&support2) < 0.0 {
                //                     let f = i * 3;
                //                     println!("b.2");
                //                     println!("F is {}, i is {}, len is {}", f, i, normals.len());
                //                     crate::gameobjects::collisions::add_if_unique_edge(&mut unique_edges, &faces, f, f + 1);
                //                     crate::gameobjects::collisions::add_if_unique_edge(&mut unique_edges, &faces, f + 1, f + 2);
                //                     crate::gameobjects::collisions::add_if_unique_edge(&mut unique_edges, &faces, f + 2, f);
                //                     println!("b.3");
                //                     faces[f + 2] = *faces.last().unwrap();
                //                     faces.pop();
                //                     faces[f + 1] = *faces.last().unwrap();
                //                     faces.pop();
                //                     faces[f] = *faces.last().unwrap();
                //                     faces.pop();
                //                     println!("b.4");
                //                     normals[i] = *normals.last().unwrap(); 
                //                     normals.pop();
                //                     //assert!(normals.len() * 3 == faces.len(), " BRO ");

                //                     i -= 1;
                //                 }
                //                 i += 1;
                //                 println!("i={}, len={}", i, normals.len());
                //                 //assert!(normals.len() * 3 == faces.len(), " WHY ");
                //             }

                //             println!("c");
                //             //println!("At c unique edges are {:?}", unique_edges);
                //             let mut new_faces = Vec::new();
                //             for (edge_index1, edge_index2) in unique_edges {
                //                 new_faces.push(edge_index1);
                //                 new_faces.push(edge_index2);
                //                 new_faces.push(polyhedron.len());
                //             }

                //             polyhedron.push_back(support);

                //             let (new_normals, new_min_face) = crate::gameobjects::collisions::get_face_normals(&polyhedron, &new_faces);
                //             //println!("Normals at d are {:?} despite us giving it {:?} faces", new_normals, new_faces);
                //             println!("d");
                //             let mut old_min_distance = f32::MAX;
                //             for i in 0..normals.len() {
                //                 if normals[i].w < old_min_distance {
                //                     old_min_distance = normals[i].w;
                //                     min_face = i;
                //                 }
                //             }
                //             println!("e");
                //             println!("Min face is {}", new_min_face);
                //             if new_normals[new_min_face].w < old_min_distance {
                //                 min_face = new_min_face + normals.len();
                //             }
                //             println!("f");
                //             println!("Before extension there are {} faces and {} normals", faces.len(), normals.len());
                //             faces.extend(new_faces);
                //             normals.extend(new_normals);
                //             println!("After extension there are {} faces and {} normals", faces.len(), normals.len());

                            
                //         }
                //     }

                //     // may have to add tiny decimal to min_distance?
                //     // TODO: HIT POS
                //     assert!(min_normal != glm::vec3(0.0, 0.0, 0.0));
                //     return None;
                // }

                // box-box, uses SAT 
                // could be pretty easily adapted to any convex shape, although SAT does NOT scale for large meshes, should use GJK for that
                if self.collider_type == crate::gameobjects::ColliderType::Box && other.get_collider_type() == crate::gameobjects::ColliderType::Box {
                    return crate::gameobjects::collision_SAT(self, other);
                }
                
                // sphere to box
                else if (self.collider_type == crate::gameobjects::ColliderType::Sphere && other.get_collider_type() == crate::gameobjects::ColliderType::Box) || (self.collider_type == crate::gameobjects::collisions::ColliderType::Box && other.get_collider_type() == crate::gameobjects::collisions::ColliderType::Sphere) { 
                    // transform both cube and sphere by same matrix so cube is at origin and unrotated 
                    let cube: &dyn crate::gameobjects::Collides;
                    let sphere: &dyn crate::gameobjects::Collides;
                    if self.collider_type == crate::gameobjects::ColliderType::Sphere {
                        sphere = self;
                        cube = other;
                    }
                    else {
                        sphere = other;
                        cube = self;
                    }
                    let mat = cube.transform().unrotate();
                    let sphere_pos: glm::Vec3 = crate::transform::vec3_from_i64vec3(&(sphere.transform().pos() - cube.transform().pos()));
                    let transformed_sphere_pos = mat * vec4(sphere_pos.x, sphere_pos.y, sphere_pos.z, 1.0);
                    //let radius =
                    let extents = cube.transform().scl() * 0.5;
                    //println!("Transformed sphere to {:?} instead of {:?}", sphere_pos, transformed_sphere_pos);

                    // find closest point on box to sphere
                    let closest = glm::vec3((-extents.x).max(transformed_sphere_pos.x).min(extents.x), (-extents.y).max(transformed_sphere_pos.y).min(extents.y), (-extents.z).max(transformed_sphere_pos.z).min(extents.z));
                    //println!("Closest point was {:?}", closest);

                    // if distance between that point and sphere center is less than radius then they touching
                    //println!("R={:?}", sphere.transform().scl().x/2.0);
                    let min_distance_squared = (sphere.transform().scl().x/2.0).powf(2.0) as f64 * (crate::transform::UNITS_PER_METER.pow(2)) as f64; // didn't want to use doubles here but FP error, if can fix yay
                    let v = crate::transform::i64vec3_from_vec3(&transformed_sphere_pos.xyz()) - crate::transform::i64vec3_from_vec3(&closest);
                    //println!("Vec = {:?}", v);
                    let distance_squared = v.x*v.x + v.y*v.y + v.z*v.z;
                    //println!("Distance was {} out of {}", distance_squared, min_distance_squared);
                    if min_distance_squared as i64 >= distance_squared {
                        let mat2 = mat.try_inverse().unwrap();
                        let mut normal = glm::vec3(0.0, 0.0, 0.0);
                        if closest.y == extents.y {
                            normal.y = 1.0;
                        }
                        else if closest.y == -extents.y {
                            normal.y = -1.0;
                        }
                        else if closest.x == extents.x {
                            normal.x = 1.0;
                        }
                        else if closest.x == -extents.x {
                            normal.x = -1.0;
                        }
                        else if closest.z == extents.z {
                            normal.z = 1.0;
                        }
                        else if closest.z == -extents.z {
                            normal.z = -1.0;
                        }
                        let hitpos = crate::transform::i64vec3_from_vec3(&(mat2 * vec4(closest.x, closest.y, closest.z, 1.0)).xyz()) + cube.transform().pos();
                        println!("Extents were {:?} so pos is {:?}", extents, hitpos);
                        let info = crate::gameobjects::collisions::CollisionInfo {
                            normal: (mat2 * glm::vec4(normal.x, normal.y, normal.z, 1.0)).xyz(),
                            collision_points: vec![(hitpos, (distance_squared as f64).sqrt() as i64 - ((sphere.transform().scl().x/2.0) as f64 * crate::transform::UNITS_PER_METER as f64) as i64)]
                        };
                        return Some(info);
                    }
                    else {
                        return None;
                    }
                }

                // sphere to sphere
                else if self.collider_type == crate::gameobjects::collisions::ColliderType::Sphere && other.get_collider_type() == crate::gameobjects::collisions::ColliderType::Sphere { 
                    let min_distance_squared = (self.transform.scl().x/2.0 + other.transform().scl().x/2.0).powf(2.0) as i64 * (crate::transform::UNITS_PER_METER).pow(2);
                    let v = self.transform.pos() - other.transform().pos();
                    let distance_squared = v.x*v.x + v.y*v.y + v.z*v.z; //println!("Distance squared was {} out of {}", distance_squared, min_distance_squared);
                    if min_distance_squared >= distance_squared {
                        let info = crate::gameobjects::collisions::CollisionInfo {
                            normal: crate::transform::vec3_from_i64vec3(&v),
                            collision_points: vec![(crate::transform::i64vec3_from_vec3(&(crate::transform::vec3_from_i64vec3(&v) * ((self.transform.scl().x/2.0)))) + self.transform.pos(), (distance_squared as f64).sqrt() as i64 - (self.transform.scl().x/2.0 + other.transform().scl().x/2.0) as i64 * (crate::transform::UNITS_PER_METER))]
                        };
                        return Some(info)
                    }
                    else {
                        return None;
                    }
                }


                else {
                    panic!("UH OH, apparently i added a collision type and didn't code reactions for it!");
                }
                
            }
            
        }  
    }
}
