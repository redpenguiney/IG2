
use glm::{Vec3, vec3, I64Vec3, Quat};

use crate::transform::*;
use crate::gameobjects::*;

use super::collisions::Collides;

// pub struct RigidBody<> {
//     density: f32,
//     friction: f32,
//     elasticity: f32,

//     transform: Arc<Mutex<Transform>>,
//     velocity: Vec3,
//     angular_velocity: Vec3,

//     collider_type: ColliderType,
// }

// impl <> RigidBody<> {
//     pub fn new(t: Arc<Mutex<Transform>>) -> Self {
//         return Self {density: 1.0, friction: 0.5, elasticity: 0.2, transform: t, velocity: vec3(0.0, 0.0, 0.0), angular_velocity: vec3(0.0, 0.0, 0.0), collider_type: ColliderType::Box  }
//     }

// }

pub trait RigidBody: GameObject + ObjectTransform + Collides {
    //fn impulse(&mut self, )
    fn mass(&self) -> f32; // no setter because based off size * density

    fn density(&self) -> f32;
    
    fn velocity(&self) -> I64Vec3;
    fn angular_velocity(&self) -> Vec3;

    fn density_mut(&mut self) -> &mut f32;
    
    fn local_velocity_at_point(&self, point: glm::Vec3) -> glm::I64Vec3;
    fn velocity_at_point(&self, point: glm::Vec3) -> glm::I64Vec3;

    fn velocity_mut(&mut self) -> &mut I64Vec3;
    fn angular_velocity_mut(&mut self) -> &mut Vec3;

    fn set_density(&mut self, density: f32);
    
    fn set_velocity(&mut self, velocity: I64Vec3);
    fn set_angular_velocity(&mut self, angular_velocity: Vec3);

    fn impulse(&mut self, force: Vec3);

    fn impulse_at_pos(&mut self, force: Vec3, rel_pos: I64Vec3);

    fn torque_from_force_at_pos(&mut self, force: Vec3, rel_pos: I64Vec3);

    fn inertia_tensor(&self) -> Vec3;
    // fn inertia_tensor_mut(&mut self) -> &mut Vec3;
}

#[macro_export]
macro_rules! impl_rigidbody {
    ($structname: ident) => {
        impl crate::gameobjects::RigidBody for $structname {
            fn impulse(&mut self, force: glm::Vec3) {
                // F/M = A
                
                let A = force/self.mass();
                //println!("A = {:?}", A);
                //println!("A = {:?}", A);
                *self.velocity_mut() += crate::transform::i64vec3_from_vec3(&A);
            }

            fn impulse_at_pos(&mut self, force: glm::Vec3, rel_pos: glm::I64Vec3) {
                self.impulse(force);
                self.torque_from_force_at_pos(force, rel_pos);
            }

            fn torque_from_force_at_pos(&mut self, force: Vec3, rel_pos: I64Vec3) {
                let rel_pos_f32 =crate::transform::vec3_from_i64vec3(&rel_pos);
                let torque_dir = rel_pos_f32.cross(&force);
                let torque_mag = rel_pos_f32.magnitude() * (force.magnitude() * rel_pos_f32.normalize().angle(&force.normalize()).sin());
                //println!("TorqueDir = {:?}, DirMag = {:?}, TorqueMag = {}", torque_dir, torque_dir.magnitude(), torque_mag);
                //println!("Relpos = {:?}, force = {:?}, angle = {}", rel_pos_f32, force, rel_pos_f32.normalize().angle(&force.normalize()));
                let torque = torque_dir;// println!("Torque is {:?}", torque);
                let tensor_at_center = self.inertia_tensor();
                //let tensor_at_point = tensor_at_center + (vec3(self.mass(), self.mass(), self.mass()).component_mul(&rel_pos_f32)); // uses parallel axis theorem
                //println!("torque = {:?}, dir {:?}, mag = {:?}", torque, torque_dir, torque_mag);
                let change_in_angular_velocity_around_point = torque.component_div(&tensor_at_center);

                *self.angular_velocity_mut() += change_in_angular_velocity_around_point;
                //println!("CHANGING ANGULAR VELOCITY BY {:?}", change_in_angular_velocity_around_point);
            }

            // TODO: PROBABLY VERY SLOW
            fn inertia_tensor(&self) -> glm::Vec3 {
                return crate::phys::moment_of_inertia(self.collider_type, self.transform.scl(), self.mass())
            }

            // fn inertia_tensor_mut(&mut self) -> &mut glm::Vec3 {
            //     return &mut self.inertia_tensor;
            // }

            // ALSO QUITE SLOW PROBABLY
            fn mass(&self) -> f32 {
                match self.collider_type {
                    ColliderType::Sphere => (4.0/3.0) * 3.1415 * self.density * self.transform.volume(),
                    _ => self.density * self.transform.volume()
                }
            }

            // point in local space
            fn velocity_at_point(&self, point_in_local_space: glm::Vec3) -> glm::I64Vec3 {
                
                //println!("Rotating at {:?}", self.angular_velocity() * r);
                return self.local_velocity_at_point(point_in_local_space) + self.velocity;
            }

            fn local_velocity_at_point(&self, point_in_local_space: glm::Vec3) -> glm::I64Vec3 {
                let direction = self.angular_velocity().cross(&point_in_local_space);
                let r = point_in_local_space.magnitude();
                return crate::transform::i64vec3_from_vec3(&(direction * r));
            }

            fn density(&self) -> f32 {
                return self.density;
            }
 
            fn velocity(&self) -> glm::I64Vec3 {
                return self.velocity;
            }

            fn angular_velocity(&self) -> glm::Vec3 {
                return self.angular_velocity;
            }



            fn density_mut(&mut self) -> &mut f32 {
                return &mut self.density;
            }

            fn velocity_mut(&mut self) -> &mut glm::I64Vec3 {
                return &mut self.velocity;
            }

            fn angular_velocity_mut(&mut self) -> &mut glm::Vec3 {
                return &mut self.angular_velocity;
            }

            
            
            fn set_density(&mut self, density: f32) {
                assert!(density != 0.0);
                self.density = density;
            }

            fn set_velocity(&mut self, velocity: glm::I64Vec3) {
                self.velocity = velocity;
            }

            fn set_angular_velocity(&mut self, angular_velocity: glm::Vec3) {
                self.angular_velocity = angular_velocity;
            }

        }  
    }
}