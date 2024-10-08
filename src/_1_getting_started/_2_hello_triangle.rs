use anyhow::{format_err, Result};
use glfw::{
    Action, Context, GlfwReceiver, Key, OpenGlProfileHint, PWindow, WindowEvent, WindowHint,
    WindowMode,
};

const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;

void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main()
{
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}
"#;

const TRIANGLE: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

pub fn run() -> Result<()> {
    let mut glfw_obj = glfw::init_no_callbacks()?;
    glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

    #[cfg(target_os = "macos")]
    glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events_rx) = glfw_obj
        .create_window(800, 600, "HelloTriangle", WindowMode::Windowed)
        .ok_or(format_err!("failed to create window"))?;

    glfw_obj.make_context_current(Some(&window));

    gl::load_with(|symbol| window.get_proc_address(symbol));

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    let (vao, vbo) = unsafe {
        let mut vao = 0;
        let mut vbo = 0;

        // Create the VAO
        gl::GenVertexArrays(1, &mut vao);
        // Create the VBO
        gl::GenBuffers(1, &mut vbo);

        // Bind the VAO and VBO to the GL context, respectively and make them the active objects to
        // operate on.
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Copy vertex data from CPU to GPU
        let vertices_ptr = TRIANGLE.as_ptr() as *const std::ffi::c_void;
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (std::mem::size_of::<f32>() * TRIANGLE.len()) as isize,
            vertices_ptr,
            gl::STATIC_DRAW,
        );

        // Modify the state of the currently bound VAO, informing OpenGL how to interpret what is
        // currently bound to `gl::ARRAY_BUFFER`.
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<f32>() as i32,
            std::ptr::null::<std::ffi::c_void>(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        (vao, vbo)
    };

    let triangle_vertex_shader = std::ffi::CString::new(VERTEX_SHADER)?;

    let vertex_shader = unsafe {
        // Create a compile the shader
        let shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            shader,
            1,
            &triangle_vertex_shader.as_ptr(),
            std::ptr::null::<i32>(),
        );
        gl::CompileShader(shader);

        // Check compilation status
        let mut success = std::mem::zeroed::<gl::types::GLint>();
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        // Check for compile error
        if success == 0 {
            let mut log: [u8; 1024] = [0; 1024];
            gl::GetShaderInfoLog(
                shader,
                std::mem::size_of_val(&log) as i32,
                std::ptr::null_mut::<i32>(),
                log.as_mut_ptr() as *mut i8,
            );
            let reason = String::from_utf8_lossy(&log);
            return Err(anyhow::format_err!(
                "failed to compile vertex shader: {reason}"
            ));
        }
        shader
    };

    let triangle_fragment_shader = std::ffi::CString::new(FRAGMENT_SHADER)?;

    let fragment_shader = unsafe {
        let shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            shader,
            1,
            &triangle_fragment_shader.as_ptr(),
            std::ptr::null::<i32>(),
        );
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut log: [u8; 1024] = [0; 1024];
            gl::GetShaderInfoLog(
                shader,
                std::mem::size_of_val(&log) as i32,
                std::ptr::null_mut::<i32>(),
                log.as_mut_ptr() as *mut i8,
            );
            let reason = String::from_utf8_lossy(&log);
            return Err(anyhow::format_err!(
                "failed to compile fragment shader: {reason}"
            ));
        }
        shader
    };

    // Creating our GPU program
    let shader_program = unsafe {
        // Create the program and attach out shaders to it
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        // Link our shaders together to make one executable shader program.
        gl::LinkProgram(program);

        // Get link status
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut log: [u8; 1024] = [0; 1024];
            gl::GetProgramInfoLog(
                program,
                std::mem::size_of_val(&log) as i32,
                std::ptr::null_mut::<i32>(),
                log.as_mut_ptr() as *mut i8,
            );
            let reason = String::from_utf8_lossy(&log);
            return Err(anyhow::format_err!("failed to link program: {reason}"));
        }
        program
    };

    // Delete our shaders since we now have successfully generated our program
    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    unsafe {
        gl::UseProgram(shader_program);
        gl::BindVertexArray(vao);
    }

    while !window.should_close() {
        handle_event(&events_rx, &mut window);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawArrays(gl::TRIANGLES, 0, 3)
        }

        window.swap_buffers();
        glfw_obj.poll_events();
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(shader_program);
    }

    Ok(())
}

fn handle_event(rx: &GlfwReceiver<(f64, WindowEvent)>, window: &mut PWindow) {
    let Some((_, event)) = rx.receive() else {
        return;
    };

    match event {
        WindowEvent::Key(key, _, action, _) if key == Key::Escape && action == Action::Press => {
            window.set_should_close(true);
        }
        WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
        },
        _ => (),
    }
}
