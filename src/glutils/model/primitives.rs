pub enum Primitive {
    Triangles,
}

impl From<Primitive> for gl::types::GLenum {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Triangles => gl::TRIANGLES,
        }
    }
}
