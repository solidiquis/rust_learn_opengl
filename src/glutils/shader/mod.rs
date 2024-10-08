use super::GLenum;
use anyhow::{format_err, Result};
use std::{
    convert::AsRef,
    ffi::{CString, OsStr},
    fs, mem,
    path::Path,
    ptr,
};

pub mod program;

pub struct Shader(u32);

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl GLenum for ShaderType {
    fn into_glenum(self) -> gl::types::GLenum {
        match self {
            Self::Vertex => gl::VERTEX_SHADER,
            Self::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

impl Shader {
    pub fn new<P: AsRef<Path>>(src: P, typ: ShaderType) -> Result<Self> {
        let shader_path = src.as_ref();
        let shader_src = fs::read_to_string(src.as_ref())
            .map_err(|e| format_err!("failed to load '{}': {e}", shader_path.display()))?;
        let file_name = shader_path
            .file_name()
            .map(OsStr::to_string_lossy)
            .ok_or(format_err!("expected shader to have a file name"))?;
        let shader = Self::compile_src(&file_name, &shader_src, typ)?;
        Ok(Self(shader))
    }

    fn compile_src(file_name: &str, src: &str, typ: ShaderType) -> Result<gl::types::GLuint> {
        let shader_type = typ.into_glenum();
        let shader_src = CString::new(src)?;

        unsafe {
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(shader, 1, &shader_src.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                let mut log: [u8; 512] = [0; 512];
                gl::GetShaderInfoLog(
                    shader,
                    (mem::size_of::<u8>() * log.len()).try_into().unwrap(),
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut i8,
                );
                let reason = String::from_utf8_lossy(&log);

                return Err(format_err!(
                    "an error occurred while compiling {file_name}: {}",
                    reason.trim()
                ));
            };
            Ok(shader)
        }
    }
}
