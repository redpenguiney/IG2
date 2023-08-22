// this little trick lets us basically have multiple files in same crate so we don't have really long namespaces
pub use gameobject::*;
pub use meshobject::*;
pub use renderable::*;
mod gameobject;
mod meshobject;
mod renderable;