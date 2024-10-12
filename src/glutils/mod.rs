pub mod model;
pub mod shader;

pub trait GLenum {
    fn into_glenum(self) -> gl::types::GLenum;
}

#[macro_export]
macro_rules! try_into {
    ($i:expr) => {
        $i.try_into().unwrap()
    };
}
pub use try_into;

fn vf32_size(vf32: &[f32]) -> usize {
    std::mem::size_of_val(vf32)
}

fn as_gl_bool(val: bool) -> gl::types::GLboolean {
    if val {
        gl::TRUE
    } else {
        gl::FALSE
    }
}
