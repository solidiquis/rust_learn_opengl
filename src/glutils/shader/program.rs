use super::Shader;
use anyhow::{format_err, Result};
use std::{ffi::CString, mem, ptr};

#[derive(Copy, Clone)]
pub struct Program {
    pub gl_object_id: gl::types::GLuint,
}

pub struct Linker {
    shaders: Vec<Shader>,
    program: gl::types::GLuint,
}

impl Program {
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.gl_object_id);
        }
    }

    pub fn get_attrib_loc(&self, attrib: &str) -> Result<gl::types::GLuint> {
        let c_attrib = CString::new(attrib)?;
        unsafe {
            let loc = gl::GetAttribLocation(self.gl_object_id, c_attrib.as_ptr());
            if loc == -1 {
                return Err(format_err!(
                    "failed to query location of attribute '{attrib}'"
                ));
            }
            Ok(loc as gl::types::GLuint)
        }
    }

    pub fn get_uniform_loc(&self, uniform: &str) -> Result<gl::types::GLint> {
        let c_uniform = CString::new(uniform)?;
        unsafe {
            let loc = gl::GetUniformLocation(self.gl_object_id, c_uniform.as_ptr());
            if loc == -1 {
                return Err(format_err!(
                    "failed to query location of uniform '{uniform}'"
                ));
            }
            Ok(loc)
        }
    }
}

impl Linker {
    pub fn new() -> Self {
        let shaders = vec![];
        let program = unsafe { gl::CreateProgram() };
        Self { shaders, program }
    }

    pub fn attach_shader(mut self, shader: Shader) -> Self {
        self.shaders.push(shader);
        self
    }

    pub fn link(self) -> Result<Program> {
        unsafe {
            for Shader(shader) in &self.shaders {
                gl::AttachShader(self.program, *shader);
            }

            gl::LinkProgram(self.program);

            let mut success = 0;
            gl::GetProgramiv(self.program, gl::LINK_STATUS, &mut success);

            if success == 0 {
                let mut log: [u8; 512] = [0; 512];
                gl::GetProgramInfoLog(
                    self.program,
                    (mem::size_of::<u8>() * log.len()).try_into().unwrap(),
                    ptr::null_mut(),
                    log.as_mut_ptr() as *mut i8,
                );
                let reason = String::from_utf8_lossy(&log);

                return Err(format_err!(
                    "an error occurred while linking program: {}",
                    reason.trim()
                ));
            }

            for Shader(shader) in self.shaders {
                gl::DeleteShader(shader);
            }

            Ok(Program {
                gl_object_id: self.program,
            })
        }
    }
}
