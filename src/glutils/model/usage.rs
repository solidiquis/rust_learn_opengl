use super::super::GLenum;

#[derive(Default, Copy, Clone)]
pub enum Usage {
    #[default]
    Static,
    Stream,
    Dynamic,
}

impl GLenum for Usage {
    fn into_glenum(self) -> gl::types::GLenum {
        match self {
            Self::Stream => gl::STREAM_DRAW,
            Self::Static => gl::STATIC_DRAW,
            Self::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}
