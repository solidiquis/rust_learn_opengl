use anyhow::{format_err, Result};
use glfw::{
    self, Context, GlfwReceiver, Key, OpenGlProfileHint, PWindow, WindowEvent, WindowHint,
    WindowMode,
};

pub fn run() -> Result<()> {
    let mut glfw_obj = glfw::init_no_callbacks()?;
    glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

    #[cfg(target_os = "macos")]
    glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events_rx) = glfw_obj
        .create_window(800, 600, "HelloWindow", WindowMode::Windowed)
        .ok_or(format_err!("failed to create window"))?;

    glfw_obj.make_context_current(Some(&window));

    gl::load_with(|symbol| window.get_proc_address(symbol));

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    while !window.should_close() {
        handle_event(&mut window, &events_rx);
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        window.swap_buffers();
        glfw_obj.poll_events();
    }
    Ok(())
}

fn handle_event(window: &mut PWindow, events_rx: &GlfwReceiver<(f64, WindowEvent)>) {
    let Some((_, event)) = events_rx.receive() else {
        return;
    };

    match event {
        WindowEvent::Key(Key::Escape, _, _, _) => window.set_should_close(true),
        WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
        },
        _ => (),
    }
}
