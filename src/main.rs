use red::tile_glyph::TileGlyphBuffer;
use red::BLACK;
use red::WHITE;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use red::buffer::Buffer;
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

    let mut glyph_buf = TileGlyphBuffer::new();

    glyph_buf.gl_init();
    glyph_buf.load_texture_atlas("charmap-oldschool_white.png");
    glyph_buf.compile_shaders("shaders/tile_glyph.vert", "shaders/tile_glyph.frag")?;

    let mut buffer = if let Some(filepath) = std::env::args().skip(1).next() {
        Buffer::from_filepath(filepath).map_err(|e| e.to_string())?
    } else {
        Buffer::new()
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
                        Keycode::F2 => match buffer.save() {
                            Ok(_) => println!("saved file!"),
                            Err(err) => eprintln!("{}", err),
                        },
                        Keycode::Backspace => buffer.backspace(),
                        Keycode::Delete => buffer.delete(),
                        Keycode::Left => buffer.move_left(),
                        Keycode::Right => buffer.move_right(),
                        Keycode::Up => buffer.move_up(),
                        Keycode::Down => buffer.move_down(),
                        Keycode::Return => buffer.newline(),
                        _ => {}
                    },
                    _ => {}
                },
                Event::TextInput { text, .. } => buffer.insert_text(&text),
                _ => {}
            }
        }

        let cursor_pos = v2!(
            buffer.cursor.x as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE,
            -(buffer.cursor.y as isize) as f32 * FONT_CHAR_HEIGHT as f32 * FONT_SCALE,
        );

        camera_vel = (cursor_pos - camera_pos) * v2s!(2.0);
        camera_pos += camera_vel * v2s!(DELTA_TIME);

        unsafe {
            let (width, height) = window.size();
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Uniform2f(
                glyph_buf.resolution_uniform,
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
            );
            gl::Uniform2f(glyph_buf.camera_uniform, camera_pos.x, camera_pos.y);

            gl::Uniform1f(glyph_buf.time_uniform, timer.ticks() as f32 / 1000.0);

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        let lines_per_screen = SCREEN_HEIGHT as f32 / (FONT_CHAR_HEIGHT as f32 * FONT_SCALE);
        let start_idx = (buffer.cursor.y)
            .checked_sub(lines_per_screen as usize)
            .unwrap_or(0);
        let end_idx = std::cmp::min(
            start_idx + (lines_per_screen * 2.0) as usize,
            buffer.lines.len(),
        );

        glyph_buf.clear();
        for i in start_idx..end_idx {
            glyph_buf.render_line(&buffer.lines[i].chars, v2!(0, -(i as i32)), WHITE, BLACK);
        }

        glyph_buf.gl_render_cursor(&buffer);
        glyph_buf.sync();
        glyph_buf.draw();

        window.gl_swap_window();

        let duration = timer.ticks() - start;
        let delta_time_ms = 1000 / FPS;
        if duration < delta_time_ms {
            timer.delay(delta_time_ms - duration);
        }
    }

    Ok(())
}
