use std::{rc::Rc, cell::RefCell};

use crate::transform::*;
use glm::*;

use super::SpatialAccelerationStructure;

pub const GRAVITY: i64 = (-1.807 * 0.016 as f64 * UNITS_PER_METER as f64) as i64;

pub fn do_physics(sas: &mut SpatialAccelerationStructure, rigidbodies: &Vec<Rc<RefCell<dyn crate::gameobjects::RigidBody>>>) {
    //std::thread::sleep(time::Duration::from_millis(500));
    // println!("DOING PHYSICS OH NO");
    //let mut positions = Vec::new();
    //let mut dbnormal = vec3(0.0, 0.0, 0.0);
    //let mut debug_pos_cuz_it_hit = false;
    for obj_cell in rigidbodies {
        let mut obj = obj_cell.borrow_mut(); 

        //gravity
        *obj.velocity_mut() += i64vec3(0, GRAVITY, 0);


        let possible_colliding = sas.query_aabb(&super::AABB::new(obj.transform()));
        //println!("Possible colliding {:?}", possible_colliding);
        //println!("i am at {:?}", ptr);
        //println!(" pos {:?}: {:?}", unsafe{&**ptr}.transform(), possible_colliding);
        //println!("jk pos {:?}: {:?}", unsafe{&**ptr}.transform(), possible_colliding);
        for obj2_cell in possible_colliding { 
            if obj_cell.as_ptr() as *const () as usize == obj2_cell.as_ptr() as *const () as usize { // ignore a self-collision for obvious reasons
                //println!("Obviously {} is touching {}", obj.name(), other.name());
                continue;
            };
            let other = obj2_cell.borrow_mut();
            let mass = obj.mass();
            
            let collision = obj.collides_with(&*other); // penetration, normal, hitpos
            if collision.is_some() {
                let normal =collision.as_ref().unwrap().normal;
                //dbnormal = normal;
                let mut greatest_penetration = 0;
                for point in collision.as_ref().unwrap().collision_points.iter() {
                    let penetration = point.1;
                    if penetration > greatest_penetration {
                        greatest_penetration = penetration;
                    }
                    //println!("COLLISION!!!! {}, normal {:?}", penetration, normal);
                    
                    //println!("Moving up by {:?} to compensate", (i64vec3_from_vec3(&normal) * penetration.abs())/UNITS_PER_METER);
                    
                    //println!("V = {:?} but at point its {:?}", obj.velocity(), v);
                    //println!("Res:?ponding with force {:?} ", i64vec3_from_vec3(&(normal * (1.0 + e)));//.component_mul(&-v));
                    //let speed_towards_wall = vec3_from_i64vec3(&v).dot(&normal);
                
                    //println!("Collision speed = {}", obj.mass());

                    //let friction_speed = vec3_from_i64vec3(&v) - collision_speed;
                    //let f = obj.friction() * other.friction();
                    //let friction_force = -friction_speed * (penetration.abs() as f32 / UNITS_PER_METER as f32) * f;
                    //obj.impulse_at_pos(friction_force, hitpos - obj.transform().pos());
                    //positions.push(point.0);
                }
                
                *obj.transform_mut().pos_mut() += (i64vec3_from_vec3(&normal) * greatest_penetration.abs())/UNITS_PER_METER;

                let mut total_pos = i64vec3(0, 0, 0);
                collision.as_ref().unwrap().collision_points.iter().for_each(|x| total_pos += x.0);
                let mut hitpos = total_pos/collision.as_ref().unwrap().collision_points.len() as i64;
                hitpos -= obj.transform().pos();
               
                let e = obj.elasticity() * other.elasticity() + 1.0;
                let v = obj.velocity_at_point(vec3_from_i64vec3(&hitpos));
                //let collision_speed = -normal * (vec3_from_i64vec3(&v)).component_mul(&normal).magnitude();
                //let force = -collision_speed * obj.mass() * (1.0 + e);
                
                
                let desired_change_in_velocity = -e * normal * (normal.dot(&&vec3_from_i64vec3(&v))/(&(normal.dot(&normal))) );
                // println!("Desired change in speed is {:?}", desired_change_in_velocity);
                println!("OBJ velocity is {:?}, but at point its {:?}", obj.velocity(), v);
                
                obj.torque_from_force_at_pos(vec3_from_i64vec3(&-v) * mass, hitpos);
                obj.impulse_at_pos(desired_change_in_velocity * mass, hitpos);
                
                //dbnormal = vec3_from_i64vec3(&-v).normalize();
                
                //positions.push(hitpos + obj.transform().pos());
                
                //debug_pos_cuz_it_hit = true;
                
            }
            else {
                //println!("{} could have been colliding with {}, but it wasn't", obj.name(), other.name());
            }
        }

        // velocity step
        
        let v = obj.velocity()/60;
        //println!(" V = {:?}", v);
        //let av = obj.angular_velocity();
        *obj.angular_velocity_mut() *= 0.99;
        *(obj.transform_mut().pos_mut()) += v;
        let av = obj.angular_velocity()/60.0;
        obj.transform_mut().rotatex(av.x);
        obj.transform_mut().rotatey(av.y);
        obj.transform_mut().rotatez(av.z);
    }

    // if debug_pos_cuz_it_hit {
    //     for p in positions {
    //         let mut debug_hitpos = MeshObject::new(&mut self.apis, Box::new(&(Mesh::from_obj("models/rainbowcube.obj", 0, 7))), -1.0);
    //         debug_hitpos.transform.setpos(p);
    //         debug_hitpos.transform.setscl(vec3(0.1, 0.1, 0.4));
    //         debug_hitpos.transform.set_look_vector(&dbnormal);
    //         println!("NORMAL IS {:?}", dbnormal);
    //         self.load_object(Box::new(debug_hitpos));

    //         //println!("NORMAL IS {:?}", dbnormal);
    //     }
    //     //loop {}
    // }

    
    

}