use anyhow::{format_err, Result};
use glfw::{
    self, Context, GlfwReceiver, Key, Modifiers, OpenGlProfileHint, PWindow, WindowEvent,
    WindowHint, WindowMode,
};
use std::path::PathBuf;

use crate::glutils::{
    model::{ModelBuilder, Usage, Vertices},
    shader::{self, ShaderType},
};

const TRIANGLE: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

pub fn run() -> Result<()> {
    let mut glfw_obj = glfw::init_no_callbacks()?;
    glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

    #[cfg(target_os = "macos")]
    glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events_rx) = glfw_obj
        .create_window(800, 600, "Shaders", WindowMode::Windowed)
        .ok_or(format_err!("failed to init window"))?;

    gl::load_with(|sym| window.get_proc_address(sym));

    glfw_obj.make_context_current(Some(&window));

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    let shaders_dir = PathBuf::new().join("shaders").join("_1_getting_started");

    let vs_src = shaders_dir.join("_3_shaders_vertex_shader.glsl");
    let fs_src = shaders_dir.join("_3_shaders_fragment_shader.glsl");

    let vertex_shader = shader::Shader::new(vs_src, ShaderType::Vertex)?;
    let fragment_shader = shader::Shader::new(fs_src, ShaderType::Fragment)?;

    let program = shader::program::Linker::new()
        .attach_shader(vertex_shader)
        .attach_shader(fragment_shader)
        .link()?;

    let model = ModelBuilder::<9, 0>::new(
        program,
        Usage::Static,
        Vertices::new("aPos", TRIANGLE, 3, false),
    )
    .build()?;

    let our_color = program.get_uniform_loc("ourColor")?;

    program.use_program();
    model.bind();

    while !window.should_close() {
        process_events(&events_rx, &mut window);
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let time_value = glfw_obj.get_time();
            let green_value = time_value.sin() / 2.0 + 0.5;
            gl::Uniform4f(our_color, 0.0, green_value as f32, 0.0, 1.0);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        window.swap_buffers();
        glfw_obj.poll_events();
    }

    Ok(())
}

fn process_events(events_rx: &GlfwReceiver<(f64, WindowEvent)>, window: &mut PWindow) {
    let Some((_, event)) = events_rx.receive() else {
        return;
    };

    match event {
        WindowEvent::Key(key, _, _, modifier) => match (modifier, key) {
            (Modifiers::Super, Key::W) | (_, Key::Escape) => window.set_should_close(true),
            _ => (),
        },
        WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
        },
        _ => (),
    }
}
