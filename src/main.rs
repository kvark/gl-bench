//! Full-screen pixel rate
//! Based on a glutin sample

extern crate gl;
extern crate glutin;

use gl::types::*;
use std::{ptr, str};

// Shader sources
static VS_SRC: &'static str = "
    #version 150 core

    void main() {
        switch (gl_VertexID) {
            case 0: gl_Position = vec4(-1.0, -3.0, 0.0, 1.0); break;
            case 1: gl_Position = vec4(3.0, 1.0, 0.0, 1.0);   break;
            case 2: gl_Position = vec4(-1.0, 1.0, 0.0, 1.0);  break;
            default: gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }"
;

static FS_SRC: &'static str = "
    #version 150
    out vec4 o_Color;

    void main() {
        o_Color = vec4(1.0, 1.0, 1.0, 1.0);
    }"
;

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    use std::ffi::CString;
    unsafe {
        let shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let cs = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &cs.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        assert_eq!(status, 1);
        shader
    }
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        assert_eq!(status, 1);
        program
    }
}

fn main() {
    use glutin::GlContext;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_fullscreen(Some(events_loop.get_primary_monitor()));
    let context = glutin::ContextBuilder::new()
        .with_vsync(false);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current() }.unwrap();

    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);
    let mut queries = [0; 20];
    let mut vao = 0;
    let mut cur_query = 0;
    let mut query_cycles = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenQueries(queries.len() as _, queries.as_mut_ptr());
        gl::BindVertexArray(vao);
        gl::UseProgram(program);
        assert_eq!(gl::GetError(), 0);
    }

    let mut sum_times = 0usize;
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            use glutin::{Event, KeyboardInput, WindowEvent, VirtualKeyCode as Key};

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(Key::Escape), .. }, .. } |
                    WindowEvent::Closed => running = false,
                    _ => (),
                }
            }
        });

        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if query_cycles != 0 {
                let mut result = 0;
                gl::GetQueryObjectuiv(queries[cur_query], gl::QUERY_RESULT, &mut result);
                sum_times += result as usize;
            }
            gl::BeginQuery(gl::TIME_ELAPSED, queries[cur_query]);

            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::EndQuery(gl::TIME_ELAPSED);
            cur_query += 1;
            if cur_query == queries.len() {
                query_cycles += 1;
                cur_query = 0;
            }

            debug_assert_eq!(gl::GetError(), 0);
        }

        gl_window.swap_buffers().unwrap();
    }

    let (width, height) = gl_window.get_inner_size().unwrap();
    let total_count = cur_query + query_cycles * queries.len();
    let average_time = sum_times / total_count;
    println!("Avg draw time: {:.2} ms for {}x{} resolution",
        average_time as f32 / 1.0e6, width, height);

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteVertexArrays(1, &vao);
    }
}
