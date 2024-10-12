use crate::glutils::{
    model::{primitives::Primitive, usage::Usage, ModelBuilder, VertexAttribute},
    shader::{program, Shader, ShaderType},
};
use anyhow::{format_err, Result};
use glfw::{
    self, Context, GlfwReceiver, Key, Modifiers, OpenGlProfileHint, PWindow, WindowEvent,
    WindowHint, WindowMode,
};
use std::path::PathBuf;

const TRIANGLE_POS: [f32; 9] = [0.5, -0.5, 0.0, -0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

const TRIANGLE_COL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

pub fn run() -> Result<()> {
    let mut glfw_obj = glfw::init_no_callbacks()?;
    glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

    #[cfg(target_os = "macos")]
    glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events_rx) = glfw_obj
        .create_window(800, 600, "ShadersTriangle", WindowMode::Windowed)
        .ok_or_else(|| format_err!("failed to initialize window"))?;

    gl::load_with(|sym| window.get_proc_address(sym));
    glfw_obj.make_context_current(Some(&window));

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    let shader_path = PathBuf::new().join("shaders").join("_1_getting_started");

    let vertex_shader_src = shader_path.join("_3_shaders_triangle_vertex_shader.glsl");
    let fragment_shader_src = shader_path.join("_3_shaders_triangle_fragment_shader.glsl");

    let vs = Shader::new(vertex_shader_src, ShaderType::Vertex)?;
    let fs = Shader::new(fragment_shader_src, ShaderType::Fragment)?;

    let program = program::Linker::new()
        .attach_shader(vs)
        .attach_shader(fs)
        .link()?;

    let model = ModelBuilder::<9, 9>::new(
        program,
        Usage::Static,
        VertexAttribute::new("aPos", TRIANGLE_POS, 3, false),
    )?
    .color_attributes(VertexAttribute::new("aCol", TRIANGLE_COL, 3, false))?
    .build()?;

    model.use_program();
    model.bind();

    while !window.should_close() {
        handle_events(&events_rx, &mut window);
        clear_color(0.2, 0.2, 0.2, 0.0);
        model.draw_arrays(Primitive::Triangles);
        window.swap_buffers();
        glfw_obj.poll_events();
    }

    Ok(())
}

fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        clear_color_impl(r, g, b, a);
    }
}

unsafe fn clear_color_impl(r: f32, g: f32, b: f32, a: f32) {
    gl::ClearColor(r, g, b, a);
    gl::Clear(gl::COLOR_BUFFER_BIT);
}

fn handle_events(events_rx: &GlfwReceiver<(f64, WindowEvent)>, window: &mut PWindow) {
    unsafe {
        handle_events_impl(events_rx, window);
    }
}

unsafe fn handle_events_impl(events_rx: &GlfwReceiver<(f64, WindowEvent)>, window: &mut PWindow) {
    let Some((_, event)) = events_rx.receive() else {
        return;
    };

    match event {
        WindowEvent::Key(key, _, _, modifier) if modifier == Modifiers::Super && key == Key::W => {
            window.set_should_close(true);
        }
        WindowEvent::FramebufferSize(width, height) => gl::Viewport(0, 0, width, height),
        _ => (),
    }
}
