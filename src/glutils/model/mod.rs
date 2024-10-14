use super::{as_gl_bool, try_into};
use crate::glutils::shader::program::Program;
use anyhow::{format_err, Result};
use std::{ffi::c_void, mem, ptr};

pub mod usage;
use usage::Usage;

pub mod primitives;
use primitives::Primitive;

pub struct Model {
    vertex_array_object: gl::types::GLuint,
    element_buffer_object: Option<gl::types::GLuint>,
    program: Program,
    num_vertices: gl::types::GLsizei,
    num_indices: gl::types::GLsizei,
    program_active: bool,
    vbo_bound: bool,
    ebo_bound: bool,
}

pub struct ModelBuilder {
    position_attributes: VertexAttribute,
    color_attributes: Option<VertexAttribute>,
    indices: Option<Vec<u32>>,
    usage: Usage,
    program: Program,

    vbo_num_elements: gl::types::GLsizei,
    stride: gl::types::GLsizei,
}

pub struct VertexAttribute {
    name: String,
    values: Vec<f32>,
    component_size: gl::types::GLint,
    normalized: bool,
}

impl VertexAttribute {
    pub fn new(
        attr: &str,
        values: Vec<f32>,
        component_size: gl::types::GLint,
        normalized: bool,
    ) -> Self {
        Self {
            name: attr.to_string(),
            values,
            component_size,
            normalized,
        }
    }
}

impl Model {
    pub fn try_draw_arrays(&mut self, primitive: Primitive) -> Result<()> {
        if !self.vbo_bound && !self.program_active {
            return Err(format_err!(
                "shader program needs to be active and VBO needs to be bound"
            ));
        }
        unsafe { self.try_draw_arrays_impl(primitive) }
        Ok(())
    }

    pub fn use_program(&mut self) {
        unsafe { self.use_program_impl() }
    }

    pub fn bind(&mut self) {
        unsafe { self.bind_impl() }
    }

    pub fn unbind(&mut self) {
        unsafe { self.unbind_impl() }
    }

    unsafe fn try_draw_arrays_impl(&mut self, primitive: Primitive) {
        if self.element_buffer_object.is_some_and(|_| self.ebo_bound) {
            gl::DrawElements(
                primitive.into(),
                self.num_indices,
                gl::UNSIGNED_INT,
                ptr::null(),
            )
        } else {
            gl::DrawArrays(primitive.into(), 0, try_into!(self.num_vertices));
        }
    }

    unsafe fn use_program_impl(&mut self) {
        gl::UseProgram(self.program.gl_object_id);
        self.program_active = true;
    }

    unsafe fn unbind_impl(&mut self) {
        gl::BindVertexArray(0);
        self.vbo_bound = false;
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        self.ebo_bound = false;
    }

    unsafe fn bind_impl(&mut self) {
        gl::BindVertexArray(self.vertex_array_object);
        self.vbo_bound = true;
        if let Some(ebo) = self.element_buffer_object {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            self.ebo_bound = true;
        }
    }
}

impl ModelBuilder {
    pub fn new(
        program: Program,
        usage: Usage,
        position_attributes: VertexAttribute,
    ) -> Result<Self> {
        let num_values = gl::types::GLint::try_from(position_attributes.values.len()).unwrap();
        if num_values % position_attributes.component_size != 0 {
            return Err(format_err!(
                "number of values for color attribute should be divisible by component size"
            ));
        }
        let stride = gl::types::GLint::try_from(mem::size_of::<f32>()).unwrap()
            * position_attributes.component_size;

        Ok(Self {
            program,
            usage,
            position_attributes,
            stride,
            indices: None,
            vbo_num_elements: num_values,
            color_attributes: None,
        })
    }

    pub fn indices(mut self, indices: Vec<u32>) -> Result<Self> {
        let Some(max_index) = indices.iter().max().copied() else {
            return Ok(self);
        };

        let num_vertices = gl::types::GLsizei::try_from(self.position_attributes.values.len())
            .unwrap()
            / self.position_attributes.component_size;

        if max_index > u32::try_from(num_vertices - 1).unwrap() {
            return Err(format_err!("index value exceeds number of vertices"));
        }

        self.indices = Some(indices);
        Ok(self)
    }

    pub fn color_attributes(mut self, vertices: VertexAttribute) -> Result<Self> {
        let num_values = gl::types::GLsizei::try_from(vertices.values.len()).unwrap();
        if num_values % vertices.component_size != 0 {
            return Err(format_err!(
                "number of values for color attribute should be divisible by component size"
            ));
        }
        self.vbo_num_elements += num_values;
        self.stride +=
            gl::types::GLsizei::try_from(mem::size_of::<f32>()).unwrap() * vertices.component_size;
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

        let pos_component_size = usize::try_from(self.position_attributes.component_size).unwrap();
        let num_vertices = self.position_attributes.values.len() / pos_component_size;

        let mut attribute_data_iters =
            vec![self.position_attributes.values.chunks(pos_component_size)];

        if let Some(color_attrs) = self.color_attributes.as_ref() {
            let col_component_size = usize::try_from(color_attrs.component_size).unwrap();
            if color_attrs.values.len() / col_component_size != num_vertices {
                return Err(format_err!(
                    "number of color vertices should match number of position vertices"
                ));
            }
            attribute_data_iters.push(color_attrs.values.chunks(col_component_size));
        }

        let vbo_num_elements = usize::try_from(self.vbo_num_elements).unwrap();
        let mut buffer = Vec::with_capacity(vbo_num_elements);

        for i in (0..attribute_data_iters.len()).cycle() {
            let Some(chunk) = attribute_data_iters.get_mut(i).and_then(|a| a.next()) else {
                break;
            };
            buffer.extend_from_slice(chunk);
        }

        gl::BufferData(
            gl::ARRAY_BUFFER,
            try_into!(mem::size_of_val(buffer.as_slice())),
            buffer.as_ptr() as *const c_void,
            self.usage.into(),
        );

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let byte_offset = mem::size_of::<f32>() * pos_component_size;

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
            *component_size,
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
                *component_size,
                gl::FLOAT,
                as_gl_bool(*normalized),
                self.stride,
                byte_offset as *const c_void,
            );
            gl::EnableVertexAttribArray(col_attr_loc);
            // For when we texture coordinates are supported
            // byte_offset += mem::size_of::<f32>() * component_size;
        }

        /*
         * Element array buffer
         */
        let mut num_indices = 0;
        let element_buffer_object = self.indices.as_ref().map(|indices| {
            let mut ebo = 0;
            num_indices = indices.len();
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                try_into!(mem::size_of_val(indices.as_slice())),
                indices.as_ptr() as *const c_void,
                self.usage.into(),
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            ebo
        });

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        Ok(Model {
            program: self.program,
            vertex_array_object: vao,
            num_vertices: try_into!(num_vertices),
            num_indices: try_into!(num_indices),
            element_buffer_object,
            program_active: false,
            vbo_bound: false,
            ebo_bound: false,
        })
    }
}
