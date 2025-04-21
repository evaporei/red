use red::tile_glyph::TileGlyphBuffer;
use red::BLACK;
use red::WHITE;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use red::editor::Editor;
use red::shaders;
use red::vector::Vector2;
use red::{v2, v2s};

// const SCREEN_WIDTH: u32 = 800;
// const SCREEN_HEIGHT: u32 = 600;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const FPS: u32 = 60;
const DELTA_TIME: f32 = 1.0 / FPS as f32;

// const FONT_SCALE: f32 = 5.0;
const FONT_SCALE: f32 = 3.0;
const FONT_WIDTH: usize = 128;
const FONT_HEIGHT: usize = 64;

const FONT_COLS: usize = 18;
const FONT_ROWS: usize = 7;

const FONT_CHAR_WIDTH: usize = FONT_WIDTH / FONT_COLS;
const FONT_CHAR_HEIGHT: usize = FONT_HEIGHT / FONT_ROWS;

#[allow(unused)]
fn gl_check_errors() {
    let mut err = unsafe { gl::GetError() };
    while err != gl::NO_ERROR {
        match err {
            gl::INVALID_ENUM => {
                eprintln!("enumeration parameter is not a legal enumeration for that function");
            }
            gl::INVALID_VALUE => {
                eprintln!("value parameter is not a legal value for that function");
            }
            gl::INVALID_OPERATION => {
                eprintln!("the set of state for a command is not legal for the parameters given to that command");
            }
            gl::STACK_OVERFLOW => {
                eprintln!("stack pushing operation cannot be done because it would overflow the limit of that stack's size");
            }
            gl::STACK_UNDERFLOW => {
                eprintln!("stack popping operation cannot be done because the stack is already at its lowest point");
            }
            gl::OUT_OF_MEMORY => {
                eprintln!("performing an operation that can allocate memory, and the memory cannot be allocated");
            }
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                eprintln!("doing anything that would attempt to read from or write/render to a framebuffer that is not complete");
            }
            gl::CONTEXT_LOST => {
                eprintln!("OpenGL context has been lost, due to a graphics card reset");
            }
            _ => {}
        };
        err = unsafe { gl::GetError() };
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    // let (gl_ver_maj, gl_ver_min) = gl_attr.context_version();
    // println!("opengl version: {}.{}", gl_ver_maj, gl_ver_min);

    let window = video_subsystem
        .window("red", SCREEN_WIDTH, SCREEN_HEIGHT)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let _gl_context = window.gl_create_context()?;
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut tile_glyph_buf = TileGlyphBuffer::new();

    tile_glyph_buf.gl_init();
    tile_glyph_buf.load_texture_atlas("charmap-oldschool_white.png");

    let program = shaders::load("shaders/font.vert", "shaders/font.frag")?;
    unsafe {
        gl::UseProgram(program);
    }

    let time_uniform;
    let resolution_uniform;
    let camera_uniform;
    unsafe {
        time_uniform = gl::GetUniformLocation(program, c"time".as_ptr());
        if time_uniform == -1 {
            eprintln!("time uniform not found");
        }

        resolution_uniform = gl::GetUniformLocation(program, c"resolution".as_ptr());
        if resolution_uniform == -1 {
            eprintln!("resolution uniform not found");
        }

        let scale_uniform = gl::GetUniformLocation(program, c"scale".as_ptr());
        if scale_uniform == -1 {
            eprintln!("scale uniform not found");
        }
        gl::Uniform1f(scale_uniform, FONT_SCALE);

        camera_uniform = gl::GetUniformLocation(program, c"camera".as_ptr());
        if camera_uniform == -1 {
            eprintln!("camera uniform not found");
        }
    };

    let mut editor = if let Some(filepath) = std::env::args().skip(1).next() {
        Editor::from_filepath(filepath).map_err(|e| e.to_string())?
    } else {
        Editor::new()
    };

    let timer = sdl_context.timer()?;

    let mut camera_pos = v2s!(0.0);
    let mut camera_vel;

    let mut event_pump = sdl_context.event_pump()?;
    let mut quit = false;
    while !quit {
        let start = timer.ticks();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => quit = true,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(key) => match key {
                        Keycode::F2 => match editor.save() {
                            Ok(_) => println!("saved file!"),
                            Err(err) => eprintln!("{}", err),
                        },
                        Keycode::Backspace => editor.backspace(),
                        Keycode::Delete => editor.delete(),
                        Keycode::Left => editor.move_left(),
                        Keycode::Right => editor.move_right(),
                        Keycode::Up => editor.move_up(),
                        Keycode::Down => editor.move_down(),
                        Keycode::Return => editor.newline(),
                        _ => {}
                    },
                    _ => {}
                },
                Event::TextInput { text, .. } => editor.insert_text(&text),
                _ => {}
            }
        }

        let cursor_pos = v2!(
            editor.cursor.x as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE,
            -(editor.cursor.y as isize) as f32 * FONT_CHAR_HEIGHT as f32 * FONT_SCALE,
        );

        camera_vel = (cursor_pos - camera_pos) * v2s!(2.0);
        camera_pos += camera_vel * v2s!(DELTA_TIME);

        tile_glyph_buf.clear();
        for (i, line) in editor.lines.iter().enumerate() {
            tile_glyph_buf.render_line(&line.chars, v2!(0, -(i as i32)), WHITE, BLACK);
        }
        tile_glyph_buf.sync();

        unsafe {
            let (width, height) = window.size();
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Uniform2f(
                resolution_uniform,
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
            );
            gl::Uniform2f(camera_uniform, camera_pos.x, camera_pos.y);

            gl::Uniform1f(time_uniform, timer.ticks() as f32 / 1000.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, tile_glyph_buf.len() as i32);
            // gl_check_errors();
        }

        tile_glyph_buf.clear();
        tile_glyph_buf.gl_render_cursor(&editor);
        tile_glyph_buf.sync();

        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, tile_glyph_buf.len() as i32);
        }

        window.gl_swap_window();

        let duration = timer.ticks() - start;
        let delta_time_ms = 1000 / FPS;
        if duration < delta_time_ms {
            timer.delay(delta_time_ms - duration);
        }
    }

    Ok(())
}
