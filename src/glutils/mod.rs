pub mod model;
pub mod shader;

#[macro_export]
macro_rules! try_into {
    ($i:expr) => {
        $i.try_into().unwrap()
    };
}
pub use try_into;

fn as_gl_bool(val: bool) -> gl::types::GLboolean {
    if val {
        gl::TRUE
    } else {
        gl::FALSE
    }
}
