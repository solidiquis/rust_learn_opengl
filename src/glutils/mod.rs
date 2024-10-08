pub mod model;
pub mod shader;

pub trait GLenum {
    fn into_glenum(self) -> gl::types::GLenum;
}
