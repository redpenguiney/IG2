use glm::{Vec3, vec3};

use crate::gameobjects::ColliderType;

const ONE_TWELTH: f32 = 0.08333333333;

// inertia tensor is 3x3 matrix used to get moment of inertia (how hard to rotate thing is)
// everything except diagonals is 0 since we're rotating around center of mass, so just a vec3
pub fn moment_of_inertia(collider_type: ColliderType, size: Vec3, mass: f32) -> Vec3 {
    match collider_type {
        ColliderType::Sphere => { // https://scienceworld.wolfram.com/physics/MomentofInertiaSphere.html
            let radius = size.x * 0.5;
            vec3(0.4 * mass * radius.powi(2), 0.4 * mass * radius.powi(2), 0.4 * mass * radius.powi(2))
        }
        ColliderType::Box => { //http://mechanicsmap.psu.edu/websites/centroidtables/centroids3D/centroids3D.html
            vec3(ONE_TWELTH * mass * (size.y.powi(2) + size.z.powi(2)), ONE_TWELTH * mass * (size.x.powi(2) + size.z.powi(2)), ONE_TWELTH * mass * (size.y.powi(2) + size.x.powi(2)))
        }
        _ => {
            panic!("not implemented");
        }
    }
}