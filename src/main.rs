use red::tile_glyph::TileGlyphBuffer;
use red::BLACK;
use red::WHITE;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use red::editor::Editor;
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

    let mut tile_glyph_buf = TileGlyphBuffer::new();

    tile_glyph_buf.gl_init();
    tile_glyph_buf.load_texture_atlas("charmap-oldschool_white.png");
    tile_glyph_buf.compile_shaders("shaders/tile_glyph.vert", "shaders/tile_glyph.frag")?;

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

        unsafe {
            let (width, height) = window.size();
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Uniform2f(
                tile_glyph_buf.resolution_uniform,
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
            );
            gl::Uniform2f(tile_glyph_buf.camera_uniform, camera_pos.x, camera_pos.y);

            gl::Uniform1f(tile_glyph_buf.time_uniform, timer.ticks() as f32 / 1000.0);

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        tile_glyph_buf.clear();
        for (i, line) in editor.lines.iter().enumerate() {
            tile_glyph_buf.render_line(&line.chars, v2!(0, -(i as i32)), WHITE, BLACK);
        }
        tile_glyph_buf.gl_render_cursor(&editor);
        tile_glyph_buf.sync();
        tile_glyph_buf.draw();

        window.gl_swap_window();

        let duration = timer.ticks() - start;
        let delta_time_ms = 1000 / FPS;
        if duration < delta_time_ms {
            timer.delay(delta_time_ms - duration);
        }
    }

    Ok(())
}
