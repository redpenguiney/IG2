// this little trick lets us basically have multiple files in same crate so we don't have really long namespaces
pub use gameobject::*;
pub use meshobject::*;
pub use renderable::*;
pub use collisions::*;
pub use physmeshobject::*;
pub use rigidbody::*;
pub use rigidmeshobject::*;
mod gameobject;
mod meshobject;
mod renderable;
mod collisions;
mod physmeshobject;
mod rigidbody;
mod rigidmeshobject;