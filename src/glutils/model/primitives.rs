use super::super::GLenum;

pub enum Primitive {
    Triangles,
}

impl GLenum for Primitive {
    fn into_glenum(self) -> gl::types::GLenum {
        match self {
            Self::Triangles => gl::TRIANGLES,
        }
    }
}
