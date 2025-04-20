use gl::types::GLuint;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use stb_image::stb_image::{stbi_load, stbi_set_flip_vertically_on_load};
use std::ffi::c_void;
use std::ffi::CString;
use std::mem::offset_of;

use red::editor::Editor;
use red::shaders;
use red::small_array::SmallArray;
use red::vector::{Vector2, Vector4};

// const SCREEN_WIDTH: u32 = 800;
// const SCREEN_HEIGHT: u32 = 600;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const FPS: u32 = 60;
const DELTA_TIME: f32 = 1.0 / FPS as f32;

const FONT_SCALE: f32 = 5.0;
// const FONT_SCALE: f32 = 3.0;
const FONT_WIDTH: usize = 128;
const FONT_HEIGHT: usize = 64;

const FONT_COLS: usize = 18;
const FONT_ROWS: usize = 7;

const FONT_CHAR_WIDTH: usize = FONT_WIDTH / FONT_COLS;
const FONT_CHAR_HEIGHT: usize = FONT_HEIGHT / FONT_ROWS;

fn load_img(file_path: &str) -> (Vec<u8>, i32, i32) {
    let c_path = CString::new(file_path).unwrap();

    let mut width = 0;
    let mut height = 0;
    let mut channels = 3;
    let stbi_rgb_alpha = 4;

    let pixels = unsafe {
        stbi_load(
            c_path.as_ptr(),
            &mut width,
            &mut height,
            &mut channels,
            stbi_rgb_alpha,
        )
    };

    if pixels.is_null() {
        panic!("image is null after load");
    }

    (
        unsafe { std::slice::from_raw_parts(pixels, (width * height * 4) as usize) }.to_vec(),
        width,
        height,
    )
}

fn surface_from_file(pixels: &mut Vec<u8>, width: i32, height: i32) -> Result<Surface, String> {
    Surface::from_data(
        pixels,
        width as u32,
        height as u32,
        (4 * width) as u32,
        PixelFormatEnum::RGBA32,
    )
}

const ASCII_DISPLAY_LOW: u8 = 32;
const ASCII_DISPLAY_HIGH: u8 = 126;

struct Font<'a> {
    spritesheet: Texture<'a>,
    glyph_table: [Rect; (ASCII_DISPLAY_HIGH - ASCII_DISPLAY_LOW + 1) as usize],
}

impl<'a> Font<'a> {
    fn new(spritesheet: Texture<'a>) -> Self {
        let glyph_table = std::array::from_fn(|i| {
            let col = i % FONT_COLS;
            let row = i / FONT_COLS;

            Rect::new(
                col as i32 * FONT_CHAR_WIDTH as i32,
                row as i32 * FONT_CHAR_HEIGHT as i32,
                FONT_CHAR_WIDTH as u32,
                FONT_CHAR_HEIGHT as u32,
            )
        });

        Self {
            spritesheet,
            glyph_table,
        }
    }
}

fn render_char(
    canvas: &mut WindowCanvas,
    font: &Font,
    c: u8,
    pos: Vector2<f32>,
    scale: f32,
) -> Result<(), String> {
    assert!(c >= ASCII_DISPLAY_LOW);
    assert!(c <= ASCII_DISPLAY_HIGH);
    let idx = (c - b' ') as usize;

    // view into the texture
    let src = font.glyph_table[idx];

    // where in the screen/window
    let dst = Rect::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        (FONT_CHAR_WIDTH as f32 * scale).floor() as u32,
        (FONT_CHAR_HEIGHT as f32 * scale).floor() as u32,
    );

    canvas.copy(&font.spritesheet, src, dst)
}

fn set_texture_color(texture: &mut Texture<'_>, color: Color) {
    texture.set_color_mod(color.r, color.g, color.b);
    texture.set_alpha_mod(color.a);
}

fn render_text(
    canvas: &mut WindowCanvas,
    font: &mut Font,
    editor: &Editor,
    camera_pos: Vector2<f32>,
    color: Color,
    scale: f32,
) -> Result<(), String> {
    set_texture_color(&mut font.spritesheet, color);

    let mut pen = Vector2::new(0.0, 0.0);
    for line in &editor.lines {
        for ch in line.chars.bytes() {
            render_char(canvas, font, ch, pen - camera_pos, scale)?;
            pen.x += FONT_CHAR_WIDTH as f32 * scale;
        }
        pen.x = 0.0;
        pen.y += FONT_CHAR_HEIGHT as f32 * scale;
    }
    Ok(())
}

fn render_cursor(
    canvas: &mut WindowCanvas,
    font: &mut Font,
    editor: &Editor,
    camera_pos: Vector2<f32>,
    pos: Vector2<f32>,
) -> Result<(), String> {
    let pos = pos - camera_pos;

    canvas.set_draw_color(Color::WHITE);
    canvas.fill_rect(Rect::new(
        (pos.x).floor() as i32,
        pos.y.floor() as i32,
        (FONT_CHAR_WIDTH as f32 * FONT_SCALE) as u32,
        (FONT_CHAR_HEIGHT as f32 * FONT_SCALE) as u32,
    ))?;

    set_texture_color(&mut font.spritesheet, Color::RGB(0, 0, 0));
    if editor.cursor.x < editor.lines[editor.cursor.y].chars.len() {
        render_char(
            canvas,
            font,
            editor.lines[editor.cursor.y]
                .chars
                .bytes()
                .nth(editor.cursor.x)
                .unwrap(),
            pos,
            FONT_SCALE,
        )?;
    }

    Ok(())
}

#[repr(C)]
struct Glyph {
    tile: Vector2<i32>,
    ch: i32,
    fg_color: Vector4<f32>,
    bg_color: Vector4<f32>,
}

struct GlAttrib {
    r#type: gl::types::GLenum,
    comps: i32,
    normalized: gl::types::GLboolean,
    stride: i32,
    offset: usize,
}

impl Glyph {
    const fn gl_attributes() -> [GlAttrib; 4] {
        let stride = size_of::<Glyph>() as i32;
        let normalized = gl::FALSE;
        [
            GlAttrib {
                r#type: gl::INT,
                comps: 2,
                normalized,
                stride,
                offset: offset_of!(Glyph, tile),
            },
            GlAttrib {
                r#type: gl::INT,
                comps: 1,
                normalized,
                stride,
                offset: offset_of!(Glyph, ch),
            },
            GlAttrib {
                r#type: gl::FLOAT,
                comps: 4,
                normalized,
                stride,
                offset: offset_of!(Glyph, fg_color),
            },
            GlAttrib {
                r#type: gl::FLOAT,
                comps: 4,
                normalized,
                stride,
                offset: offset_of!(Glyph, bg_color),
            },
        ]
    }
}

const GLYPH_BUFF_CAP: usize = 640 * 1024;
type GlyphBuffer = Vec<Glyph>;

fn gl_render_text(
    glyph_buffer: &mut GlyphBuffer,
    text: &str,
    tile: Vector2<i32>,
    fg_color: Vector4<f32>,
    bg_color: Vector4<f32>,
) {
    for (i, ch) in text.chars().enumerate() {
        let glyph = Glyph {
            tile: tile + Vector2::new(i as i32, 0),
            ch: ch as i32,
            fg_color,
            bg_color,
        };
        glyph_buffer.push(glyph);
    }
}

fn gl_render_cursor(glyph_buffer: &mut GlyphBuffer, editor: &Editor) {
    gl_render_text(
        glyph_buffer,
        &editor.char_at_cursor().unwrap_or(' ').to_string(),
        Vector2::new(editor.cursor.x as i32, -(editor.cursor.y as i32)),
        BLACK,
        WHITE,
    );
}

fn glyph_buffer_sync(glyph_buffer: &GlyphBuffer) {
    unsafe {
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            (glyph_buffer.len() * size_of::<Glyph>()) as isize,
            glyph_buffer.as_ptr() as *const c_void,
        );
    }
}

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

const BLACK: Vector4<f32> = Vector4::from_scalar(0.0);
const WHITE: Vector4<f32> = Vector4::from_scalar(1.0);
const YELLOW: Vector4<f32> = Vector4::new(1.0, 1.0, 0.0, 1.0);

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

    let mut vao: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

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

    let mut font_texture = 0;
    let (mut pixels, width, height) = load_img("charmap-oldschool_white.png");
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::GenTextures(1, &mut font_texture);
        gl::BindTexture(gl::TEXTURE_2D, font_texture);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width,
            height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut c_void,
        );
    }

    let mut vbo: GLuint = 0;
    let mut glyph_buffer = Vec::with_capacity(GLYPH_BUFF_CAP);

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of::<[Glyph; GLYPH_BUFF_CAP]>() as isize,
            glyph_buffer.as_ptr() as *const std::ffi::c_void,
            gl::DYNAMIC_DRAW,
        );
    }

    for (i, attrib) in Glyph::gl_attributes().into_iter().enumerate() {
        let index = i as u32;
        let offset = attrib.offset as *const usize as *const std::ffi::c_void;
        unsafe {
            gl::EnableVertexAttribArray(index);
            match attrib.r#type {
                gl::FLOAT => {
                    gl::VertexAttribPointer(
                        index,
                        attrib.comps,
                        attrib.r#type,
                        attrib.normalized,
                        attrib.stride,
                        offset,
                    );
                }
                gl::INT => {
                    gl::VertexAttribIPointer(
                        index,
                        attrib.comps,
                        attrib.r#type,
                        attrib.stride,
                        offset,
                    );
                }
                _ => unimplemented!("handle new gl attribute type"),
            }
            gl::VertexAttribDivisor(index, 1);
        }
    }

    let mut editor = if let Some(filepath) = std::env::args().skip(1).next() {
        Editor::from_filepath(filepath).map_err(|e| e.to_string())?
    } else {
        Editor::new()
    };

    let timer = sdl_context.timer()?;

    let mut camera_pos = Vector2::new(0.0, 0.0);
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

        let cursor_pos = Vector2::new(
            editor.cursor.x as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE,
            -(editor.cursor.y as isize) as f32 * FONT_CHAR_HEIGHT as f32 * FONT_SCALE,
        );

        camera_vel = (cursor_pos - camera_pos) * Vector2::from_scalar(2.0);
        camera_pos += camera_vel * Vector2::from_scalar(DELTA_TIME);

        glyph_buffer.clear();
        for (i, line) in editor.lines.iter().enumerate() {
            gl_render_text(
                &mut glyph_buffer,
                &line.chars,
                Vector2::new(0, -(i as i32)),
                YELLOW,
                BLACK,
            );
        }
        glyph_buffer_sync(&glyph_buffer);

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
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, glyph_buffer.len() as i32);
            // gl_check_errors();
        }

        glyph_buffer.clear();
        gl_render_cursor(&mut glyph_buffer, &editor);
        glyph_buffer_sync(&glyph_buffer);

        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, glyph_buffer.len() as i32);
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

fn main2() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("red", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let window_size = Vector2::new(window.size().0 as f32, window.size().1 as f32);

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let (mut pixels, width, height) = load_img("charmap-oldschool_white.png");
    let mut font_surface = surface_from_file(&mut pixels, width, height)?;
    font_surface.set_color_key(true, Color::RGBA(0, 0, 0, 0))?;
    let texture_creator = canvas.texture_creator();
    let font_texture = font_surface
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

    let mut font = Font::new(font_texture);

    let mut event_pump = sdl_context.event_pump()?;

    let mut editor = if let Some(filepath) = std::env::args().skip(1).next() {
        Editor::from_filepath(filepath).map_err(|e| e.to_string())?
    } else {
        Editor::new()
    };

    let mut camera_pos = Vector2::new(0.0, 0.0);
    let mut camera_vel;

    let timer = sdl_context.timer()?;

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

        let cursor_pos = Vector2::new(
            editor.cursor.x as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE,
            editor.cursor.y as f32 * FONT_CHAR_HEIGHT as f32 * FONT_SCALE,
        );

        camera_vel = (cursor_pos - camera_pos) * Vector2::from_scalar(2.0);
        camera_pos += camera_vel * Vector2::from_scalar(DELTA_TIME);

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        let projection = camera_pos - window_size / Vector2::from_scalar(2.0);

        render_text(
            &mut canvas,
            &mut font,
            &editor,
            projection,
            Color::WHITE,
            FONT_SCALE,
        )?;
        render_cursor(&mut canvas, &mut font, &editor, projection, cursor_pos)?;

        canvas.present();
        let duration = timer.ticks() - start;
        let delta_time_ms = 1000 / FPS;
        if duration < delta_time_ms {
            timer.delay(delta_time_ms - duration);
        }
    }

    Ok(())
}
