#[derive(Default, Copy, Clone)]
pub enum Usage {
    #[default]
    Static,
    Stream,
    Dynamic,
}

impl From<Usage> for gl::types::GLenum {
    fn from(value: Usage) -> gl::types::GLenum {
        match value {
            Usage::Stream => gl::STREAM_DRAW,
            Usage::Static => gl::STATIC_DRAW,
            Usage::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}
