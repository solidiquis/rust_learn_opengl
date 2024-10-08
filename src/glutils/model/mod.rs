use super::GLenum;
use crate::glutils::shader::program::Program;
use anyhow::{format_err, Result};
use std::{ffi::c_void, mem, ptr};

pub struct Model {
    vertex_array_object: gl::types::GLuint,
}

pub struct ModelBuilder<const M: usize, const N: usize> {
    position_vertices: Vertices<M>,
    color_vertices: Option<Vertices<N>>,
    usage: Usage,
    program: Program,
}

pub struct Vertices<const N: usize> {
    attribute_name: String,
    values: [f32; N],
    component_size: usize,
    normalized: bool,
}

#[derive(Default)]
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

impl<const N: usize> Vertices<N> {
    pub fn new(attr: &str, values: [f32; N], component_size: usize, normalized: bool) -> Self {
        Self {
            attribute_name: attr.to_string(),
            values,
            component_size,
            normalized,
        }
    }
}

impl Model {
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_object);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl<const M: usize, const N: usize> ModelBuilder<M, N> {
    pub fn new(program: Program, usage: Usage, position_vertices: Vertices<M>) -> Self {
        Self {
            program,
            usage,
            position_vertices,
            color_vertices: None,
        }
    }

    pub fn color_vertices(mut self, vertices: Vertices<N>) -> Self {
        self.color_vertices = Some(vertices);
        self
    }

    pub fn build(self) -> Result<Model> {
        unsafe { self.build_impl() }
    }

    unsafe fn build_impl(self) -> Result<Model> {
        let mut vbo = 0;
        let mut vao = 0;

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let program = self.program;
        let Vertices {
            attribute_name: pos_attr_name,
            values: pos_vals,
            component_size: pos_compsize,
            normalized: pos_normalized,
        } = self.position_vertices;

        if pos_vals.len() % pos_compsize != 0 {
            return Err(format_err!(
                "length of position data must be evenly divisible my component size"
            ));
        }
        let pos_attr_loc = program.get_attrib_loc(&pos_attr_name)?;
        let pos_attr_normalized = if pos_normalized { gl::TRUE } else { gl::FALSE };

        match self.color_vertices {
            None => {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<f32>() * pos_vals.len()).try_into().unwrap(),
                    pos_vals.as_ptr() as *const c_void,
                    self.usage.into_glenum(),
                );
                gl::VertexAttribPointer(
                    pos_attr_loc,
                    pos_compsize.try_into().unwrap(),
                    gl::FLOAT,
                    pos_attr_normalized,
                    0,
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(pos_attr_loc);
            }
            Some(color_data) => {
                let Vertices {
                    attribute_name: col_attr_name,
                    values: col_vals,
                    component_size: col_compsize,
                    normalized: col_normalized,
                } = color_data;

                if col_vals.len() % col_compsize != 0 {
                    return Err(format_err!(
                        "length of color data must be evenly divisible my component size"
                    ));
                }

                let num_pos_vertices = pos_vals.len() / pos_compsize;
                let num_col_vertices = col_vals.len() / col_compsize;

                if num_pos_vertices != num_col_vertices {
                    return Err(format_err!("position and color vertices length mismatch"));
                }

                let stride = mem::size_of::<f32>() * (pos_compsize + col_compsize);
                let mut data_to_buffer = Vec::with_capacity(col_vals.len() + pos_vals.len());

                for (pos_vertex, col_vertex) in pos_vals
                    .chunks(pos_compsize)
                    .zip(col_vals.chunks(col_compsize))
                {
                    data_to_buffer.extend_from_slice(pos_vertex);
                    data_to_buffer.extend_from_slice(col_vertex);
                }
                let data_ptr = data_to_buffer.as_ptr() as *const c_void;
                let col_attr_normalized = if col_normalized { gl::TRUE } else { gl::FALSE };

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (mem::size_of::<f32>() * data_to_buffer.len())
                        .try_into()
                        .unwrap(),
                    data_ptr,
                    self.usage.into_glenum(),
                );

                gl::VertexAttribPointer(
                    pos_attr_loc,
                    pos_compsize.try_into().unwrap(),
                    gl::FLOAT,
                    pos_attr_normalized,
                    stride.try_into().unwrap(),
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(pos_attr_loc);

                let col_attr_loc = program.get_attrib_loc(&col_attr_name)?;
                let col_offset = mem::size_of::<f32>() * pos_compsize;

                gl::VertexAttribPointer(
                    col_attr_loc,
                    col_compsize.try_into().unwrap(),
                    gl::FLOAT,
                    col_attr_normalized,
                    stride.try_into().unwrap(),
                    col_offset as *const c_void,
                );
                gl::EnableVertexAttribArray(col_attr_loc);
            }
        }
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        Ok(Model {
            vertex_array_object: vao,
        })
    }
}
