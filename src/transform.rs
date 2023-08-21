use std::time::Instant;

use nalgebra_glm::{Vec3, vec3, I64Vec3, Number, DVec3, TVec3, vec4};

pub const UNITS_PER_METER : i64 = 1000000; // 10^6 um per m

#[derive(Debug)]
pub struct Transform {
    pos: I64Vec3, // position is always in micrometers for int types and in meters for float types. 
    rot: nalgebra_glm::Quat,
    scl: nalgebra_glm::Vec3,
    pub rotscalemat: nalgebra_glm::Mat4x4
}

pub fn i64vec3_mag(vec: &I64Vec3) -> i64 {
    return ((vec.x * vec.x + vec.y * vec.y + vec.z * vec.z) as f64).sqrt() as i64;
}

pub fn i64vec3<T>(x: T, y: T, z: T) -> I64Vec3 where i64: From<T> {
    return I64Vec3::new(x.into(), y.into(), z.into());
}

// NOTE: DOUBLES ARE NOT USED IN PHYSICS OR GRAPHICS, AND ARE ONLY HERE SO YOU CAN CONVIENENTLY VIEW/SPECIFIY NUMBERS IN METERS (f64 because fp precision errors)
pub fn dvec3(x: f64, y: f64, z: f64) -> DVec3 {
    return TVec3::new(x, y, z);
}

pub fn i64vec3_from_vec3(vec: &Vec3) -> I64Vec3 {
    return i64vec3::<i64>((vec.x * UNITS_PER_METER as f32) as i64, (vec.y * UNITS_PER_METER as f32) as i64, (vec.z * UNITS_PER_METER as f32) as i64);
}

pub fn i64vec3_from_dvec3(vec: &DVec3) -> I64Vec3 {
    
    return i64vec3::<i64>((vec.x * UNITS_PER_METER as f64) as i64, (vec.y * UNITS_PER_METER as f64) as i64, (vec.z * UNITS_PER_METER as f64) as i64);
}

pub fn dvec3_to_vec3(vec: DVec3) -> Vec3 {
    return vec3(vec.x as f32, vec.y as f32, vec.z as f32);
}

pub fn vec3_from_i64vec3(vec: &I64Vec3) -> Vec3 {
    let v = vec3(vec.x as f32/UNITS_PER_METER as f32, vec.y as f32/UNITS_PER_METER as f32, vec.z as f32/UNITS_PER_METER as f32);
    return v;
}

pub fn dvec3_from_i64vec3(vec: &I64Vec3) -> DVec3 {
    let v = vec3(vec.x as f64/UNITS_PER_METER as f64, vec.y as f64/UNITS_PER_METER as f64, vec.z as f64/UNITS_PER_METER as f64);
    return v;
}

pub fn multiply_vec_by_matrix(vec: &Vec3, mat: &nalgebra_glm::Mat4) -> Vec3 {
    return (mat * vec4(vec.x, vec.y, vec.z, 1.0)).xyz();
}

// TODO: If neccesary, a possible optimization could be making it so changing position only recalculates right column of matrix, and changing scale/rot only touches left 3 columns
impl Transform {

    pub fn new(pos_in_um: I64Vec3) -> Self { 
        let mut t = Self {
            pos: pos_in_um,
            rot: nalgebra_glm::quat_identity(),
            scl: vec3(1.0, 1.0, 1.0),
            rotscalemat: nalgebra_glm::identity()
        };
        // t.update_rotscalemat();
        return t;
    }
    
    pub fn meters(pos_in_meters: DVec3) -> Self { 
        let mut t = Self {
            pos: i64vec3_from_dvec3(&pos_in_meters),
            rot: nalgebra_glm::quat_identity(),
            scl: vec3(1.0, 1.0, 1.0),
            rotscalemat: nalgebra_glm::identity()
        };
        // t.update_rotscalemat();
        return t;
    }

    pub fn empty() -> Self {
        return Transform::new(i64vec3(0, 0, 0))
    }

    pub fn setscl(&mut self, scl: nalgebra_glm::Vec3) {
        self.scl = scl;
        self.update_rotscalemat();
    }

    pub fn scl(&self) -> nalgebra_glm::Vec3 {
        return self.scl;
    }

    pub fn rotate_around_axis(&mut self, axis: nalgebra_glm::Vec3, angle: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(angle, &axis));
        // self.update_rotscalemat();
    }

    // rotates intrinsically (like euler angles)
    pub fn setrotyxz(&mut self, rot: nalgebra_glm::Vec3) {
        let mut fixed_rot = rot.clone();
        if fixed_rot.x >= 360.0 {fixed_rot.x = fixed_rot.x % 360.0};
        if fixed_rot.y >= 360.0 {fixed_rot.y = fixed_rot.y % 360.0};
        if fixed_rot.z >= 360.0 {fixed_rot.z = fixed_rot.z % 360.0};
        //println!("its {}", fixed_rot.y);
        //fixed_rot.x = 0.0;
        //fixed_rot.z = 0.0;
        self.rot = nalgebra_glm::quat_identity();
        self.yaw(rot.y);
        self.pitch(rot.x);
        self.roll(rot.z);
        // let y = nalgebra_glm::quat_angle_axis(fixed_rot.y, &vec3(0.0, 1.0, 0.0));
        // let x = nalgebra_glm::quat_angle_axis(fixed_rot.x, &vec3(1.0, 0.0, 0.0));
        // let z = nalgebra_glm::quat_angle_axis(fixed_rot.z, &vec3(0.0, 0.0, 1.0));
        // self.rot = y * x * z;       
        // self.update_rotscalemat();
        //println!("we have {}", self.rot());
    }

    // returns volume in meters^3 instead of um^3 because that is too big a number
    pub fn volume(&self) -> f32 {
        return self.scl.x * self.scl.y * self.scl.z;
    }

    //combines current rotation with given
    // pub fn rotateyxz(&mut self, rot: nalgebra_glm::Vec3) {
    //     self.rot *= nalgebra_glm::quat_angle_axis(rot.y, &vec3(0.0, 1.0, 0.0)) * nalgebra_glm::quat_angle_axis(rot.x, &vec3(1.0, 0.0, 0.0)) * nalgebra_glm::quat_angle_axis(rot.z, &vec3(0.0, 0.0, 1.0));     
    //     self.update_rotscalemat();
    // }

    pub fn get_look_vector(&self) -> Vec3 {
        let mut v = nalgebra_glm::quat_cross_vec(&self.rot.normalize().conjugate(), &nalgebra_glm::vec3(0.0, 0.0, 1.0));
        //v.x *= -1.0;
        
        //v.y *= -1.0;
        return v;
    }

    pub fn set_look_vector(&mut self, vec: &Vec3) {
        self.rot = nalgebra_glm::quat_look_at_rh(&vec, &vec3(0.0, 1.0, 0.0));
        self.update_rotscalemat();
    }

    pub fn get_up_vector(&self) -> Vec3 {
        return nalgebra_glm::quat_cross_vec(&self.rot.normalize().conjugate(), &nalgebra_glm::vec3(0.0,1.0, 0.0));
    }

    pub fn get_right_vector(&self) -> Vec3 {
        let mut v = nalgebra_glm::quat_cross_vec(&self.rot.normalize().conjugate(), &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        //v.z *= -1.0;
        return v;
    }

    // Locally rotates around y axis (rotates around up vector)
    pub fn yaw(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(rot, &self.get_up_vector()));
        self.update_rotscalemat();
    }

    // Locally rotates around x axis (rotates around right vector)
    pub fn pitch(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(rot, &self.get_right_vector()));
        self.update_rotscalemat();
    }

    // Locally rotates around z axis (rotates around look vector)
    pub fn roll(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(rot, &self.get_look_vector()));
        self.update_rotscalemat();
    }

    // Globally rotates around y axis
    pub fn rotatey(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot,&nalgebra_glm::quat_angle_axis(rot, &vec3(0.0, 1.0, 0.0)));
        self.update_rotscalemat();
    }

    // Globally rotates around x axis
    pub fn rotatex(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(rot, &vec3(1.0, 0.0, 0.0)));
        self.update_rotscalemat();
    }

    // Globally rotates around z axis
    pub fn rotatez(&mut self, rot: f32) {
        self.rot = nalgebra_glm::quat_cross(&self.rot, &nalgebra_glm::quat_angle_axis(rot, &vec3(0.0, 0.0, 1.0)));
        self.update_rotscalemat();
    }

    pub fn rot(&self) -> nalgebra_glm::Vec3 {
        return nalgebra_glm::quat_euler_angles(&self.rot);
    }


    pub fn pos_mut(&mut self) -> &mut I64Vec3 {
        return &mut self.pos;
    }

    pub fn setpos(&mut self, pos: I64Vec3) { // set position in MICROMETERS
        self.pos = pos;
        // self.update_rotscalemat()
    }

    pub fn setpos_meters(&mut self, pos: DVec3) {
        self.pos = i64vec3_from_dvec3(&pos);
    }

    pub fn pos(&self) -> I64Vec3 { // returns pos IN MICROMETERS
        return self.pos;
    }

    pub fn pos_meters(&self) -> DVec3 {
        return dvec3_from_i64vec3(&self.pos);
    }

    // TODO: translation direction is NOT relative to its rotation, should have an alt function when that behavior is undesired
    // returns a matrix that converts from model space to camera space (or to world space if cam_offset == (0, 0, 0))
    pub fn get_model(&self, cam_offset: &I64Vec3) -> nalgebra_glm::Mat4 {
        
        let pos = nalgebra_glm::translation(&vec3_from_i64vec3(&(self.pos - cam_offset)));
        //let now = Instant::now();
        let mat = pos * &self.rotscalemat;
    
        //println!("multiplying matrices took {:?}", now.elapsed());
        return mat;
    }

    // basically this lets us determine whether a transformed model is touching a untransformed model instead of whether two transformed model are colliding
    // returns a meaningless transformation matrix relative to a unit transform for collision detection
    // self becomes unit transform (not actually) 
    // floating point errors probably won't be an issue here BUT CHECK FOR THE LOVE OF GOD
    pub fn world_to_model(&self) -> nalgebra_glm::Mat4 {
        // unsafe { // I promise this is 100% safe, it would only break if it wasn't a square matrix, and Mat4 is square
           return self.rotscalemat.try_inverse().unwrap(); 
        // }
    }

    // TODO: OPTIMIZE, maybe by rotating by quaternion itself without matrix
    pub fn unrotate(&self) -> nalgebra_glm::Mat4 {
        return nalgebra_glm::quat_to_mat4(&self.rot).try_inverse().unwrap();
    }


    pub fn rotatemat(&self) -> nalgebra_glm::Mat4 {
        return nalgebra_glm::quat_to_mat4(&self.rot);
    }

    fn update_rotscalemat(&mut self) {
        self.rot = self.rot.normalize();
        self.rotscalemat = nalgebra_glm::quat_to_mat4(&self.rot) * nalgebra_glm::scaling(&self.scl);
    }

    // pub fn mat(&self) -> &nalgebra_glm::Mat4 {
    //     return &self.mat;
    // }

    // pub fn lookat(&mut self, target: Vec3) {
    //     self.mat = nalgebra_glm::look_at(&self.pos, &target, &vec3(0.0, 1.0, 0.0));
    // }

    pub fn clone(&self) -> Self {
        return Self { pos: self.pos, rot: self.rot, scl: self.scl, rotscalemat: self.rotscalemat }
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        return self.pos == other.pos && self.rot == other.rot && self.scl == other.scl;
    }
}

pub trait ObjectTransform {
    fn transform(&self) -> & Transform;
    fn transform_mut(&mut self) -> & mut Transform;
}

#[macro_export]
macro_rules! impl_transform {
    ($structname: ident) => {
        impl crate::transform::ObjectTransform for $structname { // TODO: wait for rust to add trait fields so we can get rid of this stupid stuff
            fn transform(&self) -> &Transform {
                return &self.transform;
            }
            
            fn transform_mut(&mut self) -> &mut Transform {
                return &mut self.transform;
            }
        }     
    };
}