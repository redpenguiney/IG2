// this little trick lets us basically have multiple files in same crate so we don't have really long namespaces
pub mod window;
pub use window::*;
pub mod input;
pub use input::*;