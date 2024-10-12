use super::GLenum;
use super::{as_gl_bool, try_into, vf32_size};
use crate::glutils::shader::program::Program;
use anyhow::{format_err, Result};
use std::{ffi::c_void, mem, ptr};

pub mod usage;
use usage::Usage;

pub mod primitives;
use primitives::Primitive;

pub struct Model {
    vertex_array_object: gl::types::GLuint,
    num_vertices: usize,
    program: Program,
}

pub struct ModelBuilder<const M: usize, const N: usize> {
    position_attributes: VertexAttribute<M>,
    color_attributes: Option<VertexAttribute<N>>,
    usage: Usage,
    program: Program,

    vbo_num_elements: usize,
    stride: usize,
}

pub struct VertexAttribute<const N: usize> {
    name: String,
    values: [f32; N],
    component_size: usize,
    normalized: bool,
}

impl<const N: usize> VertexAttribute<N> {
    pub fn new(attr: &str, values: [f32; N], component_size: usize, normalized: bool) -> Self {
        Self {
            name: attr.to_string(),
            values,
            component_size,
            normalized,
        }
    }
}

impl Model {
    pub fn draw_arrays(&self, primitive: Primitive) {
        unsafe {
            gl::DrawArrays(primitive.into_glenum(), 0, try_into!(self.num_vertices));
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program.gl_object_id);
        }
    }

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
    pub fn new(
        program: Program,
        usage: Usage,
        position_attributes: VertexAttribute<M>,
    ) -> Result<Self> {
        let num_values = position_attributes.values.len();
        if num_values % position_attributes.component_size != 0 {
            return Err(format_err!(
                "number of values for color attribute should be divisible by component size"
            ));
        }

        let stride = mem::size_of::<f32>() * position_attributes.component_size;

        Ok(Self {
            program,
            usage,
            position_attributes,
            stride,
            vbo_num_elements: num_values,
            color_attributes: None,
        })
    }

    pub fn color_attributes(mut self, vertices: VertexAttribute<N>) -> Result<Self> {
        let num_values = vertices.values.len();
        if num_values % vertices.component_size != 0 {
            return Err(format_err!(
                "number of values for color attribute should be divisible by component size"
            ));
        }
        self.vbo_num_elements += num_values;
        self.stride += mem::size_of::<f32>() * vertices.component_size;
        self.color_attributes = Some(vertices);
        Ok(self)
    }

    pub fn build(self) -> Result<Model> {
        unsafe { self.build_impl() }
    }

    unsafe fn build_impl(&self) -> Result<Model> {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        let num_vertices =
            self.position_attributes.values.len() / self.position_attributes.component_size;

        let mut attribute_data_iters = vec![self
            .position_attributes
            .values
            .chunks(self.position_attributes.component_size)];

        if let Some(color_attrs) = self.color_attributes.as_ref() {
            if color_attrs.values.len() / color_attrs.component_size != num_vertices {
                return Err(format_err!(
                    "number of color vertices should match number of position vertices"
                ));
            }
            attribute_data_iters.push(color_attrs.values.chunks(color_attrs.component_size));
        }

        let mut buffer = Vec::with_capacity(self.vbo_num_elements);

        for i in (0..attribute_data_iters.len()).cycle() {
            let Some(chunk) = attribute_data_iters.get_mut(i).and_then(|a| a.next()) else {
                break;
            };
            buffer.extend_from_slice(chunk);
        }

        gl::BufferData(
            gl::ARRAY_BUFFER,
            try_into!(vf32_size(&buffer)),
            buffer.as_ptr() as *const c_void,
            self.usage.into_glenum(),
        );

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let byte_offset = mem::size_of::<f32>() * self.position_attributes.component_size;

        /*
         * Position attribute
         */
        let VertexAttribute {
            name,
            component_size,
            normalized,
            ..
        } = &self.position_attributes;
        let pos_attr_loc = self.program.get_attrib_loc(name)?;
        gl::VertexAttribPointer(
            pos_attr_loc,
            try_into!(*component_size),
            gl::FLOAT,
            as_gl_bool(*normalized),
            try_into!(self.stride),
            ptr::null(),
        );
        gl::EnableVertexAttribArray(pos_attr_loc);

        /*
         * Color attribute
         */
        if let Some(VertexAttribute {
            name,
            component_size,
            normalized,
            ..
        }) = self.color_attributes.as_ref()
        {
            let col_attr_loc = self.program.get_attrib_loc(name)?;
            gl::VertexAttribPointer(
                col_attr_loc,
                try_into!(*component_size),
                gl::FLOAT,
                as_gl_bool(*normalized),
                try_into!(self.stride),
                byte_offset as *const c_void,
            );
            gl::EnableVertexAttribArray(col_attr_loc);
            // For when we texture coordinates are supported
            // byte_offset += mem::size_of::<f32>() * component_size;
        }

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        Ok(Model {
            num_vertices,
            program: self.program,
            vertex_array_object: vao,
        })
    }
}
