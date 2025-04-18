use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use stb_image::stb_image::stbi_load;
use std::ffi::CString;

use red::editor::Editor;
use red::vector2::Vector2;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const FPS: u32 = 60;
const DELTA_TIME: f32 = 1.0 / FPS as f32;

const FONT_SCALE: f32 = 3.0;
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
    let mut channels = 0;
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

fn main() -> Result<(), String> {
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
