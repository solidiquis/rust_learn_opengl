use crate::glutils::{
    model::{primitives::Primitive, usage::Usage, ModelBuilder, VertexAttribute},
    shader::{program::Linker, Shader, ShaderType},
};
use anyhow::{format_err, Result};
use glfw::{
    Context, GlfwReceiver, Key, Modifiers, OpenGlProfileHint, PWindow, WindowEvent, WindowHint,
    WindowMode,
};
use std::path::PathBuf;

const SQUARE: [f32; 12] = [
    0.5, 0.5, 0.0, // top right
    0.5, -0.5, 0.0, // bottom right
    -0.5, -0.5, 0.0, // bottom left
    -0.5, 0.5, 0.0, // top left
];

const INDICES: [u32; 6] = [
    0, 1, 3, // first triangle
    1, 2, 3, // second triangle
];

pub fn run() -> Result<()> {
    let mut glfw_obj = glfw::init_no_callbacks()?;
    glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

    #[cfg(target_os = "macos")]
    glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events_rx) = glfw_obj
        .create_window(800, 600, "HelloSquare", WindowMode::Windowed)
        .ok_or(format_err!("failed to create window"))?;

    glfw_obj.make_context_current(Some(&window));

    gl::load_with(|symbol| window.get_proc_address(symbol));

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    let shaders_dir = PathBuf::new().join("shaders").join("_1_getting_started");

    let vs_src = shaders_dir.join("_2_hello_square_vertex_shader.glsl");
    let fs_src = shaders_dir.join("_2_hello_square_fragment_shader.glsl");

    let vs = Shader::new(vs_src, ShaderType::Vertex)?;
    let fs = Shader::new(fs_src, ShaderType::Fragment)?;

    let program = Linker::new().attach_shader(vs).attach_shader(fs).link()?;

    let mut model = ModelBuilder::new(
        program,
        Usage::Static,
        VertexAttribute::new("aPos", SQUARE.to_vec(), 3, false),
    )?
    .indices(INDICES.to_vec())?
    .build()?;

    model.use_program();
    model.bind();

    while !window.should_close() {
        handle_event(&events_rx, &mut window);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        model.try_draw_arrays(Primitive::Triangles)?;

        window.swap_buffers();
        glfw_obj.poll_events();
    }

    Ok(())
}

fn handle_event(events_rx: &GlfwReceiver<(f64, WindowEvent)>, window: &mut PWindow) {
    let Some((_, event)) = events_rx.receive() else {
        return;
    };

    match event {
        WindowEvent::Key(key, _, _, modifier) if modifier == Modifiers::Super && key == Key::W => {
            window.set_should_close(true);
        }
        WindowEvent::FramebufferSize(width, height) => unsafe { gl::Viewport(0, 0, width, height) },
        _ => (),
    }
}
