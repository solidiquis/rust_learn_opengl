# Hello Window

- Chapter: [Hello Window](https://learnopengl.com/Getting-started/Hello-Window)
- Module: [_1_hello_window.rs](../../src/_1_getting_started/_1_hello_window.rs)

## Summary

We start this chapter by initializing a `GLFW` object. `GLFW` stands for "Graphics Library Framework"
which is an old C library that will allow us to configure our OpenGL session as well as get
a handle to our window.

To configure our OpenGL session we use window hints. This allows us to specify things like
which OpenGL version we want to use. MacOS requires an additional setting. The whole setup looks like this:

```rust
let mut glfw_obj = glfw::init_no_callbacks()?;
glfw_obj.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
glfw_obj.window_hint(WindowHint::ContextVersion(3, 3));

#[cfg(target_os = "macos")]
glfw_obj.window_hint(WindowHint::OpenGlForwardCompat(true));
```

Once we have our `GLFW` object we can grab a handle to our window and set an OpenGL context
for the current thread. OpenGL context's are thread specific. An OpenGL context is basically the OpenGL state for our
session.

```rust
let (mut window, events_rx) = glfw_obj
    .create_window(800, 600, "HelloWindow", WindowMode::Windowed)
    .ok_or(format_err!("failed to create window"))?;

glfw_obj.make_context_current(Some(&window));
```

To use symbols from the OpenGL API, we need to load them in. They can be loaded
in as needed but we'll just grab all of them:

```rust
gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
```

Next we specify what types of events we want to poll for:

```rust
// Poll for key press events
window.set_key_polling(true);

// Poll for window resize or frame buffer resize
window.set_framebuffer_size_polling(true);
```

The following is the event handler function. It will simply reset the view port whenever
window is resized as well as close the window when `ESC` is received.

```rust
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
```

Now we loop and handle the event:

```rust
while !window.should_close() {
    // pull events off the events channel
    handle_event(&mut window, &events_rx);

    // render step
    unsafe {
        // set color used to clear back buffer
        gl::ClearColor(0.2, 0.2, 0.2, 0.0);
        // actualy clear the back buffer
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    // swap the back and front buffer
    window.swap_buffers();

    // poll for new events and populate events channel with new events
    glfw_obj.poll_events();
}
```

## Important Concepts

### Back and Front Buffer

By default the `GLFW` object has two frame-buffers: front and back. The front-buffer
is what's displayed on the window while the back frame-buffer is what gets written to.
Writing to the back buffer is what's actually considered "rendering", however, rendering
does not necessarily imply the actual displaying to the screen. After the rendering step,
the `glfwSwapBuffers` is what actually causes the bytes the contents of the back buffer onto the screen.
When this occurs, the back-buffer becomes the new front-buffer, and the original front-buffer
becomes the new back-buffer, ready to be rendered to.

### Frame-buffer

A frame-buffer is essentially a 2D grid data structure, where each cell in the grid is a pixel
object. Each pixel has its own properties such as color, depth, and stencil. When we call
`gl::Clear(gl::COLOR_BUFFER_BIT)`, we are clearing the color properties of each pixel of the back-buffer
with whatever's set by `gl::ClearColor`.
